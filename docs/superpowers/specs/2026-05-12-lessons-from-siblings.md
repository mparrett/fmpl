# Lessons from sibling projects (Cairn, moor-echo, invalidation)

**Date:** 2026-05-12
**Status:** survey + adoption recommendations. No code changes; no commitments. Inputs to ITER-0005 scope refinement.
**Author:** Norman + Claude (joint study session, post-ITER-0004x)

## Why this doc exists

ITER-0004x established a dual-VM parity gate (`fmpl_core::Vm` ↔ `execution_tape::vm::Vm`) on a narrow integer-and-arithmetic subset. That experiment told us cross_compile.rs had latent bugs (now fixed) but did NOT prove that `execution_tape` is the right long-term runtime for FMPL. With ITER-0005 (image persistence) approaching and the question "what's the right architectural shape for FMPL's compiler and runtime?" still open, this doc surveys three sibling projects to extract reusable patterns:

1. **`~/development/cairn`** — a Dylan-shaped semantic kernel that already targets execution_tape. Mature `sem → lower → tape` pipeline with span-threaded IR.
2. **`~/development/moor-echo`** — an Echo/MOO database with an in-memory **SystemTracer** (Squeak-Smalltalk inspired) for transforming live objects under transformation rules.
3. **`~/development/invalidation`** — a `no_std` dependency-graph crate for incremental work. Used by cairn for compile-time cache invalidation.

Each project answers a question FMPL also needs to answer. None of them is "the right target." But each carries a pattern worth adopting.

## What FMPL gets from this study, in one paragraph

- **From cairn:** discipline around span propagation through every IR node (FMPL drops spans today at the AST boundary); a clean `sem → lower → tape` shape (FMPL's compiler is more entangled); the `RuntimeLayout`/`HostSig` separation between data and signatures (FMPL has no host layer yet but will need one).
- **From moor-echo:** the `TransformationRule` trait + tracer-runner pattern. This is the missing piece for ITER-0005's **schema migration** story. It scales from "rewrite one record on disk" to "rewrite the live image" — exactly the spectrum image persistence will need to cover.
- **From invalidation:** the dependency-tracker substrate for incremental compilation and cache freshness. **Not relevant to ITER-0005's main work** (schema versioning), but highly relevant to a downstream iteration where persisted artifacts depend on tracked source files.

The three projects compose. Cairn shows how to lay out the compiler pipeline; moor-echo shows how to author transformations over it; invalidation shows how to track what changed.

---

## 1. Cairn — semantic kernel for execution_tape

### What cairn is

Cairn is a "bounded semantic-kernel/compiler experiment" (per `~/development/cairn/DESIGN.md:6`) — a Dylan-shaped frontend for execution_tape. Classes, slots, generic functions, multiple dispatch, host-backed objects, source spans threaded through the entire pipeline. Milestone-0 covers integers, booleans, classes, and simple object construction.

### What cairn does NOT cover

- No floats, no strings, no lists, no maps as IR types
- No closures or first-class functions (top-level `define function` / `define method` only)
- No pattern matching (dispatches via generics + classes instead)

For FMPL, those four gaps mean cairn-as-IR-target is **not viable**. FMPL's value model is structural-list-shaped (DESIGN-002); cairn's is nominal-class-shaped. Translating between them would impose semantic-translation tax with no payoff.

### What's worth borrowing: the pipeline shape

Cairn's compiler is organized as five well-bounded modules:

| Module | Owns |
|---|---|
| `syntax.rs` | CST/AST + parser. `Span` definition lives here. |
| `sem.rs` | Semantic IR with resolved direct calls, locals, classes, methods, generics. |
| `lower.rs` | Backend-neutral executable IR. `Module → Vec<Function> → Vec<Instr>`. Every `Instr` variant carries a `span: Span` field. |
| `runtime.rs` | `RuntimeLayout`, `CairnHost`, host-call signatures (`HostSig`). Separates "what objects exist" (layout) from "what host calls are available" (signatures). |
| `tape.rs` | `Module → emit() → TapeModule { VerifiedProgram, RuntimeLayout, functions, main }`. Pure bytecode encoding; no semantic work. |

Compare to fmpl-core:

| FMPL module | Owns | Issue |
|---|---|---|
| `parser.rs` | parsing + AST | spans tracked at token boundary, dropped at AST |
| `compiler.rs` | AST → bytecode in one pass | no IR layer; emit interleaves with semantic decisions; `Instruction` enum carries no spans |
| `vm.rs` | bytecode → values | no spans available at runtime traps |
| `cross_compile.rs` | bytecode → execution_tape | adds a second target but doesn't help with span propagation |

**The architectural lesson:** cairn's `lower::Instr` has a span on every variant. `tape.rs` reads it during emit and writes it through `self.set_span(span)` before each bytecode instruction. execution_tape's `SpanId` makes the span available at runtime trap sites. Total cost: one `Span` field per variant (8 bytes on 64-bit), one `set_span` call per emit. Total payoff: every runtime error can point back to the source byte range.

### Concrete pattern: span-on-every-Instr

From `~/development/cairn/crates/cairn_compiler/src/lower.rs:140-345`:

```rust
pub enum Instr {
    ConstI64    { dst: Register, value: i64,  span: Span },
    ConstBool   { dst: Register, value: bool, span: Span },
    I64Binary   { dst: Register, op: sem::BinaryOp, lhs: Register, rhs: Register, span: Span },
    I64Compare  { dst: Register, op: sem::CompareOp, lhs: Register, rhs: Register, span: Span },
    Call        { dst: Register, callee: FunctionId, args: Vec<Register>, span: Span },
    Branch      { cond: Register, then_label: LabelId, else_label: LabelId, span: Span },
    Jump        { label: LabelId, span: Span },
    Return      { value: Register, span: Span },
    Trap        { code: u32, span: Span },
    // ... every variant has `span: Span`
}
```

And the emit side at `~/development/cairn/crates/cairn_compiler/src/tape.rs:274-322`:

```rust
lower::Instr::ConstI64 { dst, value, span } => {
    self.set_span(*span);
    self.asm.const_i64(dst.0, *value);
}
lower::Instr::I64Binary { dst, op, lhs, rhs, span } => {
    self.set_span(*span);
    match op {
        sem::BinaryOp::Add => self.asm.i64_add(dst.0, lhs.0, rhs.0),
        // ...
    }
}
```

The discipline is mechanical. Every emit site has a `set_span(*span)` before the bytecode call. No exceptions.

### Adoption recommendation for FMPL

**Yes, port the pattern. No, don't port through cairn.**

Two FMPL changes would carry the bulk of the benefit:

1. **Add a `Span` (or reuse `SourceLocation`) field to every `Instruction` variant.** Wire it from `compile_expr`'s emit sites. ~50-80 emit sites in `compiler.rs`; mechanical.

2. **Thread spans through `cross_compile.rs` to `execution_tape::SpanId`.** When the dual-VM parity gate evolves to cover more opcodes, the execution_tape side will gain source-attributed traps for free.

A fuller adoption — splitting `compiler.rs` into a `sem`-style semantic IR plus a `lower`-style backend-neutral IR — is a multi-iteration effort that probably belongs alongside the eventual MLIR initiative (STORY-0037). The minimal "spans everywhere" change is a one-iteration win.

### What NOT to adopt

- **Cairn's class model.** Wrong semantic shape for FMPL.
- **Cairn's `RuntimeLayout` definition.** Useful pattern, but the contents (classes, slots, methods) don't map to FMPL's tag-shape model. The **separation discipline** (data vs. signatures) is the lesson; the specific layout is not.
- **Cairn's `Dispatcher` IR variant.** FMPL doesn't have generic-function multiple dispatch.

---

## 2. moor-echo — the SystemTracer pattern

### What moor-echo is

A Rust-hosted Echo/MOO live-database environment with an in-memory `SystemTracer` for transforming live code under user-defined transformation rules. Both a Rust implementation (`crates/echo-core/src/tracer/`) and an in-MOO authorial layer (`bootstrap/system_tracer.moo`). Explicitly inspired by Squeak's SystemTracer — the original in-image format-upgrade mechanism.

### Why this matters for ITER-0005

ITER-0005 needs **schema versioning and updating** for persisted bytecode. Three sub-problems:

1. **Envelope** — `{ vm_version, schema_version, payload }` on disk. Trivially solvable with serde + a version field. STORY-0099 (loader skips incompatible VM versions) addresses this.
2. **Migration** — when the on-disk record is at schema vN and the reader is at vN+M, how does the payload get rewritten? Or how does it fall back to source recompilation? This is the **hard** part. STORY-0100 (content-addressed source reference) + SCENARIO-0102 (recovery via source recompilation) gesture at the problem.
3. **Cache freshness** — when a source file changes, which persisted artifacts go stale? `invalidation` solves this; see §3.

Sub-problem (2) — the migration — is what the SystemTracer pattern is for. Squeak's SystemTracer was literally a format-upgrade mechanism: it walked the live image, applied transformation rules, rewrote object layouts in place. moor-echo's Rust port lifts that into an explicit trait + runner.

### The `TransformationRule` pattern

From `~/development/moor-echo/crates/echo-core/src/tracer/rules.rs:14-48`:

```rust
pub trait TransformationRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn priority(&self) -> u32 { 100 }

    fn matches(&self, ast: &EchoAst, context: &TransformationContext) -> bool;
    fn transform(&self, ast: EchoAst, context: &TransformationContext) -> Result<EchoAst>;

    fn validate(&self, original: &EchoAst, transformed: &EchoAst, _ctx: &Context) -> Result<()> {
        Ok(())
    }
    fn conflicts_with(&self, other: &dyn TransformationRule) -> bool { false }
}
```

Plus a runner (`SystemTracer` at `~/development/moor-echo/crates/echo-core/src/tracer/system_tracer.rs:22-25`):

```rust
pub struct SystemTracer {
    rules: Vec<Box<dyn TransformationRule>>,
    stats: HashMap<String, RuleStats>,
    max_iterations: usize,
    dry_run: bool,
}
```

With a `RuleStats` companion that tracks `applications`, `transformations`, `errors`, `total_time_ms` per rule. The runner offers:

- **Priority-sorted dispatch** — rules with higher priority run first.
- **Dry-run mode** — compute the would-be transformation but don't apply it. Useful for diagnostics and for "show me what would change."
- **Per-rule statistics** — observability for which rules are doing work, which are erroring, how long they take.
- **Composite rules** (`CompositeRule` at `rules.rs:51-94`) — chain rules into a pipeline (`v3 → v4 → v5`).
- **Fluent helpers** (`TypedRule` at `rules.rs:95-160`) — define a rule as a `(matcher, transformer)` pair without a new struct.
- **Iteration cap** — `max_iterations` bounds fixed-point convergence so cyclic rule sets don't loop forever.

### Why this fits ITER-0005

The on-disk payload is a serializable shape — `serde_json::Value`, `Vec<u8>`, or a typed `Instruction` enum at a specific version. Each migration rule says:

> "I see a record with `schema_version: 3` matching pattern X. I emit a record with `schema_version: 4` and shape Y."

Composite rules chain `v3 → v4 → v5 → v6` transparently. The runner picks the right starting rule by inspecting the payload's `schema_version` field; the composite chain advances it.

Dry-run mode is invaluable here: when introducing a new schema version, you can run the migration over the on-disk corpus in dry-run mode FIRST, get statistics on how many records would be rewritten, what classes of error would surface, and then decide whether to ship the migration or fix the rules.

Per-rule statistics give you the audit trail: "the v4→v5 migration applied 1247 times, 1247 successful transformations, zero errors, 23ms total."

### The metacircular payoff

The moor-echo SystemTracer is **runnable inside the live image** — written in MOO and acting on live objects. The Rust trait is a host-side mirror so that platform-level rules can live next to user-authored ones.

If/when FMPL gets image persistence + live mutation (likely ITER-0006+), the same tracer pattern can be lifted into FMPL itself. Transformation rules become first-class FMPL programs that the host can register. That's DESIGN-001's metacircular bootstrap goal made concrete: FMPL programs authoring their own runtime migrations.

Adopting the Rust trait for ITER-0005 sets up the eventual FMPL-side lifting cleanly: the Rust trait IS the contract the eventual FMPL trait will satisfy.

### Adoption recommendation for FMPL (ITER-0005)

**Yes, port the pattern explicitly.** Concrete proposal:

```rust
// fmpl-core/src/persistence/migration.rs (NEW)

pub trait MigrationRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn priority(&self) -> u32 { 100 }
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;

    fn matches(&self, record: &PersistedRecord) -> bool;
    fn transform(&self, record: PersistedRecord) -> Result<PersistedRecord, MigrationError>;
    fn validate(&self, before: &PersistedRecord, after: &PersistedRecord) -> Result<(), MigrationError> {
        Ok(())
    }
}

pub struct MigrationEngine {
    rules: Vec<Box<dyn MigrationRule>>,
    stats: HashMap<String, RuleStats>,
    dry_run: bool,
    max_iterations: usize,
}

impl MigrationEngine {
    pub fn migrate(&mut self, record: PersistedRecord) -> Result<PersistedRecord, MigrationError> {
        // Sort rules by (from_version, priority).
        // Iteratively apply matching rules until the record's schema_version
        // reaches CURRENT_SCHEMA_VERSION or no rule matches.
        // Cap at max_iterations.
        // If dry_run, log but don't return the transformed record.
        // Accumulate per-rule stats.
    }
}
```

Initial rule set in ITER-0005: zero rules. Infrastructure-only. The first real rule lands the moment the schema changes (which is exactly when ITER-0005's value lands too: the migration story exists before the first breaking change, not after).

### What NOT to adopt

- **moor-echo's `EchoAst` coupling.** Use FMPL's own payload type.
- **The in-MOO `system_tracer.moo` authorial layer.** That's the eventual ITER-0006+ metacircular lift. Don't try to write it in FMPL yet — io::read_dir is still missing per the existing roadmap.
- **The `Evaluator` parameter on `transform_object`.** moor-echo's tracer needs the live evaluator because it transforms live objects with executable verbs. FMPL's ITER-0005 transforms persisted records, not live objects.

---

## 2.5 The broader tracer-family framing

### Why this section exists

§2 above scoped the moor-echo pattern as "a thing for schema migration" — `TransformationRule` + `MigrationEngine`. That framing was too narrow. The Smalltalk SystemTracer lineage is **a generic object-graph walker with pluggable visitor semantics**, of which migration is one visitor among several. Squeak's original SystemTracer is used for image **shrinking** (reachability traversal from roots) and **serialization** (deterministic deep-copy traversal) as much as for **transformation** (schema migration). moor-echo's port specialized that lineage to the transformation slice; the FMPL adoption decision should not inherit that specialization without reconsidering it.

PAR scope review on ITER-0005a.0 (2026-05-12) caught the narrow framing's downstream cost: shipping `MigrationEngine` ahead of any consumer created premature abstraction with under-specified architectural commitments. The deferral resolved that specific instance. The broader question — "what is the right shape of FMPL's tracer infrastructure?" — has multiple near-term consumers on the roadmap and deserves a sharper framing.

### The tracer-family taxonomy

The Smalltalk tracer family includes (at minimum) the following visitor classes. Each visitor uses the same object-graph traversal substrate; the substrate is shared infrastructure, the visitors are consumer-specific:

| Visitor | Use case | FMPL near-term consumer |
|---|---|---|
| **ReachabilityVisitor** | Image shrinking — given roots, mark reachable objects; everything else is dead | **ITER-0006 seed-snapshot creation** (Stage-0 → Stage-1 bootstrap needs to walk from `main` and persist only what's reachable) |
| **SerializationVisitor** | Deterministic deep-copy traversal — write each object exactly once, in a reproducible order | **ITER-0005e Vm::snapshot(dir)** (snapshot reproducibility requires deterministic traversal) |
| **MigrationVisitor** | Schema upgrade — rewrite payloads from vN to vN+1 (this is moor-echo's `TransformationRule` slice) | **First real schema change** (deferred; no near-term consumer) |
| **QueryVisitor** | Find-all-instances-where — given a predicate, enumerate matching objects | Future (no roadmap consumer today) |
| **RewriteVisitor** | Atomic `become:` — swap an object's identity in place across all live references | Future (no roadmap consumer today) |

The substrate that all of these share: a stack-machine or worklist that walks the live object graph, tracking visited nodes, dispatching to the visitor at each node, and respecting cycles. moor-echo's `SystemTracer` is one specific composition of substrate + MigrationVisitor; Squeak's original SystemTracer was substrate + ReachabilityVisitor + SerializationVisitor.

### The right design discipline

Per `feedback_ship_infrastructure_with_first_consumer.md` (the lesson from the ITER-0005a.0 deferral), infrastructure ships with its first consumer. The right granularity for that discipline here is:

- **The tracer substrate** is shared infrastructure. It earns its keep across multiple iterations.
- **Each visitor** is consumer-specific. It ships with its consumer.

So the discipline becomes: the substrate ships in **the first iteration that has a real visitor consumer**, alongside that first visitor. Not in a "foundation iteration" speculative-of-future-visitors. The substrate's design is pinned by its first real visitor's needs — not by speculation about MigrationVisitor's payload-type or QueryVisitor's predicate-protocol.

### What this means for the roadmap

The deferral of ITER-0005a.0 still stands. The `MigrationEngine` specifically should ship with the **first** `MigrationVisitor` (first real schema change to a persisted payload class). But the **tracer substrate** is a different question, and it has near-term consumers already on the roadmap:

- **ITER-0005e (Vm::snapshot)** needs a `SerializationVisitor`. Snapshot determinism + reproducibility require a deterministic traversal order, which is exactly what the substrate provides.
- **ITER-0006 (seed-snapshot)** needs a `ReachabilityVisitor`. Stage-0 → Stage-1 bootstrap walks from `main` and persists only the reachable subgraph.

If ITER-0005e is the first iteration that lands the substrate, it does so alongside its `SerializationVisitor`. That's the discipline correctly applied: the substrate is justified by ITER-0005e's snapshot determinism requirement, and the `SerializationVisitor` is the consumer that pins its design. ITER-0006 then inherits the substrate and adds a `ReachabilityVisitor` — no new substrate work, just a new visitor.

When the first real schema change arrives, it adds a `MigrationVisitor` on the same substrate. The `MigrationEngine` doesn't ship as a separate abstraction — it's "the substrate + MigrationVisitor wired together," and the rule registry is just the visitor's configuration.

### Concrete recommendation for ITER-0005e

At ITER-0005e's iteration entry: rather than hand-rolling `Vm::snapshot`'s traversal, design the snapshot as the first visitor on a generic tracer substrate. The build order becomes:

1. Tracer substrate (worklist-based graph walker with cycle handling + visitor dispatch).
2. `SerializationVisitor` implementation.
3. `Vm::snapshot(dir)` / `Vm::restore(dir)` wired through the substrate + serialization visitor.
4. Normal-startup loading (uses the restore path).
5. Full-image journey roundtrip test.

This earns the substrate's keep immediately, sets up ITER-0006's reachability work, and creates the right foundation for the eventual `MigrationVisitor` to land cleanly when its time comes. PAR scope review on ITER-0005e should specifically probe whether the substrate's design (worklist API, cycle-tracking representation, visitor-dispatch shape) is generic enough to admit `ReachabilityVisitor` and `MigrationVisitor` later without rework — and conservative enough that it isn't carrying speculative features for those future visitors.

### What NOT to do

- **Don't ship the substrate ahead of ITER-0005e.** That would repeat the ITER-0005a.0 mistake at a coarser level — substrate without a visitor is the same antipattern as engine without a rule.
- **Don't design the substrate by speculating across all 5 visitor classes.** Pin it to `SerializationVisitor`'s needs first; widen when `ReachabilityVisitor` actually demands it. Each consumer's iteration is allowed to extend the substrate.
- **Don't repackage moor-echo's `SystemTracer` wholesale as "the FMPL tracer."** That's importing the MigrationVisitor specialization with the substrate underneath it. The substrate is what's reusable; the migration specialization is one visitor.

### Open questions

- **Substrate API surface.** Worklist-based? Recursive? CPS? The choice has consequences for stack depth on deep graphs and for visitor-suspension semantics (do visitors yield mid-traversal?). Pin in ITER-0005e.
- **Cycle representation.** A `HashSet<ObjectId>` is the obvious choice but doesn't help with `become:` semantics (RewriteVisitor) where identity is mutable. If `become:` ever becomes a near-term consumer, revisit.
- **Visitor dispatch model.** Trait-object polymorphism vs. enum-of-visitors. The proof-tests preference suggests an enum (exhaustive `match` is a typed invariant); trait-object is more open but loses compile-time exhaustiveness. ITER-0005e's `SerializationVisitor` is the only visitor; trait-object is fine for one consumer, but if a future visitor lands the choice should be revisited.

---

## 3. invalidation — the dependency-graph substrate

### What invalidation is

A `no_std` Rust crate for "generic dependency-aware invalidation" — given a graph of "A depends on B," when B changes, find and process A in topological order. The public surface (per `~/development/invalidation/README.md`):

```rust
use invalidation::{Channel, EagerPolicy, InvalidationTracker};

const LAYOUT: Channel = Channel::new(0);

let mut tracker = InvalidationTracker::<u32>::new();
tracker.add_dependency(2, 1, LAYOUT).unwrap();  // 2 depends on 1
tracker.add_dependency(3, 2, LAYOUT).unwrap();  // 3 depends on 2

tracker.mark_with(1, LAYOUT, &EagerPolicy);     // 1 changed

let ordered: Vec<_> = tracker.drain_sorted(LAYOUT).collect();
assert_eq!(ordered, vec![1, 2, 3]);
```

Multi-channel support (separate dependency graphs for separate concerns), eager vs. lazy propagation policies, channel cascades, cross-channel edges, deterministic tie-breaking. Salsa-style minus the macro layer.

### Why this is NOT what ITER-0005 needs

The persistence story is *not* a dependency-graph problem. Schema migration is **payload rewriting**, not **dependency cascading**. Loading a persisted record with `schema_version: 3` doesn't require resolving "what other records depend on this." It requires running a `v3 → v_current` transformation chain.

ITER-0005's three sub-problems (envelope, migration, cache freshness) map cleanly:

| Sub-problem | Pattern | Project to borrow from |
|---|---|---|
| Envelope (record version, content-addressed source) | serde + version fields | n/a (standard practice) |
| Migration (`v3 → v4` rewrites) | `TransformationRule` + runner | **moor-echo** |
| Cache freshness (source changed → which artifacts stale?) | Dependency graph + drain | **invalidation** |

The third is real and important — but it's downstream of ITER-0005, not inside it. Cairn uses `invalidation` for **compile-time** incrementality (parse depends on source-file; bindings depend on parse; semantic depends on bindings; etc.). FMPL's compile pipeline could benefit from the same once it grows beyond "one program at a time." Persisted-artifact cache invalidation is the same problem at the persistence layer.

### Adoption recommendation for FMPL

**Defer.** ITER-0005 (envelope + migration engine) doesn't need this. A future iteration — call it ITER-0005c or ITER-0006-prelude — should:

1. Define the dependency channels FMPL cares about (likely: `Source`, `Parse`, `Bytecode`, `PersistedArtifact`).
2. Wire `mtime` or content-hash watching at the source-file boundary.
3. Use `InvalidationTracker` to mark+drain stale artifacts.

That iteration is a clean, well-bounded port. But it's premature now: without persisted artifacts, there's nothing to invalidate.

### What NOT to adopt prematurely

- **`InvalidationTracker` for in-compiler cache.** Wait until the compiler genuinely has cache. Today it doesn't (every `cargo test` recompiles from scratch).
- **Channels for sub-pipeline tracking.** Same reasoning. The complexity earns its keep at the persisted-artifact layer, not before.

---

## 4. Comparison: what each project tells FMPL about the path forward

The three projects cluster around different design dimensions. Putting them side-by-side:

| Dimension | Cairn | moor-echo | invalidation | FMPL today |
|---|---|---|---|---|
| **IR shape** | `sem → lower → tape`, span-threaded | n/a (operates on AST) | n/a | `parser → compiler` (one-pass; no IR layer) |
| **Spans** | on every `Instr` variant | n/a | n/a | tracked at lexer, dropped at AST |
| **Persistence** | none yet (single program) | live image | n/a | none yet |
| **Migration** | n/a | `TransformationRule` trait + runner | n/a | none |
| **Cache freshness** | `invalidation`-based | n/a | itself | `cargo test` recompiles every time |
| **Host/runtime split** | `RuntimeLayout` / `HostSig` separation | `Evaluator` owns it | n/a | tangled in `vm.rs` |
| **Code organization** | five small modules | three (tracer + rules + patterns) | one focused crate | one big `compiler.rs`, one big `vm.rs` |

### Where FMPL is strong (don't change)

- **Metacircular bootstrap discipline.** None of the three sibling projects has a working metacircular pipeline yet. FMPL's source-tree parser + FMPL-stdlib-generated parser parity (SCENARIO-0108) is genuinely ahead.
- **List-shaped canonical form (DESIGN-002).** Distinctive; load-bearing for FMPL's pattern matching and AST manipulation. Don't trade it for cairn's nominal-class model.
- **Behavior-scenario corpus + dual-VM parity gate.** Both built out via ITER-0004 series and ITER-0004x. Better evidence discipline than the three siblings combined.

### Where FMPL has gaps (study the siblings)

- **Span discipline.** Lost at AST boundary. Adoptable from cairn — touches every `Instruction` variant emit site in `compiler.rs` (count to be measured at iteration entry via `grep -c "self\.code\.emit(" compiler.rs`).
- **IR layer.** Compiler emits bytecode in one pass; no semantic IR to optimize over. Adoptable from cairn but multi-iteration; can defer.
- **Schema migration infrastructure.** No mechanism. Adoptable from moor-echo; one iteration; **the right thing for ITER-0005**.
- **Cache freshness.** No mechanism. Adoptable from invalidation; one iteration; can defer until persisted artifacts exist.
- **Host/runtime split.** Tangled in `vm.rs`. Adoptable from cairn; less urgent than the others.

---

## 5. Concrete recommendations

### For ITER-0005 (immediate)

**Borrow the SystemTracer pattern from moor-echo.** Specifically:

1. Add `fmpl-core/src/persistence/` (new module).
2. Inside it, port the `TransformationRule` trait as `MigrationRule` with FMPL-specific generics (`PersistedRecord` instead of `EchoAst`).
3. Port the runner as `MigrationEngine` with the same dry-run / max-iterations / per-rule-stats discipline.
4. Use it to satisfy STORY-0099, STORY-0100, SCENARIO-0102 — the loader path becomes "deserialize envelope → run migration engine → return current-schema payload OR fall back to source recompilation."
5. **Author this in Rust first**, not FMPL. The eventual FMPL-side lift (analogous to moor-echo's MOO authorial layer) is ITER-0006+ territory.

This roughly doubles the size of ITER-0005's previously-imagined scope, but it converts a one-iteration "envelope plus loader" sprint into a one-iteration "migration infrastructure" sprint that has durable value across every future schema change.

### For ITER-0004x.1 or wherever it lands (near-term)

**Adopt cairn's span-on-every-Instr discipline.** Specifically:

1. Add a `Span` (or reuse existing FMPL `SourceLocation`) field to every variant of `Instruction` in `compiler.rs`.
2. Wire it through every emit site in `compile_expr` / `compile_pattern` / etc.
3. Thread it through `cross_compile.rs` to `execution_tape::SpanId` — that side already understands spans.
4. Author a sentinel scenario that runs a deliberately-trapping FMPL program and asserts the runtime trap surfaces the source byte range.

This is a one-iteration port. The payoff is permanent diagnostic quality improvement.

### Deferred — for a later iteration once cache exists

**Adopt `invalidation` for persisted-artifact cache freshness.** When the persistence layer has been in place long enough that re-recompiling becomes painful, add:

1. Source-file content hash → parse-result invalidation channel.
2. Parse → bytecode invalidation channel.
3. Bytecode → persisted-artifact invalidation channel with file-mtime triggers.

Scope: 3 new dependency channels + edge wiring across each persistence call site introduced by 0005c/d. Don't attempt before the persistence layer has real adoption.

### Deferred — for ITER-0006+ (metacircular)

**Lift the migration runner into FMPL itself.** Once io::read_dir exists (deferred from ITER-0004d.4 → ITER-0004d.5) and FMPL can author its own migration rules, the Rust `MigrationRule` trait becomes the contract that FMPL-authored rules satisfy. This is the DESIGN-001 metacircular goal made concrete: FMPL programs authoring their own runtime migrations, registered into the host's migration engine.

---

## 6. Open questions

- **Does moor-echo's tracer support out-of-band schema upgrades (i.e., upgrading the IDLE database while the system is offline)?** Need to check; matters for FMPL's persistence story.
- **Does cairn's `invalidation` integration carry compile-time cache between cargo runs, or only within a single compile?** Likely the latter today; matters for the eventual FMPL adoption.
- **What's execution_tape's `SpanId` lifecycle?** Is it stable across persistence boundaries, or per-program?
- **Should FMPL's `MigrationRule` operate on a typed `Instruction` enum, on `serde_json::Value`, or on raw `Vec<u8>`?** Choice affects how migration rules are authored. Author-friendliness vs. type-safety tradeoff.

These are worth resolving before ITER-0005 lands, but they don't block this study.

---

## Appendix A — Direct citations

| Pattern | File | Lines |
|---|---|---|
| Cairn `Instr` enum (span-everywhere) | `~/development/cairn/crates/cairn_compiler/src/lower.rs` | 140–345 |
| Cairn `tape.rs::set_span` emit pattern | `~/development/cairn/crates/cairn_compiler/src/tape.rs` | 270–322 |
| Cairn `RuntimeLayout`/`HostSig` separation | `~/development/cairn/crates/cairn_compiler/src/runtime.rs` | 45–185 |
| moor-echo `TransformationRule` trait | `~/development/moor-echo/crates/echo-core/src/tracer/rules.rs` | 14–48 |
| moor-echo `SystemTracer` runner | `~/development/moor-echo/crates/echo-core/src/tracer/system_tracer.rs` | 22–55 |
| moor-echo `CompositeRule` chaining | `~/development/moor-echo/crates/echo-core/src/tracer/rules.rs` | 51–94 |
| moor-echo `TypedRule` fluent helper | `~/development/moor-echo/crates/echo-core/src/tracer/rules.rs` | 95–160 |
| moor-echo `RuleStats` | `~/development/moor-echo/crates/echo-core/src/tracer/rules.rs` | 161–200 |
| moor-echo MOO authorial doc | `~/development/moor-echo/docs/ECHO_SYSTEM_TRACER.md` | full |
| invalidation public surface | `~/development/invalidation/crates/invalidation/src/lib.rs` | 1–135 |
| invalidation README | `~/development/invalidation/README.md` | full |
| Cairn DESIGN goals | `~/development/cairn/DESIGN.md` | 6–131 |

## Appendix B — What this doc deliberately does NOT do

- Does not commit FMPL to any of these adoptions.
- Does not start implementation. Each recommendation should land as its own scoped iteration with PAR review.
- Does not propose deleting `fmpl-core/src/vm.rs`. The dual-VM parity gate (SCENARIO-0109) is a precursor only; replacing the in-tree VM is a downstream decision based on much more evidence than ITER-0004x or this doc provide.
- Does not propose adopting cairn's class model, Dylan-style generics, or multiple dispatch. Those are wrong for FMPL.
- Does not propose adopting moor-echo's MOO surface. The Rust trait is the entry point; the in-language authorial layer is metacircular ITER-0006+ work.
