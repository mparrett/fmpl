# Progress

**Phase:** ITER-0004d.1 in progress — 4 of ~16 task units complete; pause point 2026-05-11
**Iterations:** 6/11 done; ITER-0004d.1 in progress
**Sentinel corpus (2026-05-11):** cargo test -p fmpl-core: 1203 passed, 182 ignored (no regressions). ast_to_ir_parity 57/57 (2 #[ignore]); scenario_0103 32/32 (1 ignored); no_legacy_fmpl_syntax baseline regenerated (tests/rs 163→108, src/rs 43→38)
**Last event:** 2026-05-11 — paused for the night. T6 + T13 partial landed. Identified scope gap: grammar parser still accepts `:Tag(args)` pattern syntax that needs the same treatment as FMPL expression syntax.

## Completed this session (2026-05-11)

**T2+T2b** (commit 70230021): Extended parser heuristic to recognize `[:Symbol, ...]` inline patterns in match arms, unblocking list-pattern migration. Added 3 compiler arms (`compile_match`, `compile_match_bindings`, `compile_pattern_binding`) + refactored `ir::compile` Match arm with shared `emit_tagged_pattern_match` helper. Swept 12 test files converting `:Tag(args)` → `[:Tag, ...]`. Added 4 parity tests (2 `#[ignore]` expose pre-existing `ir::compile` gaps tracked as FOLLOWUP #30).

**T3+T4** (commit 40bdb474): Deleted `tagged_pattern_match.rs`, `tagged_values.rs`; removed tagged-value test block from `generated_parser_correctness.rs`; deleted 4 dead `:TagMatch` arms from `lib/core/grammar_optimizer.fmpl` (null_opt, associative_opt, empty_elim_opt, jump_table_opt).

**T6 + T13 (partial)** (commit 819d2ae1): FMPL expression parser explicitly rejects `:Symbol(...)` with a clear error directing users to `[:Symbol, ...]`. Migrated three residual sites that surfaced when the rejection turned on:
- `demo/tavern.fmpl`: grammar action bodies (`=> :Talk(n)`, `=> :Order(i)`) and dispatch arms migrated to list-pattern. Required adding explicit `;` separators in the dispatch block because the FMPL inline pattern parser couldn't disambiguate "end of body" from "start of next pattern" when the next pattern starts with `[`.
- `fmpl-core/src/grammar/parser.rs`: action bodies in three unit-test fixtures (`test_parse_star_quantifier_in_tag_child`, `test_parse_rule_binding_in_tag_child`, `test_parse_tag_in_list_pattern`) — pattern syntax intentionally left as `:Tag(args)` (see scope gap below).
- `no_legacy_fmpl_syntax.baseline.json`: regenerated.

**T5** (folded into T6 commit): Swept `fmpl-core/src/*.rs` for FMPL source strings containing `:Tag(args)` — none found in doctest examples or string literals.

## Scope gap discovered 2026-05-11

T6's FMPL parser rejection is **necessary but not sufficient**. The grammar parser (`fmpl-core/src/grammar/parser.rs:1309-1136`) has its own `:Tag(child_patterns)` syntax that emits `Pattern::TagMatch`. For full canonical-form transition, the grammar pattern syntax must also migrate:

- `:Tag(child_patterns)` in grammar pattern context → `[:Tag, child_patterns]` list-pattern
- Pattern::TagMatch (27 references across compiler.rs, vm.rs, grammar/runtime.rs, grammar/trampoline.rs, pattern/mod.rs, repr.rs) deletion (T14)

This crosses grammar runtime, trampoline state machine, and VM dispatch. Bigger than originally planned for T8/T13/T14 — estimated 2-3 more sessions.

## Remaining work (T7–T19 + scope gap)

**T7:** Delete orphan fmpl tests (scope item 5)
**T8:** Update `lib/core/fmpl_parser.fmpl` — migrate `:Tag(args)` pattern syntax to list form (scope item 6). Now larger than originally scoped due to grammar pattern syntax also needing migration.
**T9:** Delete `Expr::Tagged` AST variant (scope item 7). Producers in 7 sites incl. parser.rs:637 (now unreachable post-T6), compiler.rs:869, repr.rs:225, grammar_to_ir.rs:311, ast.rs:27, value_to_ast.rs:358, ir_to_rust.rs:1440.
**T10:** Delete `ast_to_ir.fmpl :Tagged` rule (scope item 8)
**T11:** Delete `ast::Pattern::Constructor` variant (scope item 9)
**T12:** Delete `pattern::Pattern::Tagged` variant (scope item 10)
**T13 (remaining):** Migrate grammar-DSL pattern fixtures `:Tag(args)` → `[:Tag, args]` after grammar parser supports it (scope item 12). Action-body half already done in T6 commit.
**T14:** Delete `pattern::Pattern::TagMatch` variant — large blast radius across grammar runtime + trampoline + VM (scope item 11)
**T15-T18:** Repair STORY-0095/AC-4 text, update EPIC-002 AC scenario tags, reconcile/add scenarios, flip `no_legacy_fmpl_syntax` CI gate to no-baseline mode
**T19:** Implement SCENARIO-0104, 0105, 0106 tests (evidence)

**FOLLOWUP task #30:** Align `ir::compile` match-arm semantics with legacy compiler (arity check + nested pattern support) — two `#[ignore]` tests in `ast_to_ir_parity.rs` await this.

## Resume notes

When picking up: the next clean entry point is **T8** (update `lib/core/fmpl_parser.fmpl`). Before starting, decide whether to scope-expand T8/T13/T14 to cover the grammar-parser `:Tag(args)` pattern syntax migration, or split out a new task (T13b?) for that. Recommendation: discuss with user before deciding — this changes the iteration's shape.
