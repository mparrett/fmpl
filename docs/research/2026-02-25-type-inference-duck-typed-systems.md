# Type Inference for Duck-Typed Systems

## Overview

Research into type inference approaches for dynamically-typed, prototype-based, duck-typed languages. FMPL needs a type system that respects its dynamic nature while providing useful static guarantees -- particularly leveraging its grammar system as a source of type information.

## The Landscape

There are roughly 12 distinct approaches to adding type information to duck-typed systems. They range from "prove type errors" (success typing) to "prove type safety" (full dependent types), with various pragmatic middle grounds.

---

## 1. Success Typing (Dialyzer)

**Philosophy**: Instead of proving type safety, prove type errors. Only warn when something is GUARANTEED to fail at runtime.

**Algorithm** (Lindahl & Sagonas, PPDP 2006):
1. Optimistic initialization -- assume all functions accept/return anything
2. Generate subtyping constraints from program constructs
3. Forward and backward constraint propagation to fixed point
4. Report only when constraints become contradictory (inconsistent = guaranteed error)

**Key property**: Zero false positives. Every warning points to a real bug.

**Used by**: Erlang (Dialyzer), Elixir (before set-theoretic types)

**Fit for FMPL**: **Highly recommended as the starting point.** Requires zero annotations, never rejects valid programs, catches real bugs. The grammar system provides additional constraints that make success typing more precise -- if data passes through a grammar rule, its type is constrained by the grammar's output pattern.

**References**:
- [Practical Type Inference Based on Success Typings](https://www.semanticscholar.org/paper/Practical-type-inference-based-on-success-typings-Lindahl-Sagonas/76afb76ccd0b98f9eeea2b4104072e467a83e8aa)
- [Learn You Some Erlang: Dialyzer](https://learnyousomeerlang.com/dialyzer)

---

## 2. Row Polymorphism (Extensible Records)

**Philosophy**: Model "this object must have at least these fields" without closing the type. An open record type `{name: String, age: Int | r}` means "has at least name and age, plus whatever `r` is."

**Algorithm** (Wand 1987, Remy 1989):
- Extend Damas-Hindley-Milner unification with row variables
- When unifying two records: common fields unify their types, non-common fields accumulate into row variables
- Standard let-generalization handles row variable polymorphism naturally
- Remy's extension adds presence/absence flags (`Pre τ` / `Abs`) for the "distinct labels" problem

**Used by**: OCaml (object types, polymorphic variants), PureScript, Elm (extensible records), Links, Koka (for algebraic effects)

**Fit for FMPL**: **One of the best fits for the core type system.** Row polymorphism directly models prototype-based objects where:
- Objects can have arbitrary slots
- Functions typically access only specific slots
- Grammar rules produce values with known structure

Inferred type: "this function takes an object with at least a `name` slot (String) and a `respond_to` slot (Function), and possibly other slots."

```fmpl
-- Given:
let greet = \obj -> obj.name ++ " says hello"

-- Inferred type:
-- greet : {name: String | r} -> String
```

**References**:
- [Row Polymorphism - Wikipedia](https://en.wikipedia.org/wiki/Row_polymorphism)
- [Adding Row Polymorphism to Damas-Hindley-Milner](https://bernsteinbear.com/blog/row-poly/)
- [Extensible Records with Scoped Labels (Leijen)](https://www.cs.ioc.ee/tfp-icfp-gpce05/tfp-proc/21num.pdf)

---

## 3. Occurrence Typing (Typed Racket)

**Philosophy**: Types depend on control flow predicates. `(if (number? x) ...)` narrows the type of `x` to `Number` in the true branch.

**Algorithm** (Tobin-Hochstadt & Felleisen, POPL 2008):
- Every predicate carries propositions about what's true when it returns true/false
- Track logical propositions about variables through the control flow
- In conditional branches, add propositions from the predicate to the environment
- User-defined predicates can carry custom propositions

**Key insight**: Grammar rules serve as type-refining predicates. If `value @ grammar.rule` succeeds, the type of `value` is narrowed to the grammar's output type.

**Used by**: Typed Racket

**Extended**: "Occurrence Typing Modulo Theories" (Kent et al., PLDI 2016) adds SMT-backed reasoning. Verified safe vector access in 50% of cases with no annotations.

**Fit for FMPL**: **Very strong fit.** FMPL's `@` operator is essentially predicate-based branching:

```fmpl
x @ {
  :Int(n)        => ...  -- x is narrowed to Int in this arm
  :String(s)     => ...  -- x is narrowed to String in this arm
  %{name: n}     => ...  -- x is narrowed to {name: _ | r} in this arm
}
```

Each pattern arm carries implicit propositions. Grammar parse success is a theory-specific predicate that narrows types.

**References**:
- [Occurrence Typing - Typed Racket Guide](https://docs.racket-lang.org/ts-guide/occurrence-typing.html)
- [The Design and Implementation of Typed Scheme](https://www2.ccs.neu.edu/racket/pubs/popl08-thf.pdf)
- [Occurrence Typing Modulo Theories](https://arxiv.org/abs/1511.07033)

---

## 4. Algebraic Subtyping / MLsub

**Philosophy**: Combine subtyping with ML-style parametric polymorphism using set-theoretic types (union, intersection). Keep input/output type polarity strictly separated.

**Algorithm** (Dolan & Mycroft, POPL 2017; Parreaux "Simple-sub", ICFP 2020):
- Strict polarity separation: input types vs output types
- Biunification extends standard unification for subtyping constraints
- Type simplification via connections to regular language algebra
- Simple-sub: implementable in under 500 lines of code

**Key innovation**: MLstruct (Parreaux, 2024) extends this with tagged records supporting extensible variants without row variables, using Boolean-algebraic subtyping (union, intersection, negation, recursive types).

**Used by**: MLsub (research), MLstruct (research), influencing Elixir's set-theoretic types

**Fit for FMPL**: **Strong fit.** FMPL's tagged data (`:Binary(:+, :Int(1), :Int(2))`) maps directly to MLstruct's tagged records. Union/intersection types naturally model dynamic dispatch and pattern matching.

**References**:
- [Polymorphism, Subtyping, and Type Inference in MLsub](https://dl.acm.org/doi/10.1145/3009837.3009882)
- [The Simple Essence of Algebraic Subtyping](https://lptk.github.io/programming/2020/03/26/demystifying-mlsub.html)
- [Boolean-Algebraic Subtyping](https://lptk.github.io/files/boolean-algebraic-subtyping.pdf)

---

## 5. Gradual Set-Theoretic Types (Elixir's Approach)

**Philosophy**: Combine gradual typing (typed and untyped code coexist) with set-theoretic types (union, intersection, negation). Based on work by Giuseppe Castagna and Guillaume Duboc.

**How it works**:
- Types are sets of values. Subtyping is set inclusion.
- Union (`A | B`), intersection (`A & B`), negation (`not A`) are first-class.
- A "dynamic" type (`?`) represents untyped code.
- The consistency relation governs the boundary between typed and untyped code.
- BDD (Binary Decision Diagram) representation for efficient type operations.

**Used by**: Elixir (in active development, 2024-2026)

**Fit for FMPL**: **Strong fit for the full type system.** Elixir faces similar challenges (dynamic BEAM language, pattern matching, tagged tuples). Their approach addresses:
- Optional annotations (gradual)
- Pattern matching produces type narrowing
- Guards refine types
- Tagged tuples (`:ok`/`:error`) get precise types

**References**:
- [The Design Principles of the Elixir Type System](https://arxiv.org/abs/2306.06391)
- [Gradual Set-Theoretic Types - Elixir Documentation](https://hexdocs.pm/elixir/gradual-set-theoretic-types.html)
- [Lazier BDDs for Set-Theoretic Types](http://elixir-lang.org/blog/2025/12/02/lazier-bdds-for-set-theoretic-types/)

---

## 6. Cartesian Product Algorithm (CPA)

**Philosophy**: Infer concrete types for prototype-based languages by analyzing all possible receiver/argument type combinations at each call site.

**Algorithm** (Agesen, ECOOP 1995):
- For each call site, compute the Cartesian product of all possible types for each argument
- Analyze each combination independently
- Merge results to get the return type
- Iterate to fixed point

**Designed for**: Self language (prototype-based, like FMPL)

**Key insight**: CPA handles prototype-based dispatch naturally because it tracks which concrete objects flow to each call site, which determines which methods are invoked.

**Fit for FMPL**: **Directly applicable.** CPA was designed for Self, which has the same prototype-based object model as FMPL. It handles:
- Dynamic dispatch (which method is called depends on the receiver's concrete type)
- Prototype chain lookup
- Multiple possible types at a call site

**References**:
- [The Cartesian Product Algorithm (ECOOP 1995)](https://link.springer.com/chapter/10.1007/3-540-49538-X_2)
- [Concrete Type Inference - Agesen PhD Thesis](https://www.semanticscholar.org/paper/Concrete-type-inference-Agesen/30fa59dcb69466a7abb42999ca5ba594b3f79c01)
- [Type Inference of Self](https://bibliography.selflanguage.org/type-inference.html)

---

## 7. Flow's Constraint-Based Abstract Interpretation

**Philosophy**: Generate constraints from the entire program, propagate them to closed form, detect inconsistencies. Flow-sensitive and path-sensitive for precision.

**Algorithm** (Chaudhuri et al., OOPSLA 2017):
- Constraint generation: uses of values generate subtyping constraints
- Constraint propagation: corresponds to subtyping
- Flow-sensitive: variables' types tracked through control flow
- Path-sensitive: runtime tests refine types in branches
- Parallel and incremental: persistent server, modular analysis

**Key architecture**: Persistent server maintains semantic info for entire codebase. Incremental re-analysis on file changes. Sub-second query responses on millions of lines.

**Used by**: Facebook Flow (JavaScript)

**Fit for FMPL**: **Excellent fit for architecture.** The persistent server model fits FMPL's "live image" philosophy. Flow-sensitivity handles `@` pattern matching naturally. Incrementality is critical for REPL and IDE use cases.

**References**:
- [Fast and Precise Type Checking for JavaScript (OOPSLA 2017)](https://arxiv.org/abs/1708.08021)
- [Flow: A Static Type Checker for JavaScript](https://flow.org/)

---

## 8. Structural Typing and Protocol Inference

**Philosophy**: Types defined by structure (what methods/fields an object has), not by name.

**Key systems**:
- **TypeScript**: Structural compatibility checked at compile time
- **Go**: Implicit interface satisfaction (structural)
- **OCaml structural objects**: Infer required methods from usage. `obj#speak` infers `< speak : unit; .. >`
- **Python PEP 544 Protocols**: Explicit protocol declarations, structural conformance
- **Sorbet (Ruby)**: Structural interfaces via `T::Interface`

**Automatic protocol inference**: No mainstream language does fully automatic protocol inference, but OCaml's structural objects come closest. The Palsberg-Schwartzbach algorithm (1991) infers required interfaces from message sends.

**Fit for FMPL**: **Critical capability.** In a prototype-based language, the "interface" is the set of messages an object responds to. Combined with row polymorphism, grammar rules could serve as reified protocol definitions.

**References**:
- [Structural type system - Wikipedia](https://en.wikipedia.org/wiki/Structural_type_system)
- [TypeScript: Type Compatibility](https://www.typescriptlang.org/docs/handbook/type-compatibility.html)
- [PEP 544: Protocols](https://peps.python.org/pep-0544/)

---

## 9. Gradual Typing (Siek & Taha)

**Philosophy**: Allow typed and untyped code to coexist and interact safely.

**Key concepts**:
- The `?` (dynamic) type is compatible with any other type
- A consistency relation (not subtyping) governs type/untype boundaries
- Runtime casts inserted at boundaries to maintain safety
- Blame tracking identifies which side of a boundary caused a failure

**Key property**: The Gradual Guarantee -- adding type annotations never changes program behavior (only adds earlier error detection).

**Used by**: TypeScript (unsound), Python/mypy (mostly sound), Dart (sound), Typed Racket (fully sound)

**Fit for FMPL**: **Essential for adoption.** FMPL must support untyped code (existing MOO-heritage patterns) alongside typed code (new safety-critical patterns). Gradual typing provides the migration path.

**References**:
- [Gradual Typing for Functional Languages (Siek)](http://scheme2006.cs.uchicago.edu/13-siek.pdf)
- [Refined Criteria for Gradual Typing (SNAPL 2015)](https://drops.dagstuhl.de/storage/00lipics/lipics-vol032-snapl2015/LIPIcs.SNAPL.2015.274/LIPIcs.SNAPL.2015.274.pdf)

---

## 10. StrongTalk / Typed Smalltalk

**Philosophy**: Add an optional type system to a dynamic language without changing the language's semantics.

**Key design** (Bracha & Griswold, OOPSLA 1993):
- Types are pluggable -- they do not affect runtime behavior
- Type system is a separate tool that checks existing code
- Supports both nominal and structural subtyping
- "Branded types" for nominal distinctions on structural types

**Used by**: StrongTalk (Smalltalk with types), influenced Dart

**Fit for FMPL**: Validates that optional typing can work for prototype-based languages without changing semantics.

**References**:
- [Strongtalk: Typechecking Smalltalk in a Production Environment](https://bracha.org/oopsla93.pdf)
- [The Strongtalk Type System for Smalltalk](https://bracha.org/nwst.html)

---

## 11. Stanza's Optional Type System

**Philosophy**: A practical optional type system for a multi-paradigm language with object-oriented and functional features.

**Key features**:
- Bivariant type parameters (subtyping on type arguments)
- Both nominal and structural subtyping
- Gradual typing with `?` type
- Designed for a language with mixins and multiple inheritance

**Fit for FMPL**: Directly relevant as Stanza is another language combining dynamic and static typing with prototype-like features.

**References**:
- [The Design of Stanza's Optional Type System](https://lbstanza.org/optional_typing.html)

---

## 12. SMT-Based Type Inference

**Philosophy**: Encode type constraints as SMT formulas, use a solver to find satisfying type assignments.

**Encoding strategies**:
- `has_method(obj_type, "X")` as boolean predicate for duck typing
- Refinement types: `{x: Int | x > 0}` with arithmetic handled by SMT
- Hybrid: HM for base types, SMT for refinement predicates

**Used by**: F*, Liquid Haskell, research systems

**Fit for FMPL**: **Useful for specific features, not the primary approach.** SMT is overkill for basic type checking but valuable for:
- Pattern match exhaustiveness checking
- Grammar rule property verification
- Numeric constraints in patterns
- Future refinement types

**References**:
- [SMT-Based Static Type Inference for Python 3 (ETH Zurich)](https://ethz.ch/content/dam/ethz/special-interest/infk/chair-program-method/pm/documents/Education/Theses/Mostafa_Hassan_BA_report.pdf)
- [OxiZ: Pure Rust SMT Solver](https://github.com/cool-japan/oxiz)

---

## Recommended Layered Approach for FMPL

### Layer 1: Success Typing (Immediate Value)
Start with Dialyzer-style success typing. Zero annotations, zero false positives, catches guaranteed failures. Quickest path to useful error detection.

### Layer 2: Row-Polymorphic Structural Inference (Core Type System)
Row polymorphism for prototype-based objects. Models "object must have at least these slots" with standard HM inference. Row types are the structural analog of duck-typed objects.

### Layer 3: Occurrence Typing with Grammar Integration (FMPL's Unique Advantage)
Pattern matching via `@` narrows types in each arm. Grammar rules serve as type-refining predicates. This leverages FMPL's grammar-centric design for type-level guarantees no other language has.

### Layer 4: Gradual/Set-Theoretic Types (Full System)
Following Elixir's approach (Castagna/Duboc), add optional type annotations using set-theoretic types (union, intersection, negation). The consistency relation governs typed/untyped boundaries.

### Layer 5 (Future): Refinement Types via SMT
SMT-backed refinement types for specific domains (numeric constraints, grammar properties) following "occurrence typing modulo theories."

### Key Algorithms to Study for Implementation

1. **Simple-sub** (Parreaux) -- algebraic subtyping inference in <500 lines
2. **CPA** (Agesen) -- for prototype-based dispatch inference
3. **Row unification** (Remy) -- for structural record/object types
4. **Flow's constraint propagation** -- for incremental, modular analysis
5. **Dialyzer's success typing** -- for the "no false positives" foundation

### FMPL's Unique Advantage: Grammars as Type Predicates

No other language has FMPL's grammar system integrated with its type inference. Grammars provide structural guarantees that feed directly into the type system:

```fmpl
-- Grammar rule defines a structural type
grammar json <: base::parser {
  object = "{" (pair ("," pair)*)? "}" => %{...}
  pair   = string ":" value => [key, val]
}

-- Parsing through the grammar narrows the type
let data = input @ json.object
-- data is now typed as: {[String]: JsonValue}
-- No annotation needed -- the grammar IS the type declaration
```

This is occurrence typing where grammars are the predicates. It's the natural synthesis of FMPL's two core innovations (grammars + `@` operator) with type theory.

## References

Full bibliography organized by approach -- see inline references above.
