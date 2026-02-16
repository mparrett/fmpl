# OxiZ: Pure Rust SMT Solver Analysis

## Overview

Analysis of [OxiZ](https://github.com/cool-japan/oxiz) as a pure-Rust alternative to Z3 for FMPL's constraint solving needs (pattern match exhaustiveness, type inference, capability verification, contracts).

## What OxiZ Is

OxiZ is a pure-Rust reimplementation of Z3 announced January 2026. Zero C/C++ dependencies, zero FFI. `cargo build` is all you need.

- **Version**: 0.1.3
- **Code**: 284,414 lines of production Rust
- **Tests**: 5,814 passing (100% pass rate)
- **Benchmark**: 100% Z3 parity across 88 tests in 8 core SMT-LIB logics
- **License**: Apache-2.0

## Supported Theories (Production-Ready)

| Logic | Description | FMPL Use Case |
|-------|-------------|---------------|
| QF_LIA | Linear Integer Arithmetic | Type constraints, contracts |
| QF_LRA | Linear Real Arithmetic | Numeric contracts |
| QF_NIA | Nonlinear Integer Arithmetic | Complex contracts |
| QF_BV | Bit-Vectors | Low-level operations |
| QF_FP | Floating Point | Float contracts |
| QF_S | Strings | String constraints |
| QF_DT | Datatypes/ADT | Pattern match exhaustiveness |
| QF_A | Arrays | Tuple space invariants |

Additional theories (partial): QF_UF, QF_NRA, AUFBV, UFLIA, HORN.

## API

Uses SMT-LIB2 format via `Context`:

```rust
use oxiz::solver::Context;

let mut ctx = Context::new();
let results = ctx.execute_script(r#"
    (set-logic QF_LIA)
    (declare-const x Int)
    (declare-const y Int)
    (assert (> x 0))
    (assert (< y 10))
    (assert (= (+ x y) 15))
    (check-sat)
    (get-model)
"#).unwrap();
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `oxiz` | Meta-crate, unified API |
| `oxiz-core` | AST, sorts, SMT-LIB parser, tactics |
| `oxiz-math` | Simplex, LP, matrices, polynomials |
| `oxiz-sat` | CDCL SAT solver (VSIDS/LRB/VMTF) |
| `oxiz-nlsat` | Nonlinear arithmetic (CAD) |
| `oxiz-theories` | EUF, Arith, BV, Arrays, Strings, FP, ADT |
| `oxiz-solver` | CDCL(T) orchestration, MBQI |
| `oxiz-opt` | MaxSAT, optimization (OMT) |
| `oxiz-spacer` | CHC solving, PDR/IC3, BMC |
| `oxiz-proof` | Proof generation (DRAT, Alethe, LFSC) |
| `oxiz-wasm` | WebAssembly bindings |

## Maturity Assessment

**Strengths**:
- Pure Rust -- no build complexity, WASM-ready
- The theories FMPL needs (QF_LIA, QF_DT) are among the validated 8
- Z3-compatible API (SMT-LIB2)
- Modular crate structure allows pulling only what's needed

**Concerns**:
- Version 0.1.3 with ~98 total downloads
- "100% parity" based on only 88 benchmarks (Z3 validated against thousands)
- Single-developer project (cool-japan/KitaSan)
- No independent reviews or production battle-testing
- Quantifier handling partial (MBQI "partially implemented")

## Recommendation for FMPL

**Use OxiZ as the default, with Z3 as fallback.**

1. Abstract the solver behind a trait:
   ```rust
   trait SmtSolver {
       fn check_exhaustiveness(&self, patterns: &[Pattern]) -> ExhaustivenessResult;
       fn check_contract(&self, pre: &Expr, post: &Expr) -> ContractResult;
       fn infer_types(&self, constraints: &[TypeConstraint]) -> TypeAssignment;
   }
   ```

2. Implement `OxizSolver` (default, pure Rust) and `Z3Solver` (fallback, requires system Z3).

3. FMPL's queries are relatively simple (no deep quantifier nesting, no complex theory combinations) -- well within OxiZ's validated capabilities.

4. The SMT-LIB2 format is standard -- same queries work on both solvers.

## SMT Solvers for Type Inference

The approach is well-established:

1. **Constraint generation**: Walk AST, emit type constraints from expressions
2. **Constraint solving**: Encode in SMT-LIB, query for satisfying assignment
3. **Type extraction**: Read model -- each variable gets a concrete type

Key references:
- InferType (ECOOP 2024) -- toolkit using Z3 for Python type inference
- SMT-based Static Type Inference for Python 3 (ETH Zurich)
- Concrete Type Inference with ML + SMT (OOPSLA 2023)
- Thrust (PLDI 2025) -- refinement types for Rust via CHC

**Caveat**: For basic Hindley-Milner style inference, a union-find unification engine is simpler and faster. Use SMT for the hard stuff (exhaustiveness, contracts, capability verification) and a lightweight unifier for basic type inference.

## Related: Rust-Native Alternatives

| Tool | Approach | Notes |
|------|----------|-------|
| OxiZ | Pure Rust SMT solver | Z3 reimplementation |
| `z3` crate | FFI bindings to C++ Z3 | Mature but requires system Z3 |
| `rsmt2` | Process wrapper for Z3/CVC5 | Needs solver binary in PATH |
| `smtlib` | Process wrapper (solver-agnostic) | Needs solver binary in PATH |
| CreuSAT | Formally verified SAT solver | SAT only, not SMT |
| Chalk | Prolog-like trait solver | Rust compiler's type inference |
| Polonius | Datalog borrow checker | Constraint-based analysis |

## References

- [OxiZ GitHub](https://github.com/cool-japan/oxiz)
- [OxiZ crates.io](https://crates.io/crates/oxiz)
- [OxiZ announcement](https://users.rust-lang.org/t/announce-oxiz-0-1-1-a-pure-rust-smt-solver-aiming-for-z3-compatibility/137541)
- [InferType (ECOOP 2024)](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2024.23)
- [Chalk](https://github.com/rust-lang/chalk)
- [Polonius](https://rust-lang.github.io/polonius/rules/relations.html)
