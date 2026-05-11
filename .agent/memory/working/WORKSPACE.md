# Workspace (live task state)

## Current task
ITER-0004d.1 — **IN PROGRESS, paused 2026-05-11 night**. 4 of ~16 task units complete. All sentinels green (1203 tests passing, 182 ignored). Next entry point: T8 (`lib/core/fmpl_parser.fmpl`) — but FIRST decide scope on the grammar-parser pattern syntax migration (see scope gap below).

## Tonight's commits (most recent first)
- `819d2ae1` — feat(parser): explicit rejection of :Tag(args) + migrate residual sites (T6 + T13 partial)
- `40bdb474` — test(diagnostics): delete legacy-syntax-validation test files (T3 + T4)
- `70230021` — feat(parser/compiler/tests): list-pattern match support + legacy syntax sweep (T2 + T2b)

## Scope gap surfaced 2026-05-11 (user-flagged)
T6 only rejects `:Tag(args)` in **FMPL expression** context. The **grammar parser** (`fmpl-core/src/grammar/parser.rs:1309-1140`) still parses `:Tag(child_patterns)` as a pattern and emits `Pattern::TagMatch`. The grammar-DSL test fixtures (T13) only got their action bodies migrated — the pattern half (e.g. `expr = :Tagged(tag, expr*:args) => ...`) is still legacy.

Full canonical transition requires:
1. Grammar pattern syntax `:Tag(child_patterns)` → `[:Tag, child_patterns]` list-pattern in grammar context
2. Reject `:Tag(args)` in grammar parser (parallel to T6 but in grammar/parser.rs)
3. Delete `Pattern::TagMatch` — 27 references across compiler.rs, vm.rs, grammar/runtime.rs, grammar/trampoline.rs, pattern/mod.rs, repr.rs (T14, larger than originally scoped)

**Open question for resume:** Does T8/T13/T14 scope-expand to cover this, or do we split a new task (T13b? T14b?) for grammar-pattern migration? Estimated 2-3 more sessions for the full migration.

## Tasks remaining in ITER-0004d.1
- **T7**: Delete orphan fmpl tests (scope item 5)
- **T8**: Update `lib/core/fmpl_parser.fmpl` (scope item 6) — may expand depending on grammar-parser scope decision
- **T9**: Delete `Expr::Tagged` AST variant (scope item 7). Producers: parser.rs:637 (unreachable post-T6), compiler.rs:869, repr.rs:225, grammar_to_ir.rs:311, ast.rs:27, value_to_ast.rs:358, ir_to_rust.rs:1440
- **T10**: Delete `ast_to_ir.fmpl :Tagged` rule (scope item 8)
- **T11**: Delete `ast::Pattern::Constructor` variant (scope item 9)
- **T12**: Delete `pattern::Pattern::Tagged` variant (scope item 10)
- **T13 (remaining half)**: grammar-DSL pattern fixtures (scope item 12)
- **T14**: Delete `pattern::Pattern::TagMatch` (scope item 11) — big blast radius
- **T15**: Repair STORY-0095/AC-4 text (scope item 13)
- **T16**: Update EPIC-002 AC scenario tags (scope item 14)
- **T17**: Reconcile/add scenarios (scope item 15)
- **T18**: Flip no_legacy_fmpl_syntax CI gate to `== 0` mode (scope item 16)
- **T19**: Implement SCENARIO-0104, 0105, 0106 tests (evidence)

## FOLLOWUP outside this iteration
- **task #30** — Align `ir::compile` match-arm semantics with legacy compiler. Two `#[ignore]` tests in `ast_to_ir_parity.rs` (`parity_match_tagged_arity_mismatch_routes_to_fallthrough`, `parity_match_tagged_nested`) await: (a) strict-arity MatchTag, (b) recurse into nested Pattern::List children.

## Sentinel baseline (current, 2026-05-11 night)
- `cargo test -p fmpl-core --no-fail-fast`: **1203 passed, 182 ignored** (71 suites)
- `cargo test -p fmpl-core --test ast_to_ir_parity`: 57 passed, 2 ignored
- `cargo test -p fmpl-core --test scenario_0103_optimizer_pipeline`: 32 passed, 1 ignored
- `cargo test -p fmpl-core --test no_legacy_fmpl_syntax`: 1 passed (baseline: tests/rs=108, src/rs=38, tests/fmpl=72, lib/core=0)
- `cargo test --test tavern_demo`: 6 passed (migrated to list-pattern + explicit `;` separators in dispatch block)

## Active findings carried forward
- **Inline pattern block parsing edge case**: FMPL inline pattern blocks (`expr @ { pat => body, ... }`) can't disambiguate end-of-body from start-of-`[...]`-pattern without explicit separators. Discovered tonight when tavern.fmpl dispatch arms migrated; fixed by adding `;` separators. Document as known constraint or improve parser look-ahead (deferred).
- **`is_at_rule_start` heuristic in grammar action parser** correctly breaks on newline+pattern-start chars, so grammar rule action bodies of form `=> [:Tag, n]\n  next_rule = ...` work without separator. Different from FMPL inline pattern blocks.

## Checkpoints
- [x] ITER-0004d.0 closed 2026-05-10
- [x] ITER-0004d.1 PAR scope review complete (7 rounds; APPROVE)
- [x] T1 baseline confirmed
- [x] T2+T2b parser/compiler list-pattern + bulk test sweep
- [x] T3+T4 delete legacy test files + dead :TagMatch arms
- [x] T6+T13(partial) FMPL parser rejection + residual fixture migration
- [ ] **T7-T19 + grammar-parser scope decision**
- [ ] Post-iteration scenario runs (step 8)
- [ ] Resolve cross-iteration TODOs (step 9)
- [ ] Wrap up — mark stories done, log entry, roadmap update (step 10)

## Next step (resume instructions)
1. Read `docs/superpowers/iterations/progress.md` for the rationale + scope-gap details
2. Decide with user: scope-expand T8/T13/T14 to cover grammar-parser pattern syntax migration, or split into new tasks (T13b/T14b)
3. Resume with chosen task. Recommended starting point: T8 (`lib/core/fmpl_parser.fmpl`) once scope is settled.
4. Sentinels are GREEN — no regressions to recover from.
