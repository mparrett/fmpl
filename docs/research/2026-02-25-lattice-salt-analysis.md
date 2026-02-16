# Lattice/Salt Analysis for FMPL

## Overview

Analysis of the Lattice project (Salt language + microkernel) at `~/development/lattice` for primitives leverageable by FMPL's cooperative multi-user architecture.

## What Salt Is

Salt is an ahead-of-time compiled systems language with embedded Z3 formal verification. Pipeline: Salt -> AST -> HIR -> typed MLIR -> LLVM IR -> native code. The project includes a microkernel OS (boots on x86_64 QEMU), LETTUCE (Redis-compatible data store), Basalt (Llama 2 inference), and Facet (GPU compositor).

Notable engineering: the parser piggybacks on `syn` (Rust's proc-macro library) by preprocessing Salt syntax into Rust-compatible tokens.

## Leverageable Primitives

### High Value

**Z3 Formal Verification** -- Salt's `VerificationEngine` translates AST expressions to Z3 formulas using a "negate and check SAT" proof strategy with counterexample extraction. The `z3 = "0.12"` crate is ready to use. Salt's arena escape verifier uses a clean depth-based taint model (0=global, 1=argument, 2+=local).

- File: `salt-front/src/codegen/verification/`
- Pattern: Encode property as Z3 formula, negate, check SAT. If UNSAT, property holds. If SAT, counterexample extracted.
- FMPL use: Pattern match exhaustiveness, capability verification, contract checking

**Note**: Consider [OxiZ](https://github.com/cool-japan/oxiz) as a pure-Rust alternative to Z3 (see `2026-02-25-oxiz-smt-solver-analysis.md`).

### Medium-High Value

**Coroutine State Machine Compilation** -- `salt-front/src/hir/async_to_state.rs` (~79K lines) transforms `@yielding` functions into stackless state machines with jump-table dispatch, spill/reload logic, and 64-byte aligned TaskFrames. Relevant for optimizing FMPL's `AsyncStream` compilation.

**Pattern Match Exhaustiveness via Z3** -- `salt-front/src/codegen/verification/exhaustiveness.rs` formulates "exists x matching no arm" as a Z3 satisfiability problem. Directly applicable to FMPL's `@` pattern matching.

### Medium Value

**Cooperative Scheduling / Yield Injection** -- `salt-front/src/codegen/passes/yield_injection.rs` injects yield checks at loop back-edges with "stripe factor" analysis (check once per N iterations). Z3 elides checks for provably-short loops. Critical for multi-user resource safety (prevents player code from hanging the server).

**Context Capability Token** -- `salt-front/std/core/sovereign/context.salt` is a zero-sized phantom type that gates I/O. Only the executor can create one. Pattern for FMPL's `user` propagation: method dispatch requires a `UserContext` capability token carrying identity + permissions.

**Coherence Policing / Sovereign Authority** -- Prevents "orphan" trait implementations at compile time. For FMPL: who can extend an object's behavior? Only the owner (or holder of a programmer facet) can define new methods.

**Arena Memory for Hot Paths** -- Salt's arena model (`mark()`/`reset_to()` with epoch-based verification) for hot-path allocations during grammar parsing.

### Conceptual Value

**Multi-Dialect MLIR Strategy** -- Analyzes loop structure to choose optimal MLIR dialects. Relevant if FMPL targets native compilation.

## What Salt Does NOT Have

- No grammar system (no OMeta, no PEG)
- No prototype-based objects (nominal struct/impl/trait)
- No REPL/interpreter (purely compiled)
- No bytecode VM (compiles to native)
- No persistent storage layer
- No streaming-first primitives
- No user-level metaprogramming

## Key Files

| File | Purpose |
|------|---------|
| `salt-front/src/codegen/verification/` | Z3 verification infrastructure |
| `salt-front/src/hir/async_to_state.rs` | Coroutine state machine transformation |
| `salt-front/src/codegen/passes/yield_injection.rs` | Cooperative yield injection |
| `salt-front/std/core/sovereign/context.salt` | Capability token pattern |
| `salt-front/tests/codegen/sovereign_authority_test.rs` | Coherence policing |
| `salt-front/std/core/sovereign/executor.salt` | Work-stealing scheduler |
| `salt-front/std/channel/channel.salt` | Bounded/unbounded channels |

## References

- [Salt/Lattice repository](~/development/lattice)
- [Z3 Rust crate](https://crates.io/crates/z3)
- [OxiZ pure-Rust SMT solver](https://github.com/cool-japan/oxiz)
