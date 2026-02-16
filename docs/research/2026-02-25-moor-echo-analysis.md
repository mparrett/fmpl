# moor-echo (ECHO Language) Analysis for FMPL

## Overview

Analysis of the moor-echo project at `~/development/moor-echo` -- a Rust reimplementation of MOO extended with ECHO (Event-Centered Hybrid Objects). Explores the same design space as FMPL: modernizing LambdaMOO's security, concurrency, and object model.

## What moor-echo Is

A Rust workspace with 3 crates (echo-core, echo-repl, echo-web) implementing:
- Full MOO backward compatibility (imports LambdaMOO `.db` files)
- ECHO language extensions (capabilities, events, Datalog queries, colorless concurrency)
- Sled-backed persistence with rkyv serialization
- Optional Cranelift/Wasmtime JIT compilation
- Tree-walking evaluator (no bytecode VM)

## Key Design Decisions

### Capability-Based Security

ECHO replaces MOO's `.wizard`/`.programmer` flags with explicit capability declarations:

```echo
define capability CreatePlayer(creator);
define capability Move(player);

secure verb "go" (this, direction, "none", "none")
    requires Move(this)
    ...
endverb
```

Capabilities are defined at the language level with `define capability`, required with `requires`, and granted/revoked with `grant`/`revoke`. The `CapabilityManager` uses grant/denial model with default-deny.

**Relevance to FMPL**: FMPL's facets address attenuation (restricting what you can see); ECHO's capabilities address authorization (what you're allowed to do). These are complementary:
- **Facets** = "this view of the object only shows these members"
- **Capabilities** = "this caller is allowed to perform this operation"

FMPL likely needs both.

### Colorless Concurrency

ECHO explicitly avoids function colors. No `async`/`await` keywords. The runtime handles cooperative threading transparently with `gather` blocks for parallel operations.

**Relevance to FMPL**: FMPL has mild coloring via `<-` operator. Combined with Salt's yield injection pattern, there's a strong argument for making FMPL's concurrency colorless.

### Verb vs Function Distinction

ECHO explicitly separates player-facing `verbs` (matched by MUD command parser) from internal `functions` (called programmatically).

**Relevance to FMPL**: Grammar rules dispatch to verbs; regular method calls are functions. The conceptual split should be explicit in the object model.

### Meta-Object Protocol

Every object has a `$meta` property providing:
- Structural reflection (properties, verbs, events, queries, capabilities)
- Behavioral reflection (add/modify/wrap verbs, properties, handlers)
- Capability-based security on reflection operations (MetaRead, MetaWrite, MetaExecute, MetaGrant)

### Tuple Space Architecture

ECHO explored but deferred tuple spaces as a unifying communication mechanism. The referenced `TUPLE_SPACE_ARCHITECTURE.md` is missing from the repo. FMPL has tuple spaces implemented with faceted capability security -- a significant advantage.

## Three-Way Comparison

| Concern | moor-echo (ECHO) | FMPL (current) | Lattice/Salt |
|---------|------------------|-----------------|--------------|
| Object model | MOO-style `object...endobject` | Prototype-based (Goblins) | Struct + impl + trait |
| Security | `define capability` / `grant` / `revoke` | Facets (parsed, not enforced) | Z3 contracts + Context token |
| Verb dispatch | Full MOO verb system | Grammar-based (designed) | N/A |
| Concurrency | Colorless, `gather` blocks | `<-` operator (mild coloring) | `@yielding` + yield injection |
| Persistence | Sled + rkyv | Fjall + rkyv | None |
| Parsing | tree-sitter/rust-sitter | OMeta-style PEG grammars | syn-based |
| Multi-user | Player-scoped environments | `current_user` (plumbed, unused) | Ring 0/3 isolation |
| JIT | Cranelift + Wasmtime | Indexed RPN bytecode VM | MLIR -> LLVM -> native |
| Datalog | `query...endquery` in objects | N/A | N/A |

## What to Leverage

| Feature | Value | Notes |
|---------|-------|-------|
| Language-level capability declarations | High | `define capability` / `requires` / `grant` / `revoke` |
| Verb vs function distinction | High | Explicit in object model |
| Colorless concurrency | Medium-High | Eliminate `<-` coloring |
| Meta-Object Protocol with capability gates | Medium | `$meta` with MetaRead/MetaWrite/MetaExecute |
| MOO `.db` import infrastructure | Medium | If targeting MUD community |
| Datalog queries in objects | Conceptual | Grammars may subsume this |

## References

- [moor-echo repository](~/development/moor-echo)
- [LambdaMOO Programmer's Manual](https://tecfa.unige.ch/guides/MOO/ProgMan/)
- [Spritely Goblins](https://spritely.institute/goblins/)
