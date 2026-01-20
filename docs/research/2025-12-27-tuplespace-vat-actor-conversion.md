# Tuplespace Conversion Research

Date: 2025-12-27

## Scope

Research what it would take to shift FMPL's current vat/actor framing toward a Linda-style tuplespace for message passing and stream coordination, and outline implications for the stream primitive and storylet flow.

## Linda and Tuple Spaces (Key Properties)

Linda introduces a shared coordination medium called a tuplespace. Core operations are:

- `out(t)`: write a tuple to the space.
- `in(p)`: remove a tuple matching pattern `p` (blocking).
- `rd(p)`: read a tuple matching pattern `p` (blocking, non-destructive).
- `inp(p)`, `rdp(p)`: non-blocking variants.
- `eval(t)`: spawn a process that eventually emits a tuple (in some implementations).

The essential properties are associative access (pattern matching, not addressing), time/space decoupling (producers and consumers do not need to know each other or be alive concurrently), and potential nondeterminism when multiple tuples match.

Sources: https://en.wikipedia.org/wiki/Linda_(coordination_language), https://en.wikipedia.org/wiki/Tuple_space

## Actor/Vat Model (Key Properties)

Actor model: each actor has a mailbox; messages are sent to a specific address; ordering is guaranteed per sender; actors encapsulate state and only mutate themselves; processing is asynchronous. A vat is typically a single-threaded event loop hosting multiple actors to enforce no shared mutable state across vats.

Source: https://en.wikipedia.org/wiki/Actor_model

## Conversion Implications

### 1) Addressing vs Matching

- Actors: explicit addresses (object IDs), direct sends.
- Tuplespace: data-centric, match-based access. You need structured tuple schemas (e.g., `%{type: :choice, player: @id, ...}`) and pattern matching semantics to replace direct addressing.

Implication: FMPL would need a canonical tuple shape for system events (storylet actions, stream events, background tasks) and structured patterns to avoid collisions.

### 2) Ordering and Determinism

- Actors: per-sender ordering; predictable sequencing.
- Tuplespace: nondeterministic match if multiple tuples satisfy a pattern.

Implication: if ordering matters (e.g., user actions), include sequence numbers or timestamps and enforce ordering in the consumer. If ordering is intentionally loose, the stream semantics need to reflect nondeterministic consumption.

### 3) Blocking and Backpressure

`in`/`rd` are blocking in Linda. In FMPL, this maps naturally to stream subscription or suspension of evaluation. Tuple-based coordination can provide backpressure: consumers block until tuples arrive.

Implication: the stream runtime should be able to block on tuplespace patterns, or expose async forms (polling with non-blocking variants) where appropriate.

### 4) Shared State vs Isolation

Actors guarantee local state isolation. Tuplespace is shared state. For FMPL, this can be reconciled by:

- Namespacing tuples by vat/actor or capability token.
- Treating the tuplespace as a capability (object with facet-limited operations).
- Ensuring tuple patterns can only match within a controlled namespace.

### 5) Persistence and the Live Image

A tuplespace can be persisted alongside the live image (the image is already the source of truth). Tuple history could be part of the continuation payload or stored as a separate durable log.

Implication: tuplespace operations should be transactionally tied to the tick/session commit to keep the image consistent.

## Integration with FMPL Streams

### Proposed Semantics

- A tuplespace is a stream root: `stream { tuplespace }` produces a stream of tuples matching a pattern or subscription rule.
- `stream |> filter(|t| ...)` and `stream |> parse(g.rule)` can be used to match and transform tuples.
- `out` adds to the tuplespace and thus pushes into streams; `in`/`rd` correspond to consuming or observing tuple events.

### Suggested API Surface (FMPL)

- `tuplespace.out(tuple)`
- `tuplespace.in(pattern)`
- `tuplespace.rd(pattern)`
- `tuplespace.inp(pattern)`
- `tuplespace.rdp(pattern)`
- `stream { tuplespace.match(pattern) }` or `tuplespace.stream(pattern)`

### Storylet Actions as Tuples

Current continuation payload keeps a JSON stream of actions in `fmpl-web/src/continuations.rs`. This can be reinterpreted as a tuple stream. Each action becomes a tuple:

```fmpl
%{type: :choice, choice: :listen, timestamp: 1700000000, player: @id}
```

The stream history then becomes a bounded tuple stream. The tuple space can store or emit these, and the continuation stream can keep the last N events for resumption.

## Implementation Outline

### Data Model

- Tuple = map or list with required tags (type + metadata).
- Pattern matching = map-subset match with symbol wildcards; use existing pattern machinery where possible.

### Runtime Components

1) Tuplespace store:
- In-memory + persisted (image)
- Indexing by tuple type and key fields
- Atomic operations for `in`/`rd`

2) Stream bridge:
- Convert tuplespace subscriptions into stream sources
- Use existing stream ops in `fmpl-core/src/value.rs`

3) Continuation linkage:
- The bounded action history can be a stream of tuples in the continuation payload
- When rollover occurs, store previous tuplespace segments and link via `prev` token

### Security/Capabilities

- Put tuplespace behind a capability (facet) with per-player namespaces
- Consider separate system tuplespace for global coordination and per-player spaces for action history

## Required Code Touchpoints (Current Repo)

- `fmpl-core/src/value.rs` defines `Value::Stream` and `StreamOp`.
- `fmpl-web/src/continuations.rs` stores action history as JSON stream payload.
- `fmpl-web/src/storylet.rs` uses continuations for storylet choice actions.
- `docs/plans/2025-12-19-fmpl-revival-design.md` (streams + event model direction).

## Risks / Open Questions

- Pattern matching semantics: should tuples be maps, lists, or both? How does matching interact with existing grammar patterns?
- Ordering: do we require deterministic selection of tuples when multiple match?
- Persistence: do we keep tuplespace history in the image or only in continuation payloads?
- Namespace/permissions: who can read/write which tuples?

## References

- Linda coordination language: https://en.wikipedia.org/wiki/Linda_(coordination_language)
- Tuple space: https://en.wikipedia.org/wiki/Tuple_space
- Actor model: https://en.wikipedia.org/wiki/Actor_model
- FMPL stream/runtime references: `fmpl-core/src/value.rs`, `fmpl-web/src/continuations.rs`, `fmpl-web/src/storylet.rs`
- North Star design and streams: `docs/plans/2025-12-19-fmpl-revival-design.md`
