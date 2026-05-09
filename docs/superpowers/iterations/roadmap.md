# Roadmap

## Walking skeleton (ITER-0000)

**Intent:** Establish the bootstrap parity test harness and sentinel corpus, verifying the currently-passing subset of the FMPL compilation pipeline (source → ast::parse → ast_to_ir.fmpl → ir::compile → code::eval) produces correct results for basic expressions.

**Design rationale:** Phase 1 (parser cutover) is already complete. ITER-0000 formalizes the test harness, confirms the passing tests as the sentinel corpus baseline, and establishes the regression gate for all subsequent iterations.

**Journey scenario:** SCENARIO-0016 (parity contract)

**Stories committed:**
- STORY-0007 (EPIC-002)

**Status:** done

---

## Iteration list

### Completed iterations

### ITER-0001 — Parity: Core Expression Coverage

**Stories:** STORY-0043, STORY-0044, STORY-0045, STORY-0046, STORY-0047, STORY-0048
**Status:** done
**Result:** 36/55 parity tests passing. IR compilation layer fully verified.

### ITER-0002 — Parity: Control Flow and Bindings

**Stories:** STORY-0006, STORY-0008, STORY-0049a
**Status:** done
**Result:** Fixed grammar engine binding scoping bug (sub-runtime rule_depth). Unblocked arithmetic, string, let, if, sequence parity.

### ITER-0003 — Parity: Advanced Language Features + ir::compile Gaps

**Stories:** STORY-0049b, STORY-0009
**Status:** done
**Result:** Fixed Star-in-TagMatch list unwrapping, `args` keyword collision, map pair binding, vacuous all() check, added ir::compile handlers for While/For/Block/Pipe/Slice/Match/TryCatch, added tagged value introspection, added InlinePatternBlock→Match in ast::parse. **55/55 parity tests passing, 0 ignored.**

---

## Remaining iterations (critical path to self-hosting)

### ITER-0004 — Optimizer Integration and Compiler Retirement

**Stories:** STORY-0010, STORY-0012, STORY-0011, STORY-0005
**Rationale:** Integrate ast_optimizer.fmpl (constant folding, algebraic simplification) into the bootstrap compilation pipeline between ast_to_ir.fmpl and ir::compile. Verify parity is preserved with optimization enabled. Then retire the Rust compiler from the main compilation path — the FMPL pipeline becomes the default, with the Rust compiler retained only in fmpl-bootstrap as the stage 0 fallback. This is the **compiler cutover milestone**.
**Status:** done (compiler cutover wired; optimizer integration deferred — ast_optimizer.fmpl uses list-based patterns and needs the AST refactor first)
**Impacted scenarios:** SCENARIO-0003, SCENARIO-0016
**Depends on:** ITER-0003 (complete)
**Look-ahead check:** Completes Phase 2. Unblocks self-compile (ITER-0006). Persistence (ITER-0005) must follow ITER-0004b for representation stability.

**Delivered:**
- `eval_via_fmpl_pipeline()` — runs source through ast::parse → ast_to_ir.fmpl → ir::compile → code::eval
- `eval_via_rust_compiler()` — explicit Rust-compiler path (formerly the only `eval`)
- `FMPL_USE_FMPL_COMPILER=1` opt-in flag for FMPL pipeline as default
- 11 E2E tests (`fmpl_pipeline_compiler.rs`) verify identical results: integer, arithmetic with precedence, string, let, if, lambda, list, nested arith, boolean logic, comparison, plus bootstrap caching
- Rust compiler is now an explicit fallback, not the only path

**Deferred to ITER-0004b:**
- STORY-0010 (ast_optimizer.fmpl integration — needs list-based AST refactor; STORY-0012 consolidated into STORY-0010 as duplicate)
- Removing `FMPL_USE_FMPL_COMPILER` opt-in and making FMPL pipeline the default everywhere

### ITER-0004b — Single Canonical Representation (Lists Everywhere + Burn the Bridge)

**Stories:** STORY-0010 (consolidated: STORY-0012 absorbed as duplicate scope; STORY-0010c absorbed because the cutover and the cleanup are one refactor)
**Rationale:** Today FMPL has two interchangeable shapes for tagged/structured data: `Value::Tagged(tag, children)` and `Value::List([Symbol(tag), ...children])`, plus two parser surfaces: `:Tag(args)` and `[:Tag, args]`. This iteration collapses both axes to a single canonical representation: list-shaped values, list-shaped patterns. After this iteration there is exactly one way to represent and pattern-match structured data, no parallel codepaths, no parser ambiguity, no runtime ambiguity. **This iteration MUST land before ITER-0005** — see "Why before persistence" below.

The cutover (make `ast::parse` emit lists; FMPL pipeline consume lists; integrate the optimizer) and the cleanup (delete `Value::Tagged`, `Expr::Tagged`, `Pattern::Constructor`, the `:Tag(args)` parser productions, tagged bytecode, `Pattern::TagMatch`) are one refactor. Splitting them was attempted in the 2026-05-08 session and produced a worse interim state (parallel representations, dual codepaths, more code to maintain) than either before or after the full cleanup. They land together or not at all.

**Status:** partially shipped 2026-05-08 — Rust runtime canonicalized; FMPL stdlib + AST/parser surfaces deferred to ITER-0004c. See iteration-log.md ("ITER-0004b — Single Canonical Representation (partial)") for details.
**Impacted scenarios:** SCENARIO-0003, SCENARIO-0016, SCENARIO-0039 (touched, ongoing); SCENARIO-0103 (NEW — blocked, optimizer not yet wired)
**Depends on:** ITER-0004 (compiler cutover wired)
**Look-ahead check (revised):** **Partial — does NOT yet fully lock in a single representation.** `Value::Tagged` is gone, but `Expr::Tagged`, `Pattern::Constructor`, the `:Tag(args)` parser production, and `Pattern::TagMatch` are still present (the parser silently translates `:Tag(args)` to list-shaped values at compile time via the surviving AST nodes). 5 stdlib files (`ast_optimizer.fmpl`, `fmpl_parser.fmpl`, `ir_to_rust.fmpl`, `prelude.fmpl`, `ir_to_execution_tape.fmpl`) still hold legacy `:Tag(args)` syntax. ITER-0005 (persistence) is technically unblocked because snapshots will only see `Value::List` — the runtime variant is the one that lives in serialized bytes — but the AST and parser ambiguity remains until ITER-0004c lands. ITER-0006 (self-compile seed) is **blocked** because the FMPL transformer was never built, so the stdlib can't be regenerated mechanically from source.

**What actually shipped (Rust side only):**
- Phase A items 1–2 and 5: `Value::list_node` + `Value::as_node` helpers, ast-grep rule files at `tools/list-transform/rust-rules/`, hand-tested.
- Phase B items 6, 9: Ran ast-grep over fmpl-core (229 mechanical rewrites). Updated `expr_to_value` and `ir::compile_node` for list-only dispatch (commit `qworqxrm`).
- Phase B item 7 partial: `lib/core/ast_to_ir.fmpl` was rewritten **by hand** (no FMPL transformer was ever built — items 3, 4, 7 in the original plan never executed).
- Phase C item 13: `Value::Tagged` enum variant deleted. The Rust type-system burn is complete.

**What was deferred (now ITER-0004c):**
- Phase A item 3 (FMPL transformer build) — never started.
- Phase B item 7 (FMPL transformer applied to all stdlib files) — only `ast_to_ir.fmpl` got rewritten, by hand. 5 files still in legacy syntax.
- Phase B item 10 (optimizer wired into `eval_via_fmpl_pipeline`) — `ast_optimizer.fmpl` is still in legacy syntax and not called by any pipeline. The 16 `#[ignore]`'d tests in `optimizer_integration.rs` remain ignored.
- Phase B item 12 (SCENARIO-0103) — added but blocked.
- Phase C items 14–18 (delete `Expr::Tagged`, `Pattern::Constructor`, tagged bytecode, `Pattern::TagMatch`, the `:Tag(args)` parser production) — not started. The Rust type system permits both shapes; the parser still accepts both syntaxes; the runtime value layer is the only place that's truly canonicalized.

**Scope:**

**Strategy:** Transformer-driven rewrite, not hand-edit-driven. The 2026-05-08 attempt confirmed that hand-editing ~349 sites mid-session burns context faster than it converges. The plan below uses two structural code transformers that do the bulk of the work mechanically, leaving only the irreducibly novel work (helper additions, deletions, optimizer integration) for hand-editing.

- **Rust side:** [ast-grep](https://ast-grep.github.io/) (already installed at `~/.cargo/bin/ast-grep`). Pattern-based structural rewrite using YAML rule files. Idempotent — re-running on its output yields no diff.
- **FMPL side:** A small FMPL-in-FMPL transformer (a tree grammar) that rewrites `:Tag(args)` → `[:Tag, args]` for both expressions and patterns. Lives at `tools/list-transform/list_transform.fmpl`. Built on FMPL's own parser; dogfoods the language. Also idempotent.

Both transformers are rules + driver, not from-scratch tools. Total transformer code is well under a few hundred lines; the win is having the rewrite be mechanical and re-runnable, not having a fancy tool.

---

**Phase A — Build and validate the transformers:**

1. **Add helpers on `Value`.** `Value::list_node(tag, children) -> Value` constructor producing `Value::List([Symbol(tag), ...children])`. `Value::as_node(&self) -> Option<(&str, &[Value])>` accessor that destructures it. Both transformer outputs depend on these existing first.

2. **Write ast-grep rule files at `tools/list-transform/rust-rules/`.** One YAML file per pattern:

   - `producer-with-args.yml` — matches `Value::Tagged(SmolStr::new($TAG), Arc::new(vec![$$$ARGS]))` → rewrites to `Value::list_node($TAG, vec![$$$ARGS])`. Verified working in the 2026-05-08 session.
   - `producer-empty.yml` — matches `Value::Tagged(SmolStr::new($TAG), Arc::new(Vec::new()))` → rewrites to `Value::list_node($TAG, vec![])`.
   - `producer-non-literal-vec.yml` — matches `Value::Tagged(SmolStr::new($TAG), Arc::new($EXPR))` (where `$EXPR` is not `vec![...]` or `Vec::new()`) → rewrites to `Value::list_node($TAG, $EXPR.to_vec())` or similar. Edge cases captured in `manual-review.md` for human review.
   - `consumer-iflet.yml` — matches `if let Value::Tagged($TAG, $CHILDREN) = $V { $$$BODY }` → rewrites to `if let Some(($TAG, $CHILDREN)) = $V.as_node() { $$$BODY }` (with appropriate `&str`/`&[Value]` reference adjustment).
   - `consumer-match-arm-guard.yml` — matches `match $V { Value::Tagged($T, $C) if $T.as_str() == $TAG => $BODY, $$$REST }` → rewrites to use `$V.as_node()` then if-let-chain on the literal tag.
   - `consumer-match-arm-bind.yml` — matches `Value::Tagged($T, $C) => $BODY` arms in matches → rewrites to use `as_node()`.
   - `display-tagged-string.yml` — matches `format!("{:?}", Value::Tagged(...))` and similar formatter assumptions → flagged in `manual-review.md` because Display output changes.

   Run with `ast-grep scan --rule tools/list-transform/rust-rules/*.yml --update-all` repeatedly until idempotent (no further diffs).

3. **Write the FMPL transformer at `tools/list-transform/list_transform.fmpl`.** A tree grammar that rewrites FMPL ASTs. Two rules:

   ```
   let list_transform = grammar list_transform {
       expr = :Tagged(any:tag, expr*:args) => [tag, args]
            | :Pattern(:Constructor(any:tag, pattern*:pats)) => [:Pattern, [:List, [tag, pats]]]
            | any:other => other  -- recurse via descend rule

       pattern = :Constructor(any:tag, pattern*:pats) => [:List, [tag, pats]]
               | any:other => other
   }
   ```

   Driver: a CLI (`tools/list-transform/transform.rs`, ~50 lines) that walks `lib/**/*.fmpl`, parses each file, applies the transformer, pretty-prints back. Comment-preserving by working at AST-trivia level rather than reformatting.

   **Special-case rules** the transformer applies after the basic rewrite:
   - **Trailing comma for single-element list patterns.** `[expr*:xs] => xs` → `[expr*:xs,] => xs`. Required to disambiguate from char classes in the grammar parser.
   - **Pair sentinel wrap.** `[_:k, expr:v] => [k_ir, v_ir]` (where both children of the result are list-shaped) → `[_:k, expr:v] => [:Pair, k_ir, v_ir]`. Required to prevent the runtime "list-of-lists ⇒ spread" collapse.
   - **List-pattern binding repair.** Bare identifiers in tag-child position become bindings; in list-pattern position they're rule references. Where the input was `:Tag(name)` (binding `name`), the output `[:Tag, name]` would be a rule reference. Rewrite to `[:Tag, any:name]`.

   The special-case rules can be expressed as additional grammar rules in `list_transform.fmpl` (recommended) or as post-processing passes in the driver.

4. **Validate dry-runs.** Both transformers run in `--check` mode and produce:
   - Diff stats: files changed, sites rewritten per rule
   - `tools/list-transform/manual-review.md` listing sites that need human attention
   - Idempotency confirmation: a second run produces zero diffs

5. **Hand-test the transformers** on a small subset (`fmpl-core/src/builtins/ir.rs` for ast-grep; `lib/core/ast_to_ir.fmpl` for the FMPL transformer). Verify the output compiles and tests pass for those files.

---

**Phase B — Apply the transformers; integrate the optimizer:**

6. **Run the Rust transformer for real.** `ast-grep scan --rule tools/list-transform/rust-rules/*.yml --update-all` over the workspace. Expected: ~349 mechanical rewrites land in one pass. Cargo build still works because `Value::list_node` and `Value::as_node` work alongside the still-defined `Value::Tagged` variant.

7. **Run the FMPL transformer for real.** Rewrites `lib/**/*.fmpl` and any inline FMPL string literals in Rust tests (the FMPL transformer's driver scans for FMPL string literals via tree-sitter or a simpler regex). Output: list-pattern syntax everywhere.

8. **Hand-edit the manual-review sites.** The transformer's `manual-review.md` lists sites it couldn't safely rewrite — typically: complex nested patterns, comments referencing Tagged, Display assertions. Walk through these.

9. **Update `expr_to_value` and `ir::compile_node` for list-only dispatch.** The transformer already converted producers and most consumers; this step removes the now-dead Tagged code paths in these two specific files. Use the `ast_node!` and `ast_match!` macros from `fmpl-core/src/macros.rs`.

10. **Wire the optimizer into `eval_via_fmpl_pipeline`** at the correct slot: `ast::parse → ast_optimizer.optimize → ast_to_ir.expr → ir::compile → code::eval`. The optimizer (`lib/core/ast_optimizer.fmpl`) is already in list-pattern form post-transformer; this step adds the call site.

11. **Verify Phase B is green.** Full `cargo test --workspace` passes. The 55 ast_to_ir parity tests pass with optimizer enabled. Tree is in a stable state (lists everywhere, but `Value::Tagged` variant still defined and unused). **This is a natural pause point** — if a session ends here, Phase C is a follow-on, not a redo.

12. **Add SCENARIO-0103: full parity corpus passes with optimizer enabled.**

---

**Phase C — Burn the bridge (delete the dual representation):**

After Phase B the transformer has eliminated almost all `Value::Tagged` references. Now delete the variants, AST nodes, parser productions, and bytecode. Each deletion surfaces a small number of remaining sites the transformer missed; fix those by hand.

13. **Delete `Value::Tagged` enum variant** in `fmpl-core/src/value.rs`, plus its `Display`, `equals`, `index`, `is_truthy`, `type_name`, and unit-test arms. Cargo errors will surface any sites the transformer missed (rare; should be near zero after a clean transformer pass).

14. **Delete `Expr::Tagged`** AST variant. Drop the parser production for `:Tag(args)` value-constructor syntax. Update `compile_expr` and `expr_to_value` to remove the `Expr::Tagged` arms.

15. **Delete `Pattern::Constructor`** AST variant. Drop the parser production for `:Tag(p1, p2)` pattern syntax. Update `compile_match_bindings` to remove the `Pattern::Constructor` arms.

16. **Delete tagged bytecode**: `Instruction::MakeTagged`, `MatchTag`, `ExtractTaggedChild`, `MatchTagged`, `MatchTaggedWithBindings`. The compiler currently emits `MatchTag` for `Pattern::Symbol` and `Pattern::Constructor` — switch the `Pattern::Symbol` case to use list-head dispatch, or rename `MatchTag` to `MatchListNode`.

17. **Delete `Pattern::TagMatch`** from `fmpl-core/src/pattern/mod.rs`. Delete its handlers in `fmpl-core/src/grammar/runtime.rs:784` and `fmpl-core/src/grammar/trampoline.rs:999`. `Pattern::ListMatch` already covers the shape.

18. **Delete grammar parser's `:Tag(args)` pattern production** in `fmpl-core/src/grammar/parser.rs::parse_value_pattern`.

19. **Document optimizer coverage gap** (TODO in `ast_optimizer.fmpl`): Lambda bodies, Let, Match, Call, List, Map, Block fall through unchanged — constants inside them don't fold. Tracked for a future iteration.

20. **Final verification.** Full `cargo test --workspace` passes, zero `Value::Tagged` references in source (`grep -r "Value::Tagged" .` returns no source matches; only doc references in `docs/` remain).

**Explicitly OUT OF SCOPE:**

- **Removing `FMPL_USE_FMPL_COMPILER` opt-in flag.** Default `eval()` still uses `eval_via_native`. Promotion to default is a separate iteration.

**Implementation discipline:**

- **Phases A and B can land independently.** Phase A (transformers) is a small, reviewable, self-contained tool. Phase B (apply transformers + optimizer integration) lands on top and produces a coherent state (lists everywhere; Tagged variant still defined but unused). If a session ends after Phase B, Phase C is a clean follow-on, not a redo. **This is the key benefit of the transformer approach** — it converts a "single huge atomic refactor" into "two reviewable artifacts" without producing an incoherent interim state.
- **Phase C should still be atomic.** Deleting `Value::Tagged` and the parser productions is a single coordinated change.
- **Don't try to keep tests green during Phase C deletions.** Get the build green first (drive cargo error count to zero), then run tests.
- **Tooling first.** Build the transformers fully before running them in anger. A dry-run with manual review of the diff is the cheap insurance.

**Why before persistence:**
ITER-0005 will serialize `ObjectDb`, `CompiledCode`, `GrammarRegistry`, and the full VM image — all of which transitively contain `Value`. With `Value::Tagged` still present, snapshots taken now would be locked to a shape we want to abandon. Landing the canonical-representation refactor first means ITER-0005 persists `Value::List`, the only shape going forward.

### ITER-0004c — FMPL Stdlib Migration + Optimizer Wiring (Phase A of STORY-0010)

**Stories:** STORY-0010 Phase A (AC-3 through AC-7) plus AC-13 (greppable stdlib invariant — the natural close-out of Phase A's stdlib migration). AC-1, AC-2, AC-8, and AC-15 are already satisfied by ITER-0004b's runtime burn (see verification notes below); they are not re-shipped here. Phase B (AC-9, AC-10, AC-11, AC-12, AC-14) split into ITER-0004d per PAR scope review 2026-05-10. Background EPIC-002.md:154 explicitly identifies Phase B as a "natural pause point"; the line 114 "land together" argument applied when Value::Tagged was still in the runtime, which ITER-0004b already removed.

**Already-satisfied AC verification (no re-work needed):**
- AC-1 (`ast::parse` emits list-shaped exclusively): verified at `fmpl-core/src/builtins/ast.rs` — every `expr_to_value` arm returns `Value::list_node(...)`.
- AC-2 (`ir::compile` consumes list-shaped exclusively): verified at `fmpl-core/src/builtins/ir.rs` — `compile_node` dispatches on `Value::as_node()` only.
- AC-8 (`Value::Tagged` enum variant removed): verified — the variant is deleted; `grep -n 'Value::Tagged' fmpl-core/src/value.rs` returns nothing.
- AC-15 (full test suite passes; no `Value::Tagged` source matches): verified at workspace baseline — 1170 passing, no `Value::Tagged` source matches.
**Rationale:** ITER-0004b shipped only the Rust-runtime half of the canonical-representation refactor. The 7 FMPL stdlib files (six listed in the original ITER-0004b plan + `ast_to_ir_indexed.fmpl`, missed in the original list) still use legacy `:Tag(args)` syntax. This iteration: (1) builds the FMPL transformer ITER-0004b's plan called for, (2) applies it to all 7 stdlib files, (3) wires `ast_optimizer.fmpl` into `eval_via_fmpl_pipeline` so the parity corpus actually exercises the optimizer. Acceptance gate is SCENARIO-0103 passing — every parity input matches Rust-compiler output AND at least one demonstrably folds AND no INT_MIN/div-zero panics. The dual-syntax parser surface (`Expr::Tagged`, `Pattern::Constructor`, `Pattern::TagMatch`, tagged bytecode) survives this iteration unchanged — it's permitted but no longer used by the stdlib. ITER-0004d removes it.
**Status:** pending
**Impacted scenarios:** SCENARIO-0103 (sentinel — completes here), SCENARIO-0016 (sentinel — must continue passing with optimizer wired into pipeline). SCENARIO-0003 and SCENARIO-0039 are ITER-0004d concerns (scenario rewrite + reconfirm).
**Depends on:** ITER-0004b (Rust-runtime burn).
**Look-ahead check:** Unblocks ITER-0005 (persistence) — stdlib representation is now stable. Unblocks ITER-0004d (parser/AST burn). Does NOT yet unblock ITER-0006 (self-compile seed) because the parser still accepts the dual syntax; ITER-0006 needs ITER-0004d's burn to guarantee the seed references exactly one AST shape.

**Files in scope (verified 2026-05-10 via `grep -cE ':[A-Z][a-zA-Z_]*\(' lib/core/*.fmpl`):**
- `lib/core/ast_optimizer.fmpl` (62 legacy lines / 156 occurrences) — also not yet wired into pipeline
- `lib/core/fmpl_parser.fmpl` (96 legacy lines / 101 occurrences)
- `lib/core/ir_to_rust.fmpl` (48 legacy lines / 84 occurrences)
- `lib/core/prelude.fmpl` (41 legacy lines / 45 occurrences)
- `lib/core/ir_to_execution_tape.fmpl` (19 legacy lines / 19 occurrences)
- `lib/core/pipeline_demo.fmpl` (2 legacy lines / 10 occurrences) — note: line 5 has a `:Binary(...)` runtime literal that the transformer will rewrite; this file is also a candidate for hand-edit
- `lib/core/ast_to_ir_indexed.fmpl` (24 legacy lines) — was missed in ITER-0004b's original list; flagged broken in `docs/plans/2026-03-03-self-hosting-bootstrap-design.md:28`. Disposition decided in scope item 7 below.

**Scope:**

1. **Build the FMPL transformer** (`tools/list-transform/list_transform.fmpl` + driver). Was Phase A item 3 in ITER-0004b's plan. Tree-grammar rules + special-case rules (trailing comma, pair sentinel wrap, list-pattern binding repair). Driver in Rust (~50 lines) walks the targets in scope item 3.
2. **Validate dry-runs** on `prelude.fmpl` first (cheapest target). Compare transformer output against hand-migration on a 10-rule subset; commit only when output matches.
3. **Apply the FMPL transformer** to all 7 files in the file list. Re-run the transformer until idempotent. Hand-edit transformer-flagged exceptions, and additionally hand-edit:
   - `lib/core/pipeline_demo.fmpl:5` — runtime-parsed expression `let ast = :Binary(:+, :Int(7), :Binary(:*, :Int(9), :Int(5)))` becomes `let ast = [:Binary, :+, [:Int, 7], [:Binary, :*, [:Int, 9], [:Int, 5]]]`. The transformer SHOULD migrate this; it's a sanity check.
   - `lib/core/pipeline_demo.fmpl:9` — `io::println("  :Binary(:+, :Int(7), :Binary(:*, :Int(9), :Int(5)))")` is a string literal containing display text. The transformer must NOT enter string-literal contexts. Hand-edit the displayed text to match the new syntax: `io::println("  [:Binary, :+, [:Int, 7], [:Binary, :*, [:Int, 9], [:Int, 5]]]")`. Without this hand edit, the verification gate (which uses a string-content-blind grep) would fail.
4. **Wire `ast_optimizer.fmpl`** into `eval_via_fmpl_pipeline`. Two edit sites in `fmpl-core/src/lib.rs`:
   - Bootstrap loader: currently lines 121-123 sequence (a) load prelude, (b) load ast_to_ir, (c) set marker. Insert a third load between current line 122 and current line 123 (i.e., before the marker is set, so subsequent invocations skip re-loading). The new call MUST wrap with `let ast_optimizer = io::load(...)` because `lib/core/ast_optimizer.fmpl` ends with a bare module-map literal (`%{ constant_fold: ..., optimize: optimize }`) — there is no internal `let ast_optimizer = ...` binding inside the file. Without the outer let, `io::load` returns the map but no name is bound, and the pipeline at the next step fails with an undefined-name error. Verbatim form: `eval_via_legacy_parser(vm, r#"let ast_optimizer = io::load("lib/core/ast_optimizer.fmpl")"#)?;`. Compare to the existing working pattern at `fmpl-core/tests/optimizer_integration.rs:31-35`. (Note: `prelude.fmpl` and `ast_to_ir.fmpl` are loaded by the existing bootstrap without an outer `let` because the FMPL VM treats every top-level `let name = ...` form *inside* a loaded file as a global binding. `prelude.fmpl` has many such bindings — `let symbol`, `let reduce`, `let fold_binary`, etc. — and `ast_to_ir.fmpl:14` has `let ast_to_ir = grammar ...`. `ast_optimizer.fmpl` has none at the top level — it ends with a bare expression — so the outer let is required to capture the returned map under a name.)
   - Pipeline wrapper at lines 126-129 (the `pipeline_source` format!) — thread `ast_optimizer["optimize"](ast)` between `ast::parse` and `ast_to_ir.expr`. The bracket-index form `ast_optimizer["optimize"](...)` matches the existing pattern in `fmpl-core/tests/optimizer_integration.rs:43`. Final order: `ast::parse → ast_optimizer["optimize"] → ast_to_ir.expr → ir::compile → code::eval`.
5. **Add SCENARIO-0103 execution.** A new test (or extension of an existing one) provides three observables:
   - **(parity)** Run all 55 parity corpus inputs through `eval_via_fmpl_pipeline` (with optimizer wired); assert each result equals the Rust-compiler result for the same input.
   - **(slot)** For at least one input, capture the IR shape *between* `ast_optimizer["optimize"]` and `ast_to_ir.expr` (or equivalently, capture the IR shape that reaches `ir::compile`). Assert that the optimizer ran at the correct slot — the IR contains a folded constant where the AST contained a `Binary` arithmetic expression. This satisfies AC-4 (slot) AND AC-5 (fold fires on real ast::parse output). The "fold fires" observable alone is insufficient because an optimizer running post-IR would produce identical Value-level results — slot-correctness needs a structural assertion.
   - **(guards)** Extend the parity corpus (or add a separate test) with 3 inputs that exercise the optimizer's existing guards in `lib/core/ast_optimizer.fmpl`:
     - `1 / 0` (source-form, exercises div-zero guard at line 5 `&{ b != 0 }`)
     - `1 % 0` (source-form, exercises mod-zero guard at line 6)
     - `:Unary(:-, [:Int, -9223372036854775808_i64])` (constructed directly as a `Value::list_node` AST in Rust — see scope item 8 sub-task — exercises the negation INT_MIN guard at line 15). The FMPL lexer cannot tokenize i64::MIN as a literal, so the source-form path is unavailable; the guard's contract is exercised via direct AST construction.
     Assert each compiles without panic and produces the same result as the Rust compiler. This satisfies AC-3 (guards preserved) — the existing 55-input corpus has zero such cases (verified `grep -nE 'i64::MIN|9223372036854775|/ 0|% 0' fmpl-core/tests/ast_to_ir_parity.rs` is empty). Note: multiplication overflow (e.g., `i64::MIN * -1`) is NOT in scope — `ast_optimizer.fmpl:4` has no multiplication guard, and AC-3 says "keeps" not "adds" guards. Adding a multiplication guard is a follow-on hardening item, not a Phase A deliverable.

   Update `behavior-corpus.md` with the execution command and `behavior-scenarios.md` automation status.
6. **Update SCENARIO-0016 / parity test infrastructure.** `fmpl-core/tests/ast_to_ir_parity.rs:44-67` (`setup_fmpl_pipeline`/`run_full_pipeline`) currently does NOT load `ast_optimizer.fmpl`. AC-6 ("all 55 ast_to_ir parity tests pass with optimizer enabled") is satisfied either by adding the optimizer to this test's pipeline OR by routing through `eval_via_fmpl_pipeline` so SCENARIO-0103 subsumes SCENARIO-0016's optimizer obligation. Pick one; document the choice.
7. **Resolve `ast_to_ir_indexed.fmpl` disposition.** Two options: (a) migrate to list-pattern syntax along with the other 6 files, OR (b) delete from `lib/core/` since it is flagged broken (state-threading bug per design doc). Decision: delete — the working `ast_to_ir.fmpl` is the canonical AST→IR translator; the indexed variant is unused dead code. Move it to `docs/_archive/` if the explorations are worth preserving. **Cascade cleanups when deleting:**
   - `lib/core/ir_to_execution_tape_indexed.fmpl:4,8` reference `ast_to_ir_indexed.fmpl` as their input source. After deletion, this file becomes an orphan input-less consumer. Either delete it as well (recommended — it's same-class dead code) or update its header comment to note that the upstream is gone.
   - `lib/core/pipeline_demo.fmpl:12` has a comment referencing `ast_to_ir_indexed.fmpl` — update to point to `ast_to_ir.fmpl` or remove the demo line.
8. **Un-ignore optimizer_integration tests.** The 17 `#[ignore = "ITER-0004b: requires lists-everywhere refactor + optimizer wired into eval_via_fmpl_pipeline"]` tests in `fmpl-core/tests/optimizer_integration.rs` un-ignored. All 17 must pass.

   **Sub-task: rewrite `ac3_int_min_negation_does_not_panic`** (`tests/optimizer_integration.rs:104-111`). The current source `"0 - (-9223372036854775808)"` cannot tokenize: the FMPL lexer (`fmpl-core/src/lexer.rs:117`) parses integer literals via `[0-9]+` then `parse::<i64>().ok()`, which returns `None` for `9223372036854775808` (one greater than `i64::MAX`). The lexer drops the token and the source never reaches the optimizer. The test was written incorrectly during ITER-0004b. Two valid rewrites:
   - **(a)** Construct the AST node directly: build `[:Unary, :-, [:Int, -9223372036854775808_i64]]` as a `Value::list_node` literal in Rust and feed it to `ast_optimizer["optimize"]` then `ast_to_ir.expr` then `ir::compile` then `code::eval`. Bypasses the lexer.
   - **(b)** Drop the test entirely and add a comment in `ast_optimizer.fmpl` explaining that the `:Unary(:-, [:Int, a])` INT_MIN guard is dead-code-defensive (the lexer cannot produce `:Int(i64::MIN)` so the case is unreachable from FMPL source). Document that the guard is preserved for safety in case a non-source AST construction path is added later.

   Recommendation: **(a)** — keeps the AC-3 observable contract intact and proves the guard's correctness. The Rust-side construction is ~5 lines.

**Verification gates:**
- `cargo test -p fmpl-core --test ast_to_ir_parity` — 55/55 passing (SCENARIO-0016).
- `cargo test -p fmpl-core --test optimizer_integration` — 17/17 passing (no `--ignored` needed).
- SCENARIO-0103's new execution command passes.
- AC-13 (greppable stdlib invariant): `grep -cE ':[A-Z][a-zA-Z_]*\(' lib/core/*.fmpl` returns 0 across all stdlib files. Note: the grep matches inside both `--`-prefixed comments and double-quoted string literals; both must be hand-edited to avoid false-positive failures (see scope item 3 hand-edits). If lingering false positives are unavoidable, switch to a stricter check like `rg --type=fmpl --multiline-dotall '(?<![-\"])\:[A-Z][a-zA-Z_]*\(' lib/core/`, but the simpler grep is preferred — false positives are bugs to fix, not noise to filter.
- Full workspace test suite passes (`cargo test --workspace`).
- TODO comment in `lib/core/ast_optimizer.fmpl` lists AST node kinds that fall through unchanged (Lambda bodies, Let, Match, Call, List, Map, Block) — AC-7.

**Out of scope (deferred to ITER-0004d):**
- Deleting `Expr::Tagged`, `Pattern::Constructor`, `Pattern::TagMatch`, tagged bytecode instructions, parser productions for `:Tag(args)`.
- Sweeping FMPL source strings inside Rust test files (`tests/parser_equivalence.rs:82-85`, `tests/tagged_values.rs`, `tests/tagged_pattern_match.rs`, `tests/fmpl_interpreter.rs`, `tests/ast_to_ir_parity.rs:88-122`, etc.).
- New parse-rejection scenarios (added in ITER-0004d).
- Reconciling SCENARIO-0039 (uses `:int(n)` value-pattern syntax) and SCENARIO-0066 (references `Value::Tagged`).
- Updating `fmpl-core/src/grammar/parser.rs:2072` internal grammar string `:Tagged(tag, expr*:args) => :MakeTagged(tag, args)` (relevant when MakeTagged is deleted in ITER-0004d).
- Removing `FMPL_USE_FMPL_COMPILER` opt-in.

### ITER-0004d — Parser/AST/Bytecode Burn (Phase B of STORY-0010)

**Stories:** STORY-0010 Phase B (AC-9, AC-10, AC-11, AC-12, AC-14 — the AST/parser/bytecode deletions plus the Rust-test source-string sweep). AC-1, AC-2, AC-8, and AC-15 already satisfied by ITER-0004b. AC-3 through AC-7 and AC-13 (stdlib greppable invariant) are Phase A (ITER-0004c). ITER-0004d's primary observables are the new parse-rejection scenarios SCENARIO-0104/0105 plus a `fmpl-core/src/`-greppable-invariant scenario SCENARIO-0106 (Rust-side `Expr::Tagged`/`Pattern::Constructor`/`Pattern::TagMatch` absence).
**Rationale:** With the stdlib in canonical list-pattern syntax (ITER-0004c), the parser can stop accepting `:Tag(args)` value-constructor and pattern syntax. This iteration deletes the surviving AST/parser/bytecode surfaces and sweeps the Rust test corpus that still feeds `:Tag(args)` strings into `eval()`. After this iteration there is genuinely one shape and one syntax — no silent fallback path.
**Status:** pending
**Impacted scenarios:** SCENARIO-0104 NEW (parse-rejection of `:Tag(args)` value-construction), SCENARIO-0105 NEW (parse-rejection of `:Tag(p)` pattern syntax), SCENARIO-0106 NEW (greppable-invariant: stdlib + `fmpl-core/src/` clean). SCENARIO-0039 must be rewritten to list-pattern syntax (or owning stories deferred). SCENARIO-0066 hygiene update — references `Value::Tagged` which no longer exists. SCENARIO-0003 reconfirms with the post-burn parser.
**Depends on:** ITER-0004c.
**Look-ahead check:** Unblocks ITER-0006 (self-compile seed) — the seed compiles `fmpl_parser.fmpl + ast_to_ir.fmpl + ast_optimizer.fmpl` through a pipeline that has exactly one AST shape with no parser ambiguity, so the seed is reproducible.

**Scope:**

1. **Sweep FMPL source strings inside Rust test files** to list-pattern syntax. Targets identified by PAR review (~50 files); enumerate exhaustively with `grep -rn ':[A-Z][a-zA-Z_]*(' fmpl-core/tests/`. Hot files: `tests/parser_equivalence.rs:82-85`, `tests/tagged_values.rs:8,32,45,58,70,82,96`, `tests/tagged_pattern_match.rs:20-100`, `tests/fmpl_interpreter.rs:36-69`, `tests/ast_to_ir_parity.rs:88-122`. Use sed/ast-grep where mechanical; hand-edit otherwise.
2. **Update internal grammar production at `fmpl-core/src/grammar/parser.rs:2072`** — current `expr = :Tagged(tag, expr*:args) => :MakeTagged(tag, args)`. Either rewrite to list-pattern syntax with `:MakeListNode` opcode, or delete entirely if the production is dead post-migration. Decide and document.
3. **Make the rename-vs-delete decision for tagged bytecode.** Two paths for AC-11: (a) DELETE — remove `Instruction::MakeTagged`, `MatchTag`, `ExtractTaggedChild`, `MatchTagged`, `MatchTaggedWithBindings` entirely; or (b) RENAME — rename surviving instructions to `MakeListNode`, `MatchListNode`, etc. to reflect list-shape semantics. Recommendation: DELETE if no remaining IR pattern emits these (which after ITER-0004c should be the case); RENAME only if the IR pattern landscape requires preserving an opcode. Make the decision at iteration start and document it as a binding precondition.
4. **Delete `Expr::Tagged`** AST variant. Update `fmpl-core/src/value_to_ast.rs:358` (constructor arm), `fmpl-core/src/builtins/ir_to_rust.rs:1440,1873`, `fmpl-core/src/repr.rs:101`, and any other producer/consumer site enumerated by `grep -rn 'Expr::Tagged' fmpl-core/src/` at iteration start. PAR review estimate: ~25 deletion sites total across `parser.rs`, `grammar/parser.rs`, `compiler.rs`, `value_to_ast.rs`, `builtins/ir_to_rust.rs`, `repr.rs`, `pattern/mod.rs`, `grammar/runtime.rs`, `grammar/trampoline.rs`, `grammar/optimizer.rs`, `builtins/grammar_to_ir.rs`, `builtins/ast.rs`. Re-grep at iteration start to enumerate authoritatively.
5. **Delete `Pattern::Constructor`** and `Pattern::TagMatch` runtime/trampoline handlers. Update `fmpl-core/src/value_to_ast.rs:1241`, `fmpl-core/src/repr.rs:225`, `fmpl-core/src/pattern/mod.rs`, `fmpl-core/src/grammar/runtime.rs:794`, `fmpl-core/src/grammar/trampoline.rs`, etc. Re-grep at iteration start.
6. **Delete the parser productions** for `:Tag(args)` value-construction expressions and `:Tag(p1, p2)` patterns at `fmpl-core/src/grammar/parser.rs::parse_value_pattern` and the corresponding expression production. Bare `:foo` symbol literals (`Expr::Symbol`) remain — only the parenthesized-arguments form is deleted.
7. **Update `lib/core/ast_to_ir.fmpl:21`** rule `[:Tagged, any:tag, exprs:xs] => [:MakeTagged, tag, xs]` becomes dead after AC-9 (no `Expr::Tagged` produced) and AC-11 (no `MakeTagged` instruction). Either delete the rule or rewrite to whatever the rename decision in scope item 3 demands.
8. **Update `lib/core/fmpl_parser.fmpl`** lines 82-83 and 287-292 (`=> :Tagged(tag, items)`, `=> :PatternTagged(tag, pats)`) — these RHS expressions emit AST node shapes that downstream `value_to_ast.rs` decodes. After ITER-0004c the syntax is `[:Tagged, tag, items]` / `[:PatternTagged, tag, pats]`, but the underlying AST shape is the same `Tagged`/`PatternTagged` node. After the AC-9 deletion of `Expr::Tagged`, either the FMPL parser stops emitting `Tagged`/`PatternTagged` AST nodes, OR the decoder keeps handling them. Decide which and update both ends together to avoid an asymmetric coherence gap.
9. **Reconcile scenarios:** rewrite SCENARIO-0039 to list-pattern syntax (or defer the owning stories STORY-0057/0054/0053 if SCENARIO-0039 is no longer authoritative). Update SCENARIO-0066 to reflect post-burn `Value` shape. Add SCENARIO-0104, SCENARIO-0105, SCENARIO-0106.
10. **Verification:**
    - `grep -rnE 'Expr::Tagged|Pattern::Constructor|Pattern::TagMatch' fmpl-core/src/` returns no matches (AC-9, AC-10, AC-12).
    - `grep -rnE ':[A-Z][a-zA-Z_]*\(' fmpl-core/tests/` returns no matches in FMPL source string positions (AC-14 — Rust-test source-string sweep).
    - AC-13 invariant established by ITER-0004c remains satisfied: `grep -cE ':[A-Z][a-zA-Z_]*\(' lib/core/*.fmpl` returns 0 (sanity check; not new work for ITER-0004d).
    - SCENARIO-0104, SCENARIO-0105 fail-fast on `:Tag(args)` input.
    - SCENARIO-0106 (new): `grep -rnE 'Expr::Tagged|Pattern::Constructor|Pattern::TagMatch' fmpl-core/src/` returns no matches.
    - Full workspace test suite passes.
    - SCENARIO-0103 still passes.

**Out of scope:** Removing `FMPL_USE_FMPL_COMPILER` opt-in. Cleanup of any dead-tagged residue inside the bootstrap parser that is gated only by tooling around it.

### ITER-0005 — Image Persistence (Consolidated)

**Stories:** STORY-0099, STORY-0100, STORY-0013, STORY-0014, STORY-0015, STORY-0019, STORY-0021, STORY-0069, STORY-0016, STORY-0017, STORY-0018, STORY-0020
**Rationale:** Consolidated from old ITER-0007/0008/0009. Persist all compiler state to Fjall in one iteration: ObjectDb (objects already derive Serialize/Deserialize), compiled bytecode (rkyv support exists), grammar definitions and memo tables (hardest — semantic actions contain AST expressions), and full VM snapshot/restore. Enable fjall-persistence feature flag. Verify full image survives process restart.
**Status:** pending
**Impacted scenarios:** SCENARIO-0007, SCENARIO-0008, SCENARIO-0009, SCENARIO-0010, SCENARIO-0011, SCENARIO-0099, SCENARIO-0100, SCENARIO-0101, SCENARIO-0102
**Depends on:** ITER-0004b (single canonical representation — see ITER-0004b "Why before persistence")
**Look-ahead check:** Unblocks self-compile seed creation (ITER-0006).

**Build order within iteration (STORY-0099 and STORY-0100 are foundational):**
1. **STORY-0099 first** — versioned envelope is the schema all other persistence callers will write through. Land it before any single payload writer is plumbed in, so no caller is ever rewritten away from raw `serde_json::to_vec`.
2. **STORY-0100 second** — content-addressed source store. The envelope (STORY-0099) carries a `source_hash` field; nothing populates it until this story lands. Constructor synthesis for sourceless artifacts (objects, anonymous lambdas, runtime grammars) is the hard part — implement and test in isolation before the per-payload stories depend on it.
3. **STORY-0013/0014/0015/0019/0021** — per-payload writers (objects, bytecode, grammars, memo tables) all built on the STORY-0099 envelope and STORY-0100 source store. None should bypass the envelope.
4. **STORY-0016/0017/0018/0020** — VM snapshot, full-image roundtrip, normal-startup loading. Depend on the per-payload writers above.
5. **STORY-0069** — feature flag wiring; ship last so the default-disabled path is well-defined.

### ITER-0006 — Self-Compile and Seed

**Stories:** STORY-0024, STORY-0025, STORY-0027, STORY-0028
**Rationale:** Create seed snapshot from current Rust compiler (Stage 0). Add --snapshot and --from-seed flags to fmpl-bootstrap. Write fmpl_compiler.fmpl — the FMPL compiler driver that orchestrates the full pipeline (fmpl_parser.fmpl → ast_to_ir.fmpl → ast_optimizer.fmpl → ir::compile). Verify round-trip: snapshot → restore → compile "1 + 2" → get 3.
**Status:** pending
**Impacted scenarios:** SCENARIO-0020
**Depends on:** ITER-0004 (compiler cutover), ITER-0004b + ITER-0004c + ITER-0004d (full canonical-representation refactor — runtime burn + FMPL stdlib migration + parser/AST burn), and ITER-0005 (persistence). The fmpl_compiler.fmpl pipeline `fmpl_parser.fmpl → ast_to_ir.fmpl → ast_optimizer.fmpl → ir::compile` requires that *every* stdlib file in that chain be in the canonical list-pattern syntax (delivered by ITER-0004c) AND that the parser accepts only one AST shape (delivered by ITER-0004d).
**Look-ahead check:** Unblocks fixpoint verification.

### ITER-0007 — Fixpoint Verification

**Stories:** STORY-0022, STORY-0023, STORY-0026
**Rationale:** The capstone. Stage 0: Rust compiler compiles FMPL compiler pipeline into bytecode seed. Stage 1: load seed, feed FMPL compiler source to itself, produce new bytecode. Verify fixpoint: Stage 1 output == Stage 0 output (byte-identical or semantically equivalent). Verify cold bootstrap from seed produces a working compiler. After this, the bootstrap is stable.
**Status:** pending
**Impacted scenarios:** SCENARIO-0021
**Depends on:** ITER-0006
**Look-ahead check:** After this, self-hosting is achieved.

---

## Deferred

### Parser Cutover Completion (was ITER-0012)

**Stories:** STORY-0001, STORY-0002, STORY-0003, STORY-0004, STORY-0038, STORY-0089
**Rationale:** Phase 1 parser cutover is functionally complete and verified by the 900+ test suite. Formal evidence gathering for AST bridge, parse_with_grammar path, and legacy parser retirement is nice-to-have but not on the critical path.
**Status:** deferred

### Grammar/VM Verification (was ITER-0005/0006)

**Stories:** STORY-0050, STORY-0053, STORY-0057, STORY-0066, STORY-0062, STORY-0070, STORY-0071, STORY-0076, STORY-0077, STORY-0082, STORY-0085, STORY-0086
**Rationale:** Absorbed by ITER-0001 through ITER-0003. The 55/55 parity tests provide stronger evidence that the grammar engine and VM work correctly than the planned formalization stories. These were evidence-gathering, not implementation.
**Status:** absorbed

### Grammar Advanced Features (was ITER-0013)

**Stories:** STORY-0051, STORY-0052, STORY-0054, STORY-0055, STORY-0058, STORY-0059, STORY-0060
**Rationale:** Grammar inheritance, anonymous grammars, PEG combinators, backtracking, memoization, trampolining. Not on the bootstrap critical path. Pursue if stability issues arise.
**Status:** deferred

### VM Advanced Features (was ITER-0014)

**Stories:** STORY-0073, STORY-0074, STORY-0075, STORY-0078, STORY-0079, STORY-0080, STORY-0083, STORY-0084, STORY-0087, STORY-0088
**Rationale:** Pipe operator, name resolution, scoping, nested compiled bodies, object properties, async. Not on the bootstrap critical path. Pursue if needed during self-compile.
**Status:** deferred

### Supporting Infrastructure (was ITER-0015)

**Stories:** STORY-0056, STORY-0061, STORY-0063, STORY-0064, STORY-0065, STORY-0067, STORY-0068, STORY-0081, STORY-0090, STORY-0091, STORY-0092, STORY-0093, STORY-0094, STORY-0095, STORY-0096, STORY-0097, STORY-0098
**Status:** pruned

### MLIR Backend (was ITER-FUTURE)

**Stories:** STORY-0029 through STORY-0042
**Rationale:** Post-self-hosting initiative. Not in scope for bootstrap stabilization.
**Status:** deferred
