# Multi-Principal

Multiple principals (humans, LLM agents, system) share the image via faceted capabilities. Multi-user and multi-agent are the same problem.

## Current State

- `vm.rs:42` — `current_user: Option<ObjectId>` exists on Vm struct, **never set**
- `vm.rs:366` — `LoadUser` instruction reads `current_user`, returns `:none` if unset
- `tuplespace/facet.rs:57` — `TupleSpaceFacet` enforces `can_out`/`can_in`/`can_rd` permissions
- No per-principal isolation (single VAT, single event loop)

## Principals

Every action has a principal. The `user` magical variable carries identity through the call chain.

| Type | Identity Source | Example |
|------|----------------|---------|
| Human | Session/connection | Player typing `take sword` |
| LLM agent | Spawned with capability token | Agent calling `treasury.as(:auditor).view_balance()` |
| System | Root capability | Cron job, startup |

### Capability Tokens

A principal's authority = the set of facets they hold. No ambient authority.

```fmpl
let agent_view = treasury.as(:auditor)
let agent = spawn llm_agent(agent_view)
-- Agent can only call view_balance, never withdraw
```

## Implementation: User Context

### Phase 1: Set `current_user` from connection

**`vm.rs:42`** — `current_user` must be set when processing a connection's input:

```rust
// When a connection sends input:
vm.current_user = Some(session.principal_id);
// Execute the parsed command
vm.eval(compiled_input);
// Clear after turn
vm.current_user = None;
```

**`fmpl-web/src/main.rs`** — The `/eval` handler must set `current_user` before evaluation.

**`fmpl-cli`** — REPL sets `current_user` to a system principal (root access).

### Phase 2: Propagate through `<-` calls

When an async message is sent cross-VAT, the sender's identity must be captured in the message envelope and restored when the message is processed.

## Implementation: Yield Injection

Compiler-injected yield checks prevent any principal from monopolizing the server.

### Mechanism

At every loop back-edge, the compiler emits a `YieldCheck` instruction:

```rust
// New instruction in compiler.rs:
YieldCheck  // Decrements budget, yields if exhausted
```

**`compiler.rs`** — Emit `YieldCheck` before every `Jump` instruction that targets a lower IP (back-edge).

**`vm.rs`** — Handle `YieldCheck`:
```rust
Instruction::YieldCheck => {
    self.turn_budget -= 1;
    if self.turn_budget <= 0 {
        // Suspend this turn, schedule resumption
        return Err(Error::YieldExhausted);
    }
}
```

**`vm.rs`** — Add `turn_budget: u32` field, reset at start of each turn. Default budget: 10_000 instructions.

### Stripe Factor

Amortize yield overhead: only check every N back-edges. N=8 gives ~12% overhead instead of per-iteration cost.

## Implementation: Multi-VAT

### Phase 2 Target

Each principal runs in a VAT (Virtual Address Territory). A VAT is a single-threaded event loop.

- `$` — Same-VAT synchronous call
- `<-` — Cross-VAT async, returns stream/promise
- Turns are atomic: errors roll back state changes

### Architecture

```
Connection/Agent → Input Queue → VAT (event loop) → Output Queue → Connection/Agent
                                   |
                                   v
                              Shared ObjectDb (with locking)
                              Shared TupleSpace (with facets)
```

Each VAT has its own turn budget. Cross-VAT calls go through message queues. The tuple space is the primary coordination mechanism.

### Promise Pipelining

```fmpl
<- (<- bank.get_account("alice")).get_balance()
-- Single round trip: pipeline the calls
```

The first `<-` returns a promise. The second `<-` is sent as a pipelined message -- it doesn't wait for the first to resolve before being queued.

## Implementation: Grammar Dispatch

Each connection has a grammar stack for parsing input. Different grammars for different protocols (ColdMUD pattern):

```fmpl
grammar mud::commands <: base::parser {
  command = verb:v spaces noun:n => %{verb: v, noun: n}
}
```

- MUD clients: `mud::commands` grammar
- HTTP: `http::request` grammar
- LLM API: `json::rpc` grammar

Grammar is swappable per-connection without touching object methods.

## Target Files

| File | Change |
|------|--------|
| `vm.rs:42` | Set `current_user` from connection context |
| `vm.rs` | Add `turn_budget`, `YieldCheck` handler |
| `compiler.rs` | Emit `YieldCheck` at loop back-edges |
| `fmpl-web/src/main.rs` | Set principal on `/eval` |
| `fmpl-cli/src/main.rs` | Set system principal for REPL |

## Related

- [facets](facets.md) — Capability model
- [tuplespace.md](../tuplespace.md) — Coordination primitive
- [grammar-system.md](../grammar-system.md) — Per-connection grammar dispatch
- Research: [multi-user-synthesis](../../docs/research/2026-02-25-multi-user-architecture-synthesis.md), [moor-echo](../../docs/research/2026-02-25-moor-echo-analysis.md), [lattice](../../docs/research/2026-02-25-lattice-salt-analysis.md)
