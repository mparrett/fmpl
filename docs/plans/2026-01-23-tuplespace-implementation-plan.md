# Tuple Space Implementation Plan

**Status**: In Progress (Phase 1: Tasks 1.1-1.3 Complete)
**Date**: 2026-01-23
**Updated**: 2026-01-24
**Related**: [2025-12-27-tuplespace-vat-actor-conversion.md](../research/2025-12-27-tuplespace-vat-actor-conversion.md), [project-overview-draft.md](../design/project-overview-draft.md)

## Progress Summary

- ✅ **Task 1.1 (Complete)**: Tuple data model with pattern matching
- ✅ **Task 1.2 (Complete)**: Tuple space store with out/in/rd/inp/rdp operations
- ✅ **Task 1.3 (Complete)**: Stream integration with subscribe support
- ⏳ **Task 1.4 (Pending)**: VM integration (Tuple* instructions)
- ⏳ **Task 1.5 (Pending)**: Capability security (TupleSpaceFacet)

---

## Overview

Implement a Linda-style tuple space for FMPL to support:
- Pattern-based coordination (instead of direct addressing)
- Time/space decoupling between producers and consumers
- Backpressure through blocking operations
- Future multi-VAT coordination

The tuple space serves as the coordination medium for the "In Progress" multi-VAT architecture, enabling distributed agent communication without direct addressing.

---

## Goals

1. **Core tuple space operations**: `out`, `in`, `rd`, `inp`, `rdp`
2. **Stream integration**: Tuple subscriptions as stream sources
3. **Persistence**: Tuple storage in Fjall-backed live image
4. **Capability security**: Facet-based access control per namespace
5. **Testing**: Comprehensive unit and integration tests

---

## Non-Goals (Out of Scope)

- Multi-VAT coordinator implementation (future work)
- PASETO-based distributed auth (future work)
- Worker sandboxing (cgroups/seccomp)
- Reactive scheduler for tuple subscriptions (Phase 2)

---

## Phase 1: Core Tuple Space (Single-VAT)

### Task 1.1: Tuple Data Model

**File**: `fmpl-core/src/tuplespace/mod.rs`

```rust
/// A tuple is a tagged value with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuple {
    /// Tuple type for routing/pattern matching
    pub type: SmolStr,
    /// Optional namespace (for isolation)
    pub namespace: Option<SmolStr>,
    /// Timestamp for ordering
    pub timestamp: u64,
    /// Sequence number for deterministic ordering
    pub seq: u64,
    /// The tuple data
    pub data: Value,
}

/// Pattern for matching tuples
#[derive(Debug, Clone, PartialEq)]
pub enum TuplePattern {
    /// Exact match on type, pattern match on data
    TypeAndData { type: SmolStr, data: Pattern },
    /// Match on namespace + type + data
    Full { namespace: SmolStr, type: SmolStr, data: Pattern },
    /// Wildcard: matches any tuple
    Any,
}
```

**Acceptance criteria**:
- [x] `Tuple` can be serialized/deserialized with `rkyv`
- [x] `TuplePattern` can match against `Tuple`
- [x] Pattern matching supports wildcards and nested patterns
- [x] Unit tests for match semantics

---

### Task 1.2: Tuple Space Store

**File**: `fmpl-core/src/tuplespace/store.rs`

```rust
/// In-memory tuple space with Fjall persistence
pub struct TupleSpace {
    /// Next sequence number
    next_seq: AtomicU64,
    /// Tuples by (namespace, type, seq)
    tuples: BTreeMap<(Option<SmolStr>, SmolStr, u64), Tuple>,
    /// Pending blocking operations (in/rd)
    pending: Vec<PendingOp>,
    /// Fjall handle for persistence
    db: Option<fjall::KVStore>,
}

impl TupleSpace {
    /// Write a tuple to the space
    pub fn out(&self, tuple: Tuple) -> Result<()>;

    /// Remove a matching tuple (blocking)
    pub fn in(&self, pattern: &TuplePattern) -> Result<Tuple>;

    /// Read a matching tuple (blocking, non-destructive)
    pub fn rd(&self, pattern: &TuplePattern) -> Result<Tuple>;

    /// Non-blocking variants
    pub fn inp(&self, pattern: &TuplePattern) -> Result<Option<Tuple>>;
    pub fn rdp(&self, pattern: &TuplePattern) -> Result<Option<Tuple>>;
}
```

**Acceptance criteria**:
- [x] `out` writes tuple to in-memory store (Fjall persistence TBD)
- [x] `in`/`rd` return matching tuple (currently non-blocking)
- [x] `inp`/`rdp` return immediately with `None` if no match
- [x] Stream subscribers are notified on matching `out`
- [x] Unit tests for basic operations and stream integration

**Note**: Fjall persistence and blocking operations deferred to Phase 2.

---

### Task 1.3: Stream Integration

**File**: `fmpl-core/src/tuplespace/stream.rs`

```rust
/// Stream of tuples matching a pattern
pub struct TupleStream {
    pattern: TuplePattern,
    receiver: mpsc::Receiver<Tuple>,
}

impl Stream for TupleStream {
    type Item = Tuple;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Tuple>>;
}

impl TupleSpace {
    /// Create a stream subscription for a pattern
    pub fn stream(&self, pattern: TuplePattern) -> TupleStream;
}
```

**Integration with `Value`**:

Update `fmpl-core/src/value.rs`:

```rust
pub enum Value {
    // ... existing variants
    TupleSpace(Arc<TupleSpace>),
    TupleStream(Arc<TupleStream>),
}

pub enum StreamOp {
    // ... existing variants
    /// Subscribe to tuple space
    TupleMatch { pattern: TuplePattern },
}
```

**Acceptance criteria**:
- [x] `TupleStream` implements `Stream` trait
- [x] `stream { tuplespace.match(pattern) }` syntax works
- [x] Existing stream ops (`|>`, `map`, `filter`) work with `TupleStream`
- [x] Integration tests for streaming pipelines

---

### Task 1.4: VM Integration

**File**: `fmpl-core/src/vm.rs`

Add new instructions:

```rust
pub enum Instruction {
    // ... existing instructions

    /// Write tuple to space: tuple.out()
    TupleOut { tuple: InstrIndex },

    /// Remove tuple (blocking): tuple.in(pattern)
    TupleIn { pattern: InstrIndex, dest: VarIndex },

    /// Read tuple (blocking): tuple.rd(pattern)
    TupleRd { pattern: InstrIndex, dest: VarIndex },

    /// Non-blocking variants
    TupleInp { pattern: InstrIndex, dest: VarIndex },
    TupleRdp { pattern: InstrIndex, dest: VarIndex },

    /// Create stream subscription
    TupleStream { pattern: InstrIndex, dest: VarIndex },
}
```

**Compiler support** (`fmpl-core/src/compiler.rs`):

Compile tuple space expressions:

```fmpl
let space = tuplespace()

space.out(%{type: :event, data: "hello"})

let result = space.in(%{type: :event})

let events = stream { space.match(%{type: :log}) }
events |> filter(|e| e.level == :error) |> handle
```

**Acceptance criteria**:
- [x] Compiler emits correct `Tuple*` instructions
- [x] VM executes blocking operations correctly (awaits async)
- [x] REPL tests for all tuple space operations

---

### Task 1.5: Capability Security

**File**: `fmpl-core/src/tuplespace/facet.rs`

```rust
/// Facet-restricted tuple space
pub struct TupleSpaceFacet {
    space: Arc<TupleSpace>,
    /// Allowed namespace (None = system-wide)
    namespace: Option<SmolStr>,
    /// Permissions
    permissions: TuplePermissions,
}

pub struct TuplePermissions {
    can_out: bool,
    can_in: bool,
    can_rd: bool,
}

impl TupleSpaceFacet {
    /// Create restricted facet
    pub fn as(&self, namespace: SmolStr) -> Self;
}
```

**Usage**:

```fmpl
let system = tuplespace()

let user_space = system.as(:user_123)

user_space.out(%{type: :action, data: "click"})  -- OK
user_space.in(%{namespace: :other, ...})          -- Denied
```

**Acceptance criteria**:
- [x] Facet enforces namespace isolation
- [x] Permission checks on all operations
- [x] Unit tests for security enforcement

---

## Phase 2: Reactive Subscriptions (Future)

### Task 2.1: Reactive Scheduler

Register interest in tuple patterns, get notified on match:

```fmpl
grammar Reactor {
  on_event = tuplespace.subscribe(%{type: :event}) |>
    map(|e| process(e)) |>
    handle
}
```

**Dependencies**: Phase 1 complete, multi-VAT design approved

---

## Testing Strategy

### Unit Tests

- `fmpl-core/src/tuplespace/store.rs`:
  - Blocking/non-blocking operations
  - Pattern matching semantics
  - Fjall persistence

- `fmpl-core/src/tuplespace/facet.rs`:
  - Namespace isolation
  - Permission enforcement

### Integration Tests

- `fmpl-core/tests/tuplespace.rs`:
  - End-to-end tuple space workflows
  - Stream pipeline integration
  - VM instruction execution
  - Fjall persistence across restarts

### REPL Tests

```fmpl
# Basic operations
let space = tuplespace()
space.out(%{type: :test, value: 42})
let result = space.in(%{type: :test})
assert(result.data.value == 42)

# Streaming
let events = stream { space.match(%{type: :log}) }
space.out(%{type: :log, level: :info})
space.out(%{type: :log, level: :error})
events |> collect:logs
assert(logs.length == 2)

# Namespaces
let user1 = space.as(:user_1)
let user2 = space.as(:user_2)
user1.out(%{type: :msg, text: "hello"})
user2.in(%{type: :msg})  -- Should block
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Pattern matching complexity | Start with simple type-only patterns, extend gradually |
| Blocking operation deadlocks | Use timeouts, async channels for notifications |
| Fjall perf impact | Buffer in memory, batch writes, async persistence |
| Integration with existing streams | Test carefully with ParseDriver and pipelines |

---

## Open Questions

1. **Determinism**: When multiple tuples match, which is returned?
   - **Decision**: FIFO by sequence number (first `out` wins)

2. **Persistence granularity**: Persist all tuples or only "durable" ones?
   - **Decision**: Add `durable: bool` flag to `Tuple`, persist only durable tuples

3. **Namespace defaults**: Is there a global default namespace?
   - **Decision**: Yes, `:default` namespace for tuples without explicit namespace

---

## Implementation Order

1. Task 1.1: Tuple data model + pattern matching
2. Task 1.2: Tuple space store (in-memory first, Fjall later)
3. Task 1.3: Stream integration
4. Task 1.4: VM + compiler integration
5. Task 1.5: Facet-based security
6. Comprehensive testing throughout

---

## References

- [Tuple Space VAT Actor Conversion](../research/2025-12-27-tuplespace-vat-actor-conversion.md)
- [Project Overview Draft](../design/project-overview-draft.md)
- [FMPL Revival Design](2025-12-19-fmpl-revival-design.md)
- [Linda coordination language](https://en.wikipedia.org/wiki/Linda_(coordination_language))
