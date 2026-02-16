# Multi-User Architecture Synthesis

## Overview

Synthesis of research across four projects (FMPL, Lattice/Salt, moor-echo, ColdMUD) into a coherent multi-user cooperative system architecture for FMPL.

## The Emerging Architecture

```
User Input (text, websocket, HTTP, etc.)
    |
Grammar Layer (FMPL PEG grammars -- replaces ColdMUD's template matching)
    | parsed command structure
Capability Check (facets + capability tokens)
    | authorized invocation
Method Dispatch (object methods -- ColdMUD's clean separation)
    | may access
Tuple Space (coordination primitive)
    | persisted via
Fjall Store (transparent persistence)
```

With yield injection ensuring no player monopolizes the server.

## Design Principles (Derived from Research)

### From ColdMUD: Separate Parsing from Dispatch

Verbs are NOT commands. Grammar rules parse user input; methods handle the structured result. The VM knows nothing about commands, rooms, players, or permissions -- those are FMPL objects.

### From moor-echo: Explicit Capability Declarations

Replace MOO's `.wizard`/`.programmer` flags with `define capability` / `requires` / `grant` / `revoke` at the language level. Facets (attenuation) and capabilities (authorization) are complementary.

### From Lattice: Yield Injection for Resource Safety

Compiler-injected yield checks at loop back-edges with stripe-factor amortization. Prevents player code from hanging the server. Replaces MOO's tick limits with a more granular mechanism.

### From ColdMUD: Strict Encapsulation by Default

Properties private by default, exposed via methods. No property-level ACLs needed -- the method layer handles access control through facets.

### From ColdMUD: Driver Minimalism

The Rust runtime provides: bytecode execution, grammar evaluation, tuple space operations, I/O, and yield injection. Everything else is FMPL code.

## Current FMPL State vs Target

| Feature | Current | Target | Source of Inspiration |
|---------|---------|--------|----------------------|
| Object prototype chain | Implemented | Keep | MOO/ColdMUD/Self |
| `spawn` operator | Implemented | Keep | Goblins |
| Facet definitions | Parsed, not enforced | Enforce on dispatch | Goblins + ColdMUD frobs |
| `bcom` (become) | Not implemented | Implement | Goblins |
| Capability declarations | Not implemented | Language-level | moor-echo |
| `current_user` propagation | Plumbed, unused | Capability token | Lattice Context pattern |
| Verb/function distinction | Not explicit | Explicit in object model | moor-echo/ColdMUD |
| Visibility enforcement | Not implemented | `.#private` default | ColdMUD strict encapsulation |
| Yield injection | Not implemented | Compiler-level | Lattice |
| Multi-VAT | Not implemented | Phase 2 | Design docs |
| World model | Not implemented | FMPL objects + grammars | MOO/ColdMUD |

## Gaps to Fill

### Critical (Blocks Multi-User)

1. **Facet enforcement on method dispatch** -- `GetFacet` must create a restricted proxy, not just validate existence
2. **User context propagation** -- Capability token carrying identity through call chain
3. **Yield injection** -- Per-player resource budgets in bytecode compiler
4. **Capability declarations** -- `define capability` / `requires` at language level

### Important (Enables Rich Interaction)

5. **Verb dispatch wiring** -- Grammar semantic actions dispatch to object methods
6. **Property visibility enforcement** -- `.#private` default, `.#public` opt-in
7. **Object lifecycle commands** -- `@create` / `@recycle` / `@chown` equivalents
8. **Meta-Object Protocol** -- Capability-gated reflection

### Future (Scaling)

9. **Multi-VAT** -- Per-user event loops coordinating via tuple space
10. **Distributed auth** -- PASETO or similar token system
11. **World model** -- Rooms, exits, zones as FMPL objects

## FMPL's Unique Advantages

1. **Grammars as the unifying abstraction** -- No other project has PEG grammars with inheritance as first-class primitives for MUD command parsing
2. **Tuple space coordination** -- Implemented with faceted capability security; moor-echo explored and deferred this
3. **`@` operator unification** -- Parsing, pattern matching, and stream processing in one formalism

## References

- [Lattice/Salt Analysis](2026-02-25-lattice-salt-analysis.md)
- [moor-echo Analysis](2026-02-25-moor-echo-analysis.md)
- [ColdMUD Architecture](2026-02-25-coldmud-architecture.md)
- [OxiZ SMT Solver](2026-02-25-oxiz-smt-solver-analysis.md)
- [FMPL Revival Design](../plans/2025-12-19-fmpl-revival-design.md)
- [Object System Spec](../../specs/object-system.md)
- [Tuple Space Spec](../../specs/tuplespace.md)
