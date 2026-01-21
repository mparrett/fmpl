# FMPL vs. Agentic System: 100-Line Summary

**Date**: 2026-01-20

## Architecture
**FMPL**: Multi-VAT Goblins-inspired system (N event loops, tick-based), streaming grammars, facet-based capabilities, planned tuple space

**Agentic**: Multi-process coordinator + workers, tuple space with reactive scheduling, PASETO token capabilities

## Key Convergence
Both systems use:
- **Linda-style tuple spaces** for coordination
- **Capability-based security** (no ambient authority)
- **Fjall persistence** for durable state
- **Fjall already used** in FMPL for live image + streaming

## Key Divergence
| Aspect | FMPL | Agentic |
|--------|------|----------|
| **Execution** | Multi-VAT event loops (coordinator or standalone) | Reactive scheduler, N workers |
| **Object model** | Goblins spawn/bcom, automatic transactions | Not defined |
| **Security** | Facets (object-bound, static) | PASETO tokens (hierarchical, revocable) |
| **Multi-user** | Not designed (shared VM) | First-class users, namespaces |
| **Coordination** | Streams + planned tuples | Reactive tuple subscriptions |
| **Async** | `<-` operator (Goblins), `$` sync | Blocking tuple reads |

## Facets vs. Tokens
**Facets** (FMPL): Object-bound, static surfaces, `obj.as(:facet)`, terminal: bool prevents re-faceting

**PASETO** (Agentic): Token-based, dynamic hierarchical attenuation, instant revocation (delete signing key)

**Both ARE capabilities** — different implementation for different deployment models (local vs distributed)

## Tuple Space: FMPL Plans
Designed (not implemented): `out`/`in`/`rd` Linda ops, stream integration (`stream { tuplespace }`), facet-based namespace security

Agentic: Same Linda ops + reactive subscriptions (`on (pattern) { handler }`)

**Research question**: VAT model → tuple space conversion (addressing vs matching, ordering, isolation)

## Current FMPL Status
✅ Implemented (6,500 LOC):
- Streaming grammars (incremental PEG, backtracking, Fjall persistence)
- Async operations (`<-` operator, tokio channels, StreamOp)
- Facets (Goblins-style object capabilities)
- Exception handling (cross-frame try/catch)
- Spawn operator (Goblins `spawn ^constructor`)

⏳ Planned:
- Tuple space (`out`/`in`/`rd` + streams)
- bcom (functional state updates)
- Automatic transactions (Goblins rollback)
- LLM client for streaming grammars

❌ Not designed:
- Multi-user (multi-VAT but no coordinator/namespace isolation)
- Multi-process (multi-VAT but no central coordinator)
- Worker sandboxing (cgroups, seccomp)
- Task lifecycle (spawn/suspend/resume)
- PASETO tokens (multi-user only)

## Strategic Recommendation: Hybrid
**Coordinator + Tuple Store + PASETO**
  └── N Workers (separate processes)
      └── Each runs FMPL VAT
          ├── Goblins patterns (spawn/bcom, auto-transactions)
          ├── Streaming grammars (agent behavior)
          ├── Facets (local object security)
          └── Tuple space integration (VAT coordination)

**Benefits**:
- Leverages 6,500 LOC of FMPL work
- Declarative agents (grammars) + coordination (tuples)
- Horizontal scaling (multi-VAT workers)
- Security: Facets (local) + PASETO (multi-user)

**Estimated effort**: 4-8 months

## Naming Note
**Language has diverged** significantly from original FMPL (1992) — probably deserves new name. Current implementation is:
- Multi-VAT Goblins-inspired (original was single-process)
- Streaming grammars (not in original)
- Facet capabilities (Goblins-style, not original FMPL)
- Fjall persistence (original had live image too)

## Reference
- This implementation: Multi-VAT, streaming grammars, facets
- Agentic system: Multi-process tuple space, reactive scheduler, PASETO
- Both: Linda-style tuples, capability security, Fjall persistence
- Convergence: Tuple spaces + capabilities (different implementations)
