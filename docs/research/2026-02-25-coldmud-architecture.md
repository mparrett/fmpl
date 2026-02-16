# ColdMUD/Genesis Architecture Analysis

## Overview

Analysis of ColdMUD (Genesis server, ColdC language, ColdCore database) and how its clean separation of parsing from dispatch validates and informs FMPL's grammar-based architecture.

## What ColdMUD Is

ColdMUD was created by Greg Hudson at MIT in 1993. Its successor, Genesis (by Brandon Gillespie, ~1995), is the production implementation. The system has three layers:

1. **ColdC** -- The language (dynamically typed, prototype-based, C-derived syntax)
2. **Genesis** -- The byte-compiler and interpreter (the "driver")
3. **ColdCore** -- The MUD application written in ColdC

Genesis was used commercially by "The Eternal City," validating production-readiness.

## The Critical Design Decision: Parsing vs Dispatch

### LambdaMOO: Verbs ARE Commands

A MOO verb simultaneously defines a callable function AND a user-facing command handler. The server's built-in parser matches typed input directly to verbs via argument specifiers (`this`/`any`/`none` for dobj, prep, iobj). Parsing and dispatch are fused.

### ColdMUD: Methods Are Pure Functions

The Genesis driver delivers raw input to the database and does nothing else. A separate `$command_parser` chain (written in ColdC) matches templates to methods. Three distinct layers:

1. **Parser** -- matches user input to a template (swappable per-connection)
2. **Command registration** -- binds templates to methods (`@ac "take *" to $thing.take_cmd`)
3. **Method** -- pure function that receives parsed arguments

This enables multiple input protocols (MUD commands, HTTP, SMTP, POP3) by swapping parsers without touching methods.

## Key ColdC Features

### Strict Encapsulation

All instance variables are private -- no external access. Methods must be written to expose them. This is more restrictive than MOO (which has `r`/`w`/`c` property flags) but simpler and more secure.

**Implication for FMPL**: Make `.#private` the default, `.#public` the explicit opt-in.

### Frobs (Lightweight Proxy Objects)

ColdC's frobs (`<class, value>` or `<class, value, handler>`) are immutable proxy objects. Method calls on a frob are intercepted -- the class receives the call with the value as first argument. With a handler, the handler receives the method name as a symbol.

**Implication for FMPL**: Facets could be implemented as frob-like proxies where method calls are intercepted and capability-checked before forwarding.

### Driver Minimalism

Genesis knows nothing about commands, permissions, rooms, players, or any MUD concepts. It interprets ColdC and delivers connection I/O. All MUD semantics live in ColdCore.

**Implication for FMPL**: The Rust VM handles bytecode execution, grammar evaluation, tuple space ops, and I/O. Everything else (command parsing, world model, player management, permissions) should be FMPL objects using FMPL grammars.

### Security via Encapsulation, Not Flags

No built-in security in the driver. Security implemented entirely in ColdCore using ColdC's strict encapsulation. Four tiers (System/Manager/Writer/Trustee) and groups are all database objects.

**Implication for FMPL**: The VM provides facets and tuple space capabilities as primitives; actual security policy is expressed in FMPL code, not Rust code.

### No Built-in Properties

ColdC has no built-in `owner` or `location` properties. The language is deliberately minimal, pushing concepts like ownership and containment into the database layer.

### Multiple Inheritance

Objects can have multiple parents, forming a DAG. Precedence: direct ancestors before their ancestors; first parent line before second parent line.

### `pass()` for Parent Delegation

Similar to `super` -- delegates to parent's implementation of the same method.

### `disallow_overrides`

Methods can prevent descendants from overriding them.

## Mapping to FMPL

| ColdMUD | FMPL Equivalent |
|---------|-----------------|
| `match_template("take *")` | `grammar mud::commands { take = "take" spaces noun:obj => ... }` |
| `$command_parser` chain | Grammar composition/inheritance (`<:`) |
| `.add_command(template, method)` | Grammar semantic action dispatches to object method |
| Per-connection parser swap | Per-stream grammar selection via `@` |
| `$http_interface` parser | `grammar http::request <: base::parser { ... }` |
| Frobs (proxy objects) | Facet proxies with capability checking |
| `pass()` | Prototype chain delegation |
| `disallow_overrides` | Terminal facets (`facet!:`) |

## The FMPL Advantage

ColdMUD had to invent `match_template()` and a command cache because it lacked grammars. FMPL has PEG grammars as a first-class primitive, so the MUD command parser is just another grammar that composes with other grammars -- the `mud::commands <: base::parser` pattern already in the codebase.

## References

- [ColdC/Genesis Homepage](https://cold.org/coldc/genesis.html)
- [ColdCore GitHub](https://github.com/the-cold-dark/ColdCore)
- [Genesis GitHub](https://github.com/the-cold-dark/genesis)
- [ColdC GitHub (whilke)](https://github.com/whilke/ColdC)
- [MIT ColdMUD Reference](https://stuff.mit.edu/afs/sipb/project/coldmud/doc/coldmud.html)
- [ColdCore Help 2017](https://lisdude.com/cold/documentation/Cold_Help_2017.txt)
- [Cold/Genesis Resources Archive](https://lisdude.com/cold/)
- [ColdC History](https://cold.org/coldc/history.html)
- [Genesis Project Page](http://the-cold-dark.github.io/genesis/)
