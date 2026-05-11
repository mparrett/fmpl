# Progress

**Phase:** ITER-0004d.1 in progress — Tier 1/3 findings cleanup landing; T7-T19 next
**Iterations:** 6/11 done; ITER-0004d.1 in progress
**Sentinel corpus (2026-05-12):** cargo test -p fmpl-core: 1201 passed, 182 ignored (71 suites). ast_to_ir_parity 57/57 (2 #[ignore]); scenario_0103 32/32 (1 ignored); tavern_demo 6/6; no_legacy_fmpl_syntax baseline regenerated (lib/core=0, src/rs=29, tests/fmpl=72, tests/rs=4).
**Last event:** 2026-05-12 — Tier 1/3 findings from 2026-05-11 review pass addressed in working tree (F3 doc fix, F2+F1+F9 combined parser-rejection commit, MF1 root-cause fix in `parse_inline_pattern_block`, F12+F13+F18 progress.md correction). No commits yet — staged together for one or more coherent commits.

## Session of 2026-05-12

Resumed from the paused state captured on 2026-05-11. Per `.agent/memory/working/reviews/iter-0004d.1/FINDINGS.md`, addressed Tier 1 + Tier 3 findings before continuing with T7-T19:

**F3 (Tier 1 doc fix).** Replaced the misleading doc comment in `fmpl-core/tests/ast_to_ir_parity.rs:475-481` that claimed `ir::compile`'s emit path produces "same MatchTag + ExtractTaggedChild" as the Rust compiler. It actually emits `LoadVar → GetProp(tag) → LoadSymbol → Eq → JumpIfFalse → ExtractTaggedChild` (no MatchTag, no arity check; see `builtins/ir.rs:946-1011`). Comment now states the divergence and notes that `assert_pipeline_parity` asserts equal results, not equal bytecode. FOLLOWUP #30 still owns the arity-check + nested-pattern alignment.

**F2 + F1 + F9 (Tier 1 parser rejection).** Combined commit per DESIGN-001 "same parser in end-state." Three rejection sites added, all returning `Error::Parser` with the polished message `"legacy :Tag(...) constructor pattern is not supported; use [:Tag] or [:Tag, ...] instead"` (F9 fix):
  - `fmpl-core/src/parser.rs:1839` — FMPL pattern-position rejection in `parse_pattern` (mirrors `parse_primary`'s T6 rejection at :619).
  - `fmpl-core/src/grammar/parser.rs:874` — grammar top-level pattern (`parse_primary`).
  - `fmpl-core/src/grammar/parser.rs:1316` — grammar value pattern (`parse_value_pattern`).

The previous third grammar-DSL site at `parse_tag_child_pattern` was the entry point for nested `:Tag(args)` children. With all three top-level callers rejecting, `parse_tag_child_pattern` became dead code (131 lines, 0 callers). Deleted entirely.

Test sweeps to keep the suite green after the rejection went live (all are syntax-only conversions of behavior-preserving patterns from `:Tag(args)` to `[:Tag, args]`):
  - `fmpl-core/src/grammar/parser.rs:2067-2118` — three internal tests resolved per roadmap step 12: `test_parse_star_quantifier_in_tag_child` and `test_parse_rule_binding_in_tag_child` deleted (asserted that a removed syntactic form parses; behavior subsumed by list-pattern tests). `test_parse_tag_in_list_pattern` rewritten as `test_parse_nested_list_pattern` using list-pattern children.
  - `fmpl-core/tests/fmpl_interpreter.rs` — 95 raw-string blocks converted (mostly inline match arms like `:Int(n) => [:LoadInt, n]` → `[:Int, n] => [:LoadInt, n]`).
  - `fmpl-core/tests/integration_pattern_unification.rs` — 7 destructuring sites (`let (:Some(x) = ...)` → `let ([:Some, x] = ...)`) + 1 match-arm sweep that landed earlier.
  - `fmpl-core/tests/integration_polymorphic_streams.rs` — 4 match-arm sites.
  - `fmpl-core/tests/metaprogramming.rs` — 4 raw-string blocks.
  - `fmpl-core/tests/yield.rs` — 2 raw-string blocks.

Sweep tooling: `/tmp/sweep_tag_args.py` — string-aware char-walker that finds `r#"..."#` Rust raw-string bodies and converts `:Tag(args)` to `[:Tag, args]` recursively, skipping FMPL string literals (`"..."`) and Rust module paths (`module::method`). Validated on 9 representative inputs before applying to files.

**MF1 (Tier 1 root-cause fix in `parse_inline_pattern_block`).** Two adjacent cases with `[`-led bodies (e.g., `[:Unary, :-, [:Int, n]] => [:Neg, [:LoadInt, n]]` followed by `[:Int, n] => [:LoadInt, n]`) failed to parse because `parse_postfix` greedily consumed the next case's `[` as an `Expr::Index` postfix on the first case's body. Per the user, the comma/semi separator must remain optional (the demo/tavern.fmpl `;` workaround added during T6 was a band-aid). 

Fix: added `in_pattern_case_body` flag to `Parser` and `newline_before_current_token()` source-byte check (uses the token spans already stored on `SpannedToken`). When the flag is set AND there's a newline between the previous token and a `[`, `parse_postfix` breaks out of its loop instead of consuming the bracket — yielding control back to `parse_inline_pattern_block`'s case loop. Flag is scoped to the body parse inside `parse_pattern_case`. When `source` is unavailable (the `Parser::new` constructor path), the heuristic is conservative (returns false; legacy behavior). 

Removed the `;` workarounds added by T6 in `demo/tavern.fmpl:47-49` and the `;` separator added today in `fmpl-core/tests/fmpl_interpreter.rs:51` and `fmpl-core/tests/metaprogramming.rs:60`. All three previously-blocked patterns now parse cleanly. tavern_demo 6/6 green.

**F12 + F13 + F18 (Tier 3 audit-trail repair).** Corrected the narrative below to match what the commits actually did. Cannot retroactively fix commit messages, so the discrepancies are documented here.

## Audit trail corrections (F12 + F13 + F18)

The 2026-05-11 progress.md narrative contained three audit-trail errors flagged by review reviewers (PAR-A, PAR-B):

- **F13 (the `163` number).** The earlier narrative said `tests/rs 625 → 163 → 108` across two commits. The actual transition is `tests/rs 625 → 108` in a single commit (f4b91cef "list-pattern match support + legacy syntax sweep"). The intermediate `163` value never existed as a baseline. Commit 6abab103 modified `lib/core/grammar_optimizer.fmpl` and `.agent/memory/episodic/AGENT_LEARNINGS.jsonl` only — it did not move the tests/rs hit count.

- **F12 (commit 6abab103's message).** The commit message reads "test(diagnostics): delete legacy-syntax-validation test files (ITER-0004d.1 T3)" but the actual diff shows only the `grammar_optimizer.fmpl` edits described above. The legacy-syntax-validation test file deletions (`tagged_pattern_match.rs`, `tagged_values.rs`, the tagged-value block in `generated_parser_correctness.rs`) happened in f4b91cef, not 6abab103. Cannot retroactively amend the commit message.

- **F18 (reversed line range).** The earlier narrative cited `fmpl-core/src/grammar/parser.rs:1309-1136` — start greater than end. The actual `Pattern::TagMatch` construction sites were at lines 899, 1136, 1333 (now deleted/replaced with rejection sites per F1 above).

## Current baseline (post Tier 1/3 cleanup)

`no_legacy_fmpl_syntax.baseline.json`:
```json
{
  "lib/core": 0,
  "src/rs": 29,
  "tests/fmpl": 72,
  "tests/rs": 4
}
```

`tests/rs` ratcheted 108 → 98 (initial integration_pattern_unification + integration_polymorphic_streams sweep) → 88 (further destructuring sweep) → 4 (broader fmpl_interpreter + metaprogramming + yield sweep). The remaining 4 hits are residual FMPL `module:function(args)` calls or fixture content the gate's `Symbol+LParen` heuristic doesn't distinguish from the legacy `:Tag(args)` form; these will be addressed in T18 when the gate flips to `== 0` mode.

`src/rs` ratcheted 38 → 29 from the deletion of `parse_tag_child_pattern` and the two grammar-parser test functions (their `:Tag(args)` fixtures lived in `src/grammar/parser.rs`).

## Remaining work (T7–T19)

**T7:** Delete orphan `fmpl-core/tests/fmpl/{ast_to_ir,fmpl_parser}.fmpl` (scope item 5)

**T8:** Update `lib/core/fmpl_parser.fmpl` — delete `tagged_*`, `pat_constructor`, `tag_name` rules; update `primary` and `pat_primary` rules (scope item 6). Grammar-parser scope expansion (open question at session pause) is now answered: F1+F2 went into the rejection task above, so T8 only needs to update the FMPL stdlib parser to match. Probably one session.

**T9:** Delete `Expr::Tagged` AST variant (scope item 7). Per cargo-check ground truth, surviving consumers are:
  - `fmpl-core/src/compiler.rs:869` (arm emitting `Instruction::MakeTagged`)
  - `fmpl-core/src/repr.rs:225` (Display impl)
  - `fmpl-core/src/builtins/grammar_to_ir.rs:311`
  - `fmpl-core/src/builtins/ast.rs:27` (`expr_to_value` arm)
  - `fmpl-core/src/builtins/ir_to_rust.rs:1440`
  - `fmpl-core/src/value_to_ast.rs:358` (decoder)
  - `fmpl-core/src/ir_builder.rs:239-240` (`fn tagged`, now zero-caller — delete)
  - Variant definition: `fmpl-core/src/ast.rs:27` (numbering may have shifted; verify)

**T10:** Delete `ast_to_ir.fmpl :Tagged` rule (scope item 8)

**T11:** Delete `ast::Pattern::Constructor` variant (scope item 9). With F2's rejection, all parser producers are gone; deletion is pure cleanup of consumer arms.

**T12:** Delete `pattern::Pattern::Tagged` variant (scope item 10).

**T13 (remaining):** Already done in this session as part of F1 — three grammar/parser.rs internal tests resolved.

**T14:** Delete `pattern::Pattern::TagMatch` variant (scope item 11). With F1's rejection, all three grammar-parser producers are gone; deletion is pure cleanup. The consumer surface (compiler.rs:3638, :4360-4392; runtime.rs:784, :821; trampoline.rs:999; optimizer.rs:221; grammar_to_ir.rs:234; pattern/mod.rs:281; repr.rs:616; plus `fn contains_repeat` at pattern/mod.rs:169 which becomes orphan).

**T15:** Repair STORY-0095/AC-4 text in EPIC-032.md:21 (scope item 13).

**T16:** Update EPIC-002 STORY-0010 AC-9/AC-10/AC-12 scenario tags (scope item 14).

**T17:** Reconcile/add scenarios in behavior-scenarios.md — SCENARIO-0039 rewrite, SCENARIO-0066 rewrite, SCENARIO-0104 NEW, SCENARIO-0105 NEW, SCENARIO-0106 NEW (scope item 15).

**T18:** Flip `no_legacy_fmpl_syntax.rs` CI gate from baseline mode to `== 0` mode (scope item 16). Cleans up the residual 4 hits in tests/rs and 29 in src/rs that survive the Tier 1/3 cleanup.

**T19:** Implement SCENARIO-0104, 0105, 0106 evidence tests.

## Resume notes (next session)

1. Read `docs/design-principles.md` first.
2. Read this file.
3. Decide whether to commit the Tier 1/3 cleanup as one big commit or split (recommend: 3 commits — `F3+F12+F13+F18 audit trail`, `F2+F1+F9+test sweeps parser rejection`, `MF1 root-cause fix`).
4. After commits land, start T7. The next-clean entry point is `T7` (orphan fmpl test deletion), then `T8` (lib/core/fmpl_parser.fmpl), then `T9-T14` in order.
5. FOLLOWUP #30 (ir::compile arity check + nested pattern alignment) remains outside this iteration.

## Tier 2 + Tier 4 findings deferred to T-task execution

These findings fold naturally into the T-tasks that touch the same files:

- **F4** (emit_tagged_pattern_match nested pattern gap) → FOLLOWUP #30
- **F5** (ast::pattern_to_value PatList tail silent-drop) → revisit during T11/T14
- **F7** (PatList no-leading-symbol wildcard) → revisit during T11/T14
- **F8** (grammar action body confusing diagnostic) → minor, can fix during T8 or defer
- **F10** (cross-parser equivalence gap) → minor; F2's rejection makes both parsers reject equivalently, addressing the spirit
- **F11** (compile_pattern_binding zero test coverage) → opportunistic during T11
- **F14** (:ListMatch/:MapMatch identity arms in grammar_optimizer.fmpl) → during T8 or T11
- **F15** (vacuous doc comment in builtins/ir.rs:944) → during T9 cleanup
- **F16** (`:TagMatch` arm deletion safety) — verified, no action
- **F17** (no_legacy_fmpl_syntax baseline asymmetric scan) → resolved when T18 flips to `== 0` mode
- **F19** (incorrect claim at roadmap.md:460) → fix during T11 (the step that scope-touches that area)
- **F20** (compiler.rs intentional symmetry) → defer until T11 deletes the legacy arms

## Sentinel green status (2026-05-12)

- ast_to_ir_parity: 57 passed, 2 ignored (one `#[ignore]` waits on FOLLOWUP #30 arity-check; the other on nested pattern recursion in `ir::compile`)
- scenario_0103_optimizer_pipeline: 32 passed, 1 ignored
- tavern_demo: 6 passed (no `;` workarounds — MF1 fix landed)
- no_legacy_fmpl_syntax: 1 passed (baseline regenerated; flip to `== 0` is T18)
- Full fmpl-core suite: 1201 passed, 182 ignored across 71 suites

Sentinels are GREEN. No regressions to recover from on next-session resume.
