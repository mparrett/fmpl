# FMPL Design Principles

Durable invariants for the FMPL project. These are NOT feature requirements,
NOT roadmap items, and NOT decomposable tasks. They are constraints that
shape every decision and override iteration scope when in conflict. If you
are about to make a change that violates one of these principles, stop —
the principle wins, the change needs to be reframed.

Each principle has:
- **Statement** — what the principle requires
- **Reason** — why this principle exists (origin, motivation, what it prevents)
- **Implication** — concrete consequences that fall out of the principle
- **Origin** — where the decision was made (conversation date if known)

---

## DESIGN-001 — Metacircular bootstrap

**Statement.** The Rust-implemented parser is a stage-0 bootstrap mechanism
only. The end-state parser is `lib/core/fmpl_parser.fmpl` (written in FMPL,
applied via the FMPL grammar runtime). The `parse` builtin should eventually
load the FMPL parser, apply it to source, and return AST as canonical values.

**Reason.** FMPL is metacircular by design — the language describes its own
compilation pipeline. Anything that keeps Rust as a permanent dependency
breaks self-hosting. Rust artifacts that have no FMPL equivalent are bugs,
not features.

**Implication.**
- The Rust `parser.rs` and the FMPL `fmpl_parser.fmpl` describe the same
  language. Any change to one must be reflected in the other.
- The Rust `grammar/parser.rs` (grammar DSL parser) and the FMPL grammar
  parser are *the same parser* in the end-state. They are separate today
  only because the bootstrap isn't complete.
- Iterations that delete Rust-side surface (AST variants, pattern variants,
  parser logic) are progress toward bootstrap completion, not feature loss.
- `fmpl-bootstrap` retains the Rust compiler as a stage-0 fallback; the main
  pipeline is FMPL.

**Origin.** Discussed 2026-01-28 with the foundational design decision to
mirror bytecode instructions and use FMPL grammar for `parse`. User said:
"Parse should probably actually be using an fmpl grammar."

---

## DESIGN-002 — Single canonical form for structured data

**Statement.** Structured data uses one representation across the whole
system: list-shaped nodes `[:Tag, child1, child2, ...]` where `:Tag` is a
symbol head and the remaining elements are children. There is no separate
"tagged value" type, no `:Tag(args)` constructor syntax, and no
`Pattern::Constructor` distinct from list-pattern matching.

**Reason.** A second representation would force every consumer
(printer, compiler, optimizer, VM, FMPL stdlib, FMPL parser) to either
support both forms or convert between them. That doubles surface area and
introduces representation-mismatch bugs at every seam. Originally the
project used `Value::Tagged` with `:Tag(args)` constructor syntax; the
2026-01-30 pivot to list-form was motivated by alignment with grammar
patterns (which were already list-shaped) and OMeta-JS conventions.

**Implication.**
- `Value::Tagged` deleted (done ITER-0004b).
- `Expr::Tagged`, `ast::Pattern::Constructor`, `pattern::Pattern::Tagged`,
  `pattern::Pattern::TagMatch` to be deleted (ITER-0004d).
- The `:Tag(args)` syntax must be rejected by BOTH the FMPL expression
  parser AND the grammar pattern parser (they accept the same language —
  see DESIGN-001).
- FMPL stdlib uses `[:Tag, ...]` exclusively. Any stdlib file using
  `:Tag(args)` is legacy and migration debt.
- The diagnostics gate `no_legacy_fmpl_syntax` is the enforcement mechanism.

**Origin.** Pivot decision recorded 2026-01-30. User said: "Switch from
using :Tag() to [:symb, ...]". Prior design (2026-01-28) used `:Tag(args)`;
the list-form pivot superseded that decision and is the current canonical form.

---

## DESIGN-003 — Symbols for type and node names

**Statement.** Type names, node names, and tags in structured data are
symbols (`:Binary`, `:Int`, `:Call`), not strings (`"Binary"`, `"Int"`).
Symbols are interned (`SmolStr`) and compared by identity.

**Reason.** Symbols are faster to compare than strings, are semantically
clearer (a `:Binary` head distinguishes a tagged value from a string-keyed
map), and align with OMeta/Ohm conventions where rule names and tag heads
are symbols.

**Implication.**
- Pattern matching against tags uses symbol equality, not string equality.
- The compiler's `MatchTag` instruction takes a symbol index, not a string.
- Stdlib introspection helpers return symbols for tags, not strings.

**Origin.** Decided 2026-01-28. User said: "I'd prefer symbols for the
various type names and node names, rather than arbitrary strings."

---

## DESIGN-004 — Tree-based IR with named temporaries (hybrid B+C)

**Statement.** The IR is tree-shaped (subexpressions nest naturally) and
the `compile` builtin linearizes the tree into Indexed RPN bytecode. When
sharing or sequencing is needed, named temporaries via `:Let` are used:
`:Let(:x, expr1, body)` introduces `:x` for use in `body`.

**Reason.** A pure stack IR forces hand-written grammars to track
instruction indices manually — error-prone. A pure tree IR can't express
sharing or control flow elegantly. The hybrid keeps trees simple for
expressions and uses explicit names only when sharing or sequencing is
required.

**Implication.**
- AST→IR grammar (`lib/core/ast_to_ir.fmpl`) produces tree-shaped IR with
  let-bindings; it does not produce instruction indices.
- The `compile` builtin walks the tree, allocates indices, tracks let
  scopes, and emits Indexed RPN.
- The user-written transformation grammar never sees instruction indices.

**Origin.** Decided 2026-01-28. User said: "I'm torn between B and C" then
"yes, that looks better" to a hybrid B+C proposal.

---

## DESIGN-005 — Grammar inheritance via shared semantic actions (deferred)

**Statement.** Grammar inheritance (`<:`) is on the roadmap but not yet
implemented. Until then, each optimization grammar duplicates the full
recursive traversal with its own specific rules at the top.

**Reason.** OMeta supports grammar inheritance for code reuse across
optimizer passes. FMPL plans the same. Until inheritance lands, the
workaround is duplication — accepted technical debt.

**Implication.**
- `lib/core/grammar_optimizer.fmpl` has near-duplicate traversal across
  `null_opt`, `associative_opt`, `empty_elim_opt`, `jump_table_opt`.
- Do NOT factor these by hand; the abstraction is grammar inheritance,
  not a code-level helper.
- An iteration to implement `<:` will eliminate the duplication.

**Origin.** Captured as a `NOTE` in `lib/core/grammar_optimizer.fmpl:18-19`.

---

## How to use this document

- **At session start.** Load this file. It is short by design — every line
  should be load-bearing.
- **At iteration planning.** Every iteration scope review should explicitly
  identify which principles the iteration depends on or affects. If an
  iteration's scope contradicts a principle, the principle wins and the
  scope is revised.
- **At task execution.** When task scope is ambiguous, consult the
  principles. If T6 says "reject :Tag(args) in the parser," DESIGN-001 +
  DESIGN-002 together tell you BOTH parsers need the rejection.
- **At PAR review.** Reviewers should flag any iteration or task that
  silently violates a principle. A principle violation is a higher-severity
  finding than a citation error or scope creep.
- **When updating this file.** Only add principles that are durable
  invariants. Roadmap entries, story cards, and scenario specs go elsewhere.
  When the project's foundational direction changes (e.g., the 2026-01-30
  pivot from `:Tag(args)` to `[:Tag, ...]`), supersede the old principle
  rather than amending it — keep the history visible.

