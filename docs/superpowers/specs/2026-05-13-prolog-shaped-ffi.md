# Prolog-shaped FFI surface — observations and path

**Date:** 2026-05-13
**Status:** observations + design path. No code changes; no iteration commitments. Inputs to a future iteration that reframes FMPL's FFI surface, and a check on whether ITER-0005a.2+'s persistence work is wrapping the right substrate.
**Author:** Norman + Claude (joint study session during ITER-0005a.2 T0 implementation)

## Why this doc exists

During ITER-0005a.2 T0 (envelope `write` helper), Norman shared a sketch of a Self-inspired language with `make_identity`/`make_relation`/`assert`/method-with-roles primitives, followed by:

> "Note that we had a prolog/datalog shape to grammars as well as the interpretation — I had set up things initially to allow infinite backtracking on streams. See send_more_money.fmpl. The biggest thing is that those top level global functions could also be part of the FFI bridge into rust/host land."

That comment connected three pieces this doc captures together:

1. **FMPL's grammar engine is already a Prolog-style backtracking evaluator** — `PegRuntime` has backtrack/choice-point/get_all_alternatives APIs, shared memoization across branches, and full CSP solving demonstrated in `send_more_money.fmpl`. The Prolog/datalog substrate exists; it's just not surfaced as the canonical FMPL value/computation shape.
2. **The sketched language's primitives are also an FFI shape.** `make_identity`, `make_relation`, `assert`, method-with-roles, `In($coin, ?container)` queries — every one of those is the kind of thing that maps cleanly onto the host's responsibility surface. The host implements specific relations; FMPL code asserts/queries against them. Backtracking-on-streams becomes a native concept.
3. **FMPL's current FFI surface is a hardcoded `(object, method)` match table.** `vm.rs:3415` is `fn call_builtin(&mut self, object: &str, method: &str, args: Vec<Value>) -> Result<Value>` followed by a giant `match (object, method) { (..., ...) => ..., ... }`. Each builtin is bespoke. Adding a new host capability means writing a new match arm.

Putting these together: **the long-term FFI shape could become Prolog-style relations + assertions, with the host implementing specific relations.** That's substantially cleaner than the current bespoke-per-builtin pattern AND it composes with existing backtracking infrastructure.

## Concrete state of the substrate

### What exists today

**Backtracking infrastructure** (`fmpl-core/src/grammar/runtime.rs`):

- `PegRuntime::backtrack(&mut self) -> Option<(Value, usize)>` — line 1751.
- `PegRuntime::get_all_alternatives(&self) -> Vec<(Value, usize)>` — line 1859.
- Choice-point stack, shared memoization across branches.

**Backtracking test coverage** (all passing 2026-05-13):

- `fmpl-core/tests/backtracking.rs` — 4 tests covering multiple-alternative dispatch, named-grammar sinks, anonymous-grammar sinks, apply-syntax variants.
- `fmpl-core/tests/guard_backtracking.rs` — 11 tests covering guard-driven backtracking, sequence guards, CSP generate-and-constrain, early pruning, in-order exploration, multi-constraint propagation.
- `fmpl-core/tests/send_more_money_fmpl.rs` — 4 tests covering full SEND+MORE=MONEY CSP solving end-to-end (known-solution, rejects-invalid, rejects-duplicates, timing).

**19 backtracking tests total**, all passing as of 2026-05-13. The substrate is load-bearing, not aspirational. SEND+MORE=MONEY works.

**Tuple-space module** (`fmpl-core/src/tuplespace/`) — already part of the workspace; provides the multi-arity tuple storage that the sketched language's `make_relation(:Delegates, 3)` + `assert Delegates(...)` would compose with directly.

**Value/tag model** (`Value::List([Value::Symbol(tag), ...children])` per DESIGN-002) — the sketched language's `:player` / `:thing` / `:alice` symbols are FMPL's exact canonical form. `assert Delegates($portable, $thing, 0)` is a tuple-of-FMPL-values keyed by a `:Tag` symbol. The grammar overlap is exact.

**FFI surface today** (`fmpl-core/src/vm.rs:3415`):

```rust
fn call_builtin(&mut self, object: &str, method: &str, args: Vec<Value>) -> Result<Value> {
    match (object, method) {
        ("__builtin_curl", "get") => { ... }
        ("__builtin_curl", "post") => { ... }
        ("__builtin_human", "approve") => { ... }
        ("__builtin_io", "load") => { ... }
        ("__builtin_env", "get") => { ... }
        ("__builtin_sse", "parse") => { ... }
        ("__builtin_time", "sleep") => { ... }
        ("__builtin_rand", "int") => { ... }
        ("__builtin_rand", "float") => { ... }
        // ... many more arms
    }
}
```

Each host capability gets a hardcoded match arm + a per-arm argument-pattern-match. Adding a new capability means editing this function. No introspection on what host capabilities exist; no uniform error surface; no way for FMPL code to query "is this capability available?"

### What the sketch implies as the alternative

```fmpl
make_identity(:player)            # mints a unique identity symbol
make_identity(:thing)

make_relation(:Delegates, 3)      # declares an arity-3 relation
make_relation(:HeldBy, 2)
make_relation(:Portable, 1)

assert Delegates($portable, $thing, 0)
assert Portable($coin)

method $get_thing :get            # method dispatched via roles
  roles actor: $player, item: $thing
do
  if Portable(item)               # query: does this fact hold?
    assert HeldBy(actor, item)    # state change: add a fact
    return true
  else
    return false
  end
end

:get(item: $coin, actor: $alice)  # invocation by symbol-named role

return In($coin, ?container)      # query with logic variable;
                                  # returns 0+ bindings via backtracking
```

In this model:

- **Identities** (`$player`, `$coin`) are FMPL symbols; mint via `make_identity(:player)`. Equivalent to `Value::Symbol`.
- **Relations** are partitions in a (potentially-persistent) tuple space, declared by arity. Each `assert R(a, b, c)` adds a tuple `[:R, a, b, c]` to its partition. Each query `R(a, b, ?c)` walks tuples + backtracks on alternative bindings for `?c`.
- **Methods** dispatch on the delegation chain (`:alice` `Delegates` `:player`; a method whose `roles` includes `actor: $player` is callable with `actor: $alice`). This is Self-style prototype inheritance via explicit `Delegates` facts.
- **Top-level functions** like `make_identity`, `make_relation`, `assert` are the FFI bridge. The host implements them by manipulating the tuple space. Adding host capabilities means asserting new relations into the space; the FMPL surface doesn't grow new builtin match arms.

## How this connects to existing work

### Grammars are already Prolog-shaped

`send_more_money.fmpl`'s own commentary (lines 30-34) documents this:

> "The current FMPL implementation provides:
> - Backtracking stack in PegRuntime (Rust level)
> - Choice pattern creates choice points when multiple alternatives match
> - Shared memoization across all branches
> - backtrack() and get_all_alternatives() for exploration"

The grammar engine IS the Prolog/datalog substrate. Choice points in alternative grammar rules are the same machinery as choice points in alternative bindings to a logic variable. Memoization across grammar rule applications is the same machinery as memoization across query bindings. The 19 backtracking tests prove the substrate works correctly.

### The tuple space is already there

`fmpl-core/src/tuplespace/` exists. The relation-storage primitive is in the workspace. What's missing isn't capability — it's a **canonical FFI surface** that exposes the capability uniformly.

### The persistence work in ITER-0005a.2+ is at the right layer

Important: the persistence envelope (ITER-0005a.1) and the writer sweep (ITER-0005a.2) operate at the **byte-level on-disk format** — they don't commit to any particular logical value model. The envelope wraps `Object`, `CompiledCode`, `Grammar`, `ParseState`, `MemoTable` payloads today; it would wrap `[:Delegates, $portable, $thing, 0]` tuples under a tuple-space FFI just as cleanly. The PayloadKind taxonomy would gain new variants (`TupleSpaceRecord`, `RelationDef`, `IdentityMint`), not get replaced.

So the persistence work isn't wrapping the wrong substrate. It's wrapping the byte-format layer of any substrate.

However: **most current writer call sites would lose relevance** under a tuple-space FFI. `ObjectDb::save_to_fjall` would be subsumed by "the object tuple space partition persists by default." `ParseState::save_to_fjall` likewise. `MemoFjall` is already tuple-shaped. Only `CompiledCode::save_to_fjall` is the odd one out — bytecode isn't naturally tuple-shaped; it'd persist as a `[:Bytecode, instructions...]` list-node per DESIGN-002, or stay outside the tuple-space model as a "compiled artifact" payload class.

ITER-0005a.2 still ships valuable infrastructure (the helper, the sweep gate, the version-aware envelope). It's transitional in the sense that **the specific call sites it sweeps may not survive a future FFI reframe**, but the architectural layer it operates at is durable.

## Path toward the tuple-space FFI shape

This is a long path; no single iteration ships it. Concrete phases below, each independently iteration-shaped.

### Phase 0 — Capture this analysis (done by writing this doc)

The observations above are durable. They don't require iteration scheduling.

### Phase 1 — Expose existing backtracking via FMPL-surface builtins

Per `send_more_money.fmpl`'s own follow-up note (lines 36-39):

> "To expose this to FMPL code, we'd need:
> 1. Builtins to access backtracking API from FMPL
> 2. Stream-based iteration over alternatives
> 3. Constraint predicates in semantic actions"

This is the smallest standalone step. The `PegRuntime::backtrack` and `PegRuntime::get_all_alternatives` are Rust-side; expose them as `__builtin_backtrack` + `__builtin_alternatives` (or similar) in `vm.rs::call_builtin`. Then FMPL code can drive backtracking explicitly without writing a grammar.

**Scope:** ~3-5 tasks (builtin registration + Stream<Alternative> integration + 1-2 example tests). Single concern. Iteration-sized. No FFI reframe yet; just lifts existing Rust APIs to the FMPL surface.

### Phase 2 — Tuple-space primitives at the FMPL surface

Add `make_relation`, `assert`, query-with-logic-variable as FMPL builtins routed to `tuplespace/`. Identity-mint via `make_identity` can ride on existing `Value::Symbol`. The query primitive returns a stream of binding maps; iteration over alternatives uses Phase 1's backtracking-stream infrastructure.

**Scope:** ~6-8 tasks. Iteration-sized but bigger than Phase 1. Touches `tuplespace/`, `vm.rs::call_builtin`, the Stream module. Adds a new behavior-corpus scenario covering "assert + query returns expected bindings" and "query with no matches yields empty stream."

**Persistence interaction:** at this phase, tuple-space partitions could be made persistent by routing through ITER-0005a.2's envelope helper (the helper is payload-class-agnostic — it wraps any `serde::Serialize` value with a `PayloadKind` discriminator). PayloadKind taxonomy would gain `RelationTuple` and `IdentityRegistration` variants. The persistence work doesn't conflict; it's the storage layer for the new FMPL-surface primitives.

### Phase 3 — Method dispatch via roles + Delegates

The Self-style piece: methods declared with `roles actor: $player, item: $thing` are dispatched by walking the `Delegates` relation chain. `:get(item: $coin, actor: $alice)` resolves because `$alice Delegates $player` and `$coin Delegates $portable Delegates $thing`.

**Scope:** ~5-7 tasks. Adds method-table-as-relation indexing; adds the `Delegates`-chain walker; adds dispatch failure semantics ("no method applicable to these roles"). Adds scenarios for unambiguous dispatch, ambiguous dispatch (which alternative wins?), and dispatch failure.

**This is the phase where the FFI surface starts to feel categorically different.** The host registers methods by asserting tuples into the method-table relation. No more `vm.rs::call_builtin` match arms — host capabilities are facts in the tuple space.

### Phase 4 — Migrate existing builtins to the new shape

Move `__builtin_curl::get`, `__builtin_io::load`, etc. from `vm.rs::call_builtin` match arms into method-as-relation registrations. The host registers each builtin as a method during VM startup; FMPL code calls them via the regular method-dispatch path.

**Scope:** large. Each builtin (currently ~10-15 in `vm.rs`) needs migration + per-builtin scenario. Should split into smaller iterations along functional categories (network builtins, IO builtins, etc.) — same writer/reader split discipline the ITER-0005 family worked out.

**At this phase the `vm.rs::call_builtin` match table shrinks to a fallback / startup-registration path.** The FFI bridge IS the tuple space.

### Phase 5 — Persistence convergence

If tuple-space partitions are the canonical persistence path (per Phase 2's adoption decision), then most explicit `save_to_fjall`/`load_from_fjall` callers become redundant — the partitions persist by default. ITER-0005d's per-payload writers (objects, grammars, memo tables) get subsumed. ITER-0005a.2's writer sweep is recorded as transitional infrastructure that prepared the envelope layer for any payload class.

**At this point CompiledCode is the lone holdout** — bytecode isn't naturally tuple-shaped. Either keep it as a special payload class wrapping `[:Bytecode, instructions...]` in the envelope, or accept that the FFI reframe doesn't subsume every persistence concern.

## Risks and counter-considerations

**Risk 1: Phase 4 is large and easy to over-scope.** The current FFI has ~10-15 builtins; each migration is a small surface but the aggregate is multi-iteration work. The recently-validated discipline (`feedback_split_iterations_on_reader_writer_asymmetry.md`) applies — split along category lines.

**Risk 2: Backtracking semantics need to be stable across the FFI boundary.** Today the backtracking machinery lives in `PegRuntime`; for the FFI reframe it needs to lift up so that host calls participate in the choice-point stack. A host call that returns "no, this fact doesn't hold" should be a backtrack-point, not a final error. This is non-trivial — the host's `Result<Value>` return type needs to grow a "this is a backtrack point" variant. Worth a focused design pass before Phase 2.

**Risk 3: The method-dispatch model has resolution-order subtleties.** Self's prototype inheritance has well-known dispatch-ambiguity issues (multiple parents declaring the same method; which wins?). The sketch's `Delegates` chain is a tree, not a DAG today, but real systems grow into DAGs. The dispatch semantics need pinning before Phase 3.

**Risk 4: Tuple-space persistence has cache-freshness questions.** When the tuple space is persistent and the VM mutates it via `assert`/`retract`, when do those changes flush? Per-call? Per-method? On checkpoint? This question composes with the (already-deferred) `invalidation` work from the sibling-projects study (`docs/superpowers/specs/2026-05-12-lessons-from-siblings.md` §3 / §6 deferred).

**Counter-consideration: this might be too much architecture for FMPL's near-term needs.** The current FFI works; the writer sweep ships value; the bootstrap path doesn't strictly need any of this. Phase 1 (expose backtracking to FMPL code) is genuinely standalone-valuable. Phases 2-5 are a substantial multi-iteration commitment that should only be undertaken if the FFI ergonomics + uniform persistence story justify it.

## Concrete recommendations

### For the in-flight ITER-0005 family

**No changes.** The envelope work is at the right architectural layer and survives any FFI reframe. Continue ITER-0005a.2's writer sweep as scoped. The "transitional" framing is real but not paralyzing — every code base ships transitional infrastructure on its way to better designs.

### As a future-iteration candidate

Add **ITER-FFI-PROLOG-PHASE-1** as a candidate iteration on the roadmap (NOT yet sequenced into the main path). Scope: expose `PegRuntime`'s backtracking APIs to FMPL code via builtins. Standalone-valuable; ships its own behavior corpus scenario; no FFI reframe required.

If Phase 1 lands and reveals genuine surface ergonomics + use-case pressure for Phases 2-5, schedule those one-by-one along the same path. If Phase 1 lands and reveals the backtracking-from-FMPL surface gets little use, the FFI reframe stays at the design-doc stage.

### As a never-iteration but durable reference

This doc captures the analysis. It joins `2026-05-12-lessons-from-siblings.md` as a design-thinking artifact that informs scope decisions across future iterations even when nothing concrete schedules from it.

## Appendix A — File:line citations

| Pattern | Location |
|---|---|
| `PegRuntime::backtrack` | `fmpl-core/src/grammar/runtime.rs:1751` |
| `PegRuntime::get_all_alternatives` | `fmpl-core/src/grammar/runtime.rs:1859` |
| `call_builtin` (current FFI surface) | `fmpl-core/src/vm.rs:3415` |
| `tuplespace/` module | `fmpl-core/src/tuplespace/` |
| `send_more_money.fmpl` (CSP solver demo) | `send_more_money.fmpl` |
| Backtracking test files | `fmpl-core/tests/{backtracking,guard_backtracking,send_more_money_fmpl}.rs` |
| Backtracking test count | 19 tests across 3 files, all passing 2026-05-13 |

## Appendix B — Decisions deferred to whoever schedules Phase 2+

- Tuple-space persistence model (partition-per-relation vs. monolithic tuple table; flush semantics; transaction boundaries).
- Backtracking across FFI calls (does a host call return a single value or potentially a stream of values; does the choice-point stack span host calls).
- Method-dispatch resolution order under DAG inheritance (which parent wins when multiple `Delegates` paths apply).
- Identity equality (are `:player` from one `make_identity(:player)` call and another `make_identity(:player)` call the same? interned by name? distinct?).
- Logic-variable scoping (does `?container` in a query bind for the rest of the surrounding block, or only for the query expression?).

These are real design choices that need pinning before code lands. None block this doc's recommendation; all block Phase 2+ iterations.
