# Category-Theoretic Type System for FMPL

## Overview

Research into using category theory, algebraic laws, coalgebraic semantics, and structural typeclass inference to build FMPL's type system. The core insight: facets are named categories with algebraic laws, objects are coalgebras defined by observable behavior, and type inference proceeds by contradiction detection (success typing) rather than type assignment.

## Design Philosophy

1. **Image-based, interactive** -- like Self, Smalltalk, Common Lisp. Laws and types live in the image, not in source annotations
2. **No explicit typing** -- types inferred from usage, fail only on guaranteed contradictions
3. **Operations are morphisms** -- `+` means "supports combine" (Semigroup-like), not "is a number"
4. **Facets are named categories** -- member lists with optional arity and unification variables; laws discovered automatically
5. **Objects are coalgebras** -- defined by observable behavior, not declared type
6. **Success typing foundation** -- zero false positives, zero annotations required
7. **Inspector over annotations** -- algebraic properties reported in tooling, not declared in syntax

---

## 1. Typeclasses for Dynamic Languages

### 1.1 Clojure Protocols

Clojure protocols provide named sets of functions dispatching on the first argument's type. They solve the expression problem (extending both types and operations) without requiring modification of existing code. Key insight: protocols are structurally defined but nominally dispatched.

### 1.2 Cecil/Diesel Predicate Classes

Craig Chambers' Cecil and its successor Diesel introduce **predicate classes**: objects are automatically classified into categories based on satisfying predicates -- both structural (has these methods) and behavioral (satisfies these properties). This is the closest existing system to what FMPL needs.

**Key insight**: An object doesn't declare "I am a Monoid." The system observes that it has `combine` and `empty`, tests associativity and identity laws, and classifies it as a Monoid automatically.

### 1.3 Julia's Multiple Dispatch

Julia's type system is built around multiple dispatch with abstract type hierarchies. Functions dispatch on the concrete types of ALL arguments (not just the receiver). This enables `+(::Int, ::Int)` and `+(::String, ::String)` as separate methods of the same function -- operations as morphisms in different categories.

### 1.4 CLOS and Dylan

Common Lisp's CLOS and Dylan provide generic functions with multiple dispatch, method combinations, and the MOP (Meta-Object Protocol). Dylan adds sealed domains for optimization and `limited` types for constrained generics.

---

## 2. Algebraic Law Inference

### 2.1 QuickSpec: Automatic Law Discovery

QuickSpec (Smallbone et al.) automatically discovers algebraic laws from Haskell code by:
1. Enumerating well-typed terms up to a given size
2. Testing candidate equations using QuickCheck
3. Pruning redundant laws to find a minimal equational theory

Given `reverse`, `++`, and `[]`, QuickSpec discovers:
- `reverse (reverse xs) == xs` (involution)
- `reverse [] == []` (identity case)
- `reverse (xs ++ ys) == reverse ys ++ reverse xs` (anti-homomorphism)

### 2.2 Speculate: Conjecturing Laws from Testing

Speculate (Braquehais) extends QuickSpec's approach with LeanCheck for enumeration-based testing. It can discover laws about ADTs by testing all small instances.

### 2.3 Propel: Verification of Algebraic Laws (PLDI 2024)

Propel (Zakhour, Weisenburger, Salvaneschi, PLDI 2024) is a specialized verifier for algebraic laws (commutativity, associativity, idempotence). Key innovations:
- Conjectures auxiliary properties
- Reasons about both equalities **and inequalities**
- Verified against 142 instances across 5 verifiers
- Application domains include CRDTs and data flow engines

### 2.4 Stainless: Algebraic Laws in Scala

Stainless (EPFL) allows `@law` annotations on methods in abstract classes. When concrete implementations are provided, Stainless **automatically verifies** that laws hold using SMT solvers.

### 2.5 Recommended Pipeline for FMPL

1. **Detect candidate operations** from duck-typed usage
2. **Test for algebraic laws** using QuickSpec-style enumeration
3. **Verify** using SMT (OxiZ) or property-based testing
4. **Classify** into known algebraic structures (Semigroup, Monoid, etc.)

Steps 1 and 4 are novel; steps 2 and 3 have well-established implementations.

---

## 3. Category Theory in PL Design

### 3.1 The Algebraic Hierarchy

The standard hierarchy from category theory / abstract algebra:

```
Semigroup (associative binary operation)
  |
Monoid (+ identity element)
  |
Group (+ inverse)

Functor (structure-preserving map)
  |
Applicative (+ lifting of multi-argument functions)
  |
Monad (+ sequential composition)
```

### 3.2 PureScript's Fine-Grained Splitting

PureScript splits typeclasses along precise category-theoretic lines:
- `Semigroupoid` (composition without identity) vs `Category` (composition with identity)
- `Semigroup -> Monoid`, `Semiring -> Ring -> CommutativeRing -> EuclideanRing -> DivisionRing -> Field`

Key insight: `Tuple` is a `Semigroupoid` but **not** a `Category` (no identity). This fine granularity reveals real mathematical structure that coarser systems miss.

### 3.3 Idris: Verified Algebraic Interfaces

Idris encodes laws directly in interfaces via dependent types. `VerifiedSemigroup` requires a proof of `semigroupOpIsAssociative`. However, these remain in `contrib` because Idris lacks **function extensionality** -- functions cannot be proved equal even when they produce identical outputs.

**Key insight for FMPL**: Testing-based verification (QuickSpec/QuickCheck style) sidesteps the function extensionality problem. You cannot *prove* `map id = id` in general, but you can *test* it extensively and flag contradictions.

### 3.4 F-Algebras and Catamorphisms

F-algebras provide the category-theoretic foundation for recursive data types. A type `Algebra f a = f a -> a` represents an evaluation strategy; a catamorphism (`cata`) is the unique homomorphism from the initial algebra.

**For FMPL**: Pattern matching with `@` is essentially a catamorphism. The grammar system's recursive descent parsing produces initial algebras, and `@` blocks define algebras for folding over them.

---

## 4. Structural Typeclass Inference

### 4.1 Agesen's Cartesian Product Algorithm

CPA (ECOOP 1995) infers concrete types for prototype-based languages:
- "Concrete types should not be obtained from explicit type declarations because their presence limits polymorphism unacceptably"
- Each call site tracks which concrete types flow through it
- The cartesian product of all argument types determines which methods may be invoked
- Designed for Self (prototype-based), adapted for Python and other dynamic languages

**Key insight**: CPA shows that type inference for prototype-based languages without declarations is feasible. FMPL's inference should use CPA-style flow analysis to determine which operations each value supports, then classify the resulting operation sets as algebraic structures.

### 4.2 OCaml Modular Implicits

OCaml's modular implicits use **structural signature matching** for typeclass-like resolution. Rather than Haskell's nominal matching ("find the `Monoid Int` instance"), OCaml matches module signatures structurally ("find any module that provides `combine` and `empty` with the right types").

**Key insight**: Structural resolution is exactly what FMPL needs. An object that has `combine` and `empty` with the right behavior should be recognized as a Monoid regardless of whether it declares itself as one.

### 4.3 "Scrap Your Typeclasses" -- Dictionary Passing

Typeclasses are syntactic sugar for dictionary passing. The key difference: typeclasses assume **one canonical instance per type** (coherence), while explicit dictionaries allow multiple instances.

**For FMPL**: Since objects are duck-typed, there's no global coherence. An object may be a Monoid under addition AND a Monoid under multiplication. The inference system should detect **all** valid algebraic structures, not assume one canonical structure.

### 4.4 GHC's Constraint Solver

GHC's OutsideIn(X) constraint solver uses a **work list** and **inert set**. Constraints start non-canonical and are progressively canonicalized. A constraint-based approach could work for FMPL: each operation generates constraints, the solver collects them and tests whether they form known algebraic structures.

---

## 5. Algebraic Effects and Facets

### 5.1 Koka: Row-Polymorphic Effect Types

Koka's effect types are row types tracking side effects in function signatures. The key connection to FMPL: **effects model "what operations are available"** in the same way facets do:

- An effect row `<State, IO>` says "this computation may use State and IO operations"
- A facet `[view_balance, deposit]` says "this view provides view_balance and deposit operations"

The mathematical structure is the same: both are **row types** over sets of operations.

### 5.2 Frank: Ambient Abilities

Frank's "abilities" use a novel form of effect polymorphism that avoids mentioning effect variables. The empty ability means "ability polymorphic." Relevant because FMPL objects with no declared facet could be "facet polymorphic" (supports whatever operations it has).

### 5.3 Rows and Capabilities as Modal Effects (POPL 2026)

A recent paper establishes a **unified framework** relating row-based and capability-based effect systems. The key insight: both row polymorphism and capabilities track "what operations are available," but through different mechanisms. Modal effect types decouple effect tracking from functions.

**Key insight for FMPL**: This paper directly relates FMPL's two central mechanisms -- facets (capabilities) and the set of operations an object supports (rows). Formally, facets are capabilities in a row-polymorphic effect system where the "effects" are the object's methods.

---

## 6. Coalgebraic Semantics for Objects

### 6.1 Rutten's Universal Coalgebra

The three basic notions of universal algebra -- algebra, homomorphism, congruence -- correspond dually to **coalgebra, homomorphism of coalgebras, bisimulation**. An F-coalgebra `(S, alpha_S : S -> F(S))` is a set of states with transition structure.

For objects: each object is a state, `F` describes observable behavior (methods and returns). Two objects are **bisimilar** if they produce the same observations for all sequences of method calls -- this is precisely behavioral equivalence / duck typing.

### 6.2 Jacobs: Objects and Classes, Co-algebraically

Bart Jacobs models classes as coalgebras where the functor `F` describes the interface:
- **Observers/selectors** yield attribute values (coalgebraic "destructors")
- **Mutators** take parameters and yield new states
- **Refinement** captures behavioral subtyping

An object's coalgebra factorizes into its methods:
```
F(X) = A x (B -> X) x (C -> D x X) x ...
```
where `A` is an attribute type, `B -> X` is a mutator, `C -> D x X` is a method taking `C` and returning `D` while updating state.

### 6.3 Abadi-Cardelli: A Theory of Objects

Foundational work developing object calculi where objects are primitives. Models self, dynamic dispatch, classes, inheritance, prototyping, subtyping, covariance, contravariance, and method specialization.

### 6.4 Prototypes as Fixed Points (Scheme Workshop 2021)

Models a prototype as a function of two arguments: `self` and `super`. Prototype composition is function composition. Object instantiation is a **fixed point** operation. This connects prototype-based OOP to:
- Fixed-point combinators (Y combinator for object self-reference)
- Mixin composition (algebraic property: associativity of mixin composition)
- The isomorphism between mutable objects (coalgebraic) and pure-functional state threads (algebraic/monadic)

---

## 7. MUD-Specific Algebraic Laws

### 7.1 Container Laws

- **Put-then-take identity**: `take(put(container, item)) == (container, item)` (modulo other items)
- **Commutativity of independent puts**: `put(put(c, a), b) == put(put(c, b), a)`
- **Capacity constraints** as a semilattice: available space forms a bounded join-semilattice

### 7.2 Movement Laws

- **Enter-then-leave identity**: `leave(enter(world, room)) == world` (if no side effects)
- **Path composition**: moving A->B then B->C equals A->C (associativity -- rooms are objects, paths are morphisms in a **category**)
- **Connected components** form an equivalence relation

### 7.3 Permission/Capability Composition

- Capabilities form a **lattice**: union is join, intersection is meet
- Facet restriction is a **meet** operation: `restrict(full_access, auditor_facet)` is meet
- Authority attenuation is monotone: cannot gain capabilities through delegation

### 7.4 Transaction Semantics

- **Rollback = identity**: committed then rolled-back transaction restores initial state
- **Serializable transactions** form a monoid: sequential composition is associative with empty transaction as identity
- **CRDTs** provide commutative, associative, idempotent merge -- a join-semilattice, exactly what distributed virtual worlds need for conflict-free state merging

### 7.5 Object-Capability Security Laws (Goblins/E)

- **Attenuation**: `attenuate(c, policy) <= c` in capability order
- **Confinement**: no capability obtained without explicit grant (no ambient authority)
- **Composition**: facet restrictions compose via meet/intersection

---

## 8. Synthesis: FMPL's Type System Design

### 8.1 Objects as Coalgebras

Each FMPL object is a coalgebra `(S, alpha : S -> F(S))` where `F` describes the interface:
```
F(X) = Method1_Return x (Method2_Arg -> X) x ...
```
Two objects with bisimilar behavior (same responses to same method sequences) are the same "type."

### 8.2 Facets as Subfunctors / Row Restrictions

A facet restricts the functor to a subset of observations -- a **natural transformation** from the full functor to a restricted one. In row-type terms:

```fmpl
-- Full object interface:
{ view_balance: () -> Int, deposit: Int -> (), withdraw: Int -> () }

-- Facet auditor (row restriction):
{ view_balance: () -> Int }
```

### 8.3 Facet Syntax: Three Levels of Specificity

Facets use the existing FMPL `.#facets` section syntax, extended with optional arity and unification variables. Three levels of specificity, scaling cognitive overhead with constraint complexity:

**Level 1: Slot names only** (arity inferred from usage)
```fmpl
.#facets
movable: [enter, leave]
auditor: [view_balance]
```

**Level 2: Slot names with arity** (explicit parameter count)
```fmpl
.#facets
movable: [enter(_), leave()]
auditor: [view_balance()]
container: [put(_), take()]
```

Parenthesized = method, bare = value slot. `_` means "one argument, don't care about relationships."

**Level 3: Unification variables** (cross-slot type relationships)
```fmpl
.#facets
combinable(T): [combine(T) -> T]
reducible(T): [combine(T) -> T, empty -> T]
container(T): [put(T), take() -> T]
mappable(A, B): [map(A -> B) -> Self(B)]
movable(R): [enter(R), leave() -> R]
```

Unification variables express **relationships**, not types. `combine(T) -> T` means "input and output must be the same kind of thing." The system infers what `T` actually is from usage.

**Design rationale** (vs C++ concepts): Same information content, but surfaced in an image-based inspector rather than compiler error messages. No template metaprogramming, no SFINAE, no header-file compilation model. Success typing means violations are reported as guaranteed contradictions, not type errors.

### 8.4 Laws: Discovered, Not Declared

**FMPL is an interactive, image-based language like Self, Smalltalk, and Common Lisp.** Laws are not source-level annotations -- they are properties the system discovers and reports in the inspector.

**Automatic discovery** (default): The runtime tests objects against known algebraic laws in the background:
1. Object has `combine/1` -- test associativity: `combine(combine(a, b), c) == combine(a, combine(b, c))`
2. Object has value slot `empty` -- test identity: `combine(empty, x) == x`
3. Report findings in the inspector

**Inspector view**:
```
treasury
  facets:
    auditor: [view_balance()]
      discovered laws:
        view_balance is idempotent
        view_balance is pure (no mutation)
    combinable(T): [combine(T) -> T]
      discovered laws:
        combine is associative
        combine is commutative
      classification: CommutativeMonoid (with empty)
```

**Explicit laws** (when needed): A lambda list in a slot, not a new syntax form:
```fmpl
combinable.laws: [
  \(a, b, c) -> a.combine(b).combine(c) == a.combine(b.combine(c))
]
```

Or attached via the inspector at runtime -- laws are objects in the image, not declarations in text files.

### 8.5 Success Typing for Constraint Collection

Following Dialyzer's approach:
1. Start optimistic: every value can be anything
2. Each usage generates constraints: `x.combine(y)` adds `HasMethod(x, combine, [typeof(y)])`
3. Unification variables in facets generate cross-slot constraints
4. Constraints narrow the possible types
5. Error only on guaranteed contradictions

### 8.6 The Classification Pipeline

```
Usage Analysis (CPA-style)              -> Set of operations each value supports
    |
    v
Constraint Collection (Dialyzer-style)  -> Constraints between operations
    |                                      (including facet unification variables)
    v
Law Testing (QuickSpec-style)           -> Discovered algebraic equations
    |
    v
Structure Classification               -> Named algebraic structures (Semigroup, Monoid, ...)
    |                                      Reported in inspector
    v
Contradiction Detection                -> Errors when value cannot inhabit required structure
```

This pipeline runs in the image as a background process, not as a compile-time pass. Results are live and update as objects change -- consistent with Self/Smalltalk's live programming model.

### 8.7 Row Polymorphism for Effect/Capability Tracking

Following Koka and the POPL 2026 "Rows and Capabilities" paper, facet sets tracked as row types internally. A function that accesses `view_balance` on its argument implicitly constrains that argument to `{ view_balance | r }` -- the system infers this, the programmer doesn't write it.

### 8.8 Multiple Algebraic Structures per Object

Since objects are duck-typed, there's no global coherence. An object may be:
- A Monoid under `+` (string concatenation)
- A Monoid under `*` (repetition)
- A Functor via `map`

The inference system detects **all** valid algebraic structures, not one canonical structure. Context determines which structure is in play. The inspector shows all discovered structures for a given object.

---

## Integration with Other Type System Layers

This category-theoretic approach integrates with the layered type system from the [type inference research](2026-02-25-type-inference-duck-typed-systems.md):

| Layer | Approach | Role of Category Theory |
|-------|----------|------------------------|
| 1. Success Typing | Dialyzer-style | Constraint collection and contradiction detection |
| 2. Row Polymorphism | Remy-style row unification | Facets as row restrictions / subfunctors |
| 3. Occurrence Typing | Grammar predicates | Grammars narrow coalgebraic observation type |
| 4. Algebraic Structure | This document | Law discovery, structure classification, facet laws |
| 5. SMT Refinement | OxiZ-backed | Law verification, exhaustiveness checking |

---

## References

### Typeclasses and Protocols
- [Clojure Protocols Reference](https://clojure.org/reference/protocols)
- [Cecil Language Specification (Chambers)](http://projectsweb.cs.washington.edu/research/projects/cecil/www/Internal/doc-cecil-lang/cecil-spec.pdf)
- [Diesel Language Specification (Chambers)](http://projectsweb.cs.washington.edu/research/projects/cecil/www/Release/doc-diesel-lang/diesel-spec.pdf)
- [Predicate Classes (Springer)](https://link.springer.com/chapter/10.1007/3-540-47910-4_15)
- [Julia Types Documentation](https://docs.julialang.org/en/v1/manual/types/)
- [CLOS Generic Functions - Practical Common Lisp](https://gigamonkeys.com/book/object-reorientation-generic-functions.html)

### Algebraic Law Inference
- [QuickSpec Paper (Smallbone)](https://smallbone.se/papers/quickspec.pdf)
- [QuickSpec GitHub](https://github.com/nick8325/quickspec)
- [Propel: Automated Verification of Algebraic Laws (PLDI 2024)](https://dl.acm.org/doi/10.1145/3656408)
- [Stainless: Specifying Algebraic Properties](https://epfl-lara.github.io/stainless/laws.html)

### Category Theory in PL Design
- [PureScript Semigroupoid](https://pursuit.purescript.org/packages/purescript-prelude/4.0.0/docs/Control.Semigroupoid)
- [Idris Verified Interfaces](https://github.com/idris-lang/Idris-dev/blob/master/libs/contrib/Interfaces/Verified.idr)
- [Catamorphisms and F-Algebras](https://medium.com/@olxc/catamorphisms-and-f-algebras-b4e91380d134)
- [Bartosz Milewski: Recursion Schemes for Higher Algebras](https://bartoszmilewski.com/2018/08/20/recursion-schemes-for-higher-algebras/)

### Structural Inference
- [Agesen: Cartesian Product Algorithm (ECOOP 1995)](https://link.springer.com/chapter/10.1007/3-540-49538-X_2)
- [OCaml Modular Implicits (arXiv)](https://arxiv.org/pdf/1512.01895)
- [Scrap Your Type Classes (Haskell for All)](https://www.haskellforall.com/2012/05/scrap-your-type-classes.html)
- [GHC Constraint Solver](https://github.com/sheaf/ghc-constraint-solver/blob/master/constraint_solving.md)

### Algebraic Effects
- [Koka: Row-Polymorphic Effect Types (arXiv)](https://arxiv.org/pdf/1406.2061)
- [Frank: Do Be Do Be Do (arXiv)](https://arxiv.org/pdf/1611.09259)
- [Rows and Capabilities as Modal Effects (POPL 2026)](https://popl26.sigplan.org/details/POPL-2026-popl-research-papers/34/Rows-and-Capabilities-as-Modal-Effects)

### Coalgebraic Semantics
- [Rutten: Universal Coalgebra (PDF)](https://www.cs.cornell.edu/courses/cs6861/2024sp/Handouts/Rutten.pdf)
- [Jacobs: Objects and Classes, Co-algebraically (ScienceDirect)](https://www.sciencedirect.com/science/article/pii/S1571066104000611)
- [Abadi-Cardelli: A Theory of Objects (Springer)](https://link.springer.com/book/10.1007/978-1-4419-8598-9)
- [Prototypes: Object-Orientation, Functionally (Scheme Workshop 2021)](http://webyrd.net/scheme_workshop_2021/scheme2021-final91.pdf)

### Behavioral Types
- [Foundations of Session Types and Behavioural Contracts (ACM)](https://dl.acm.org/doi/10.1145/2873052)
- [Parameterized Algebraic Protocols (PLDI 2023)](https://dl.acm.org/doi/10.1145/3591277)

### Object Capabilities
- [Goblins Documentation (Racket)](https://docs.racket-lang.org/goblins/intro.html)
- [Analysing Object-Capability Security (Oxford)](https://www.cs.ox.ac.uk/files/2690/AOCS.pdf)

### Success Typing
- [Dialyzer Official Documentation](https://www.erlang.org/doc/apps/dialyzer/dialyzer_chapter.html)
- [Practical Type Inference Based on Success Typings](https://www.semanticscholar.org/paper/Practical-type-inference-based-on-success-typings-Lindahl-Sagonas/76afb76ccd0b98f9eeea2b4104072e467a83e8aa)
