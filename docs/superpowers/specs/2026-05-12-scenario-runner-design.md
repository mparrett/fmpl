# Scenario Runner — Design Spec (ITER-0004d.4)

**Date:** 2026-05-12 (revised 2026-05-12 to add bootstrap-durability scope)
**Owner:** ITER-0004d.4
**Status:** Design — pending writing-plans
**Origin:** User feedback during ITER-0004d.1 T19 review on 2026-05-12. The per-scenario Rust test pattern in `fmpl-core/tests/structural_invariants.rs` is stylish but redundant against the scenario cards in `behavior-scenarios.md`. A cucumber / FitNesse-SLIM-style data-driven runner where the scenario card IS the source of truth would (a) make cards directly executable, (b) collapse per-test boilerplate, and (c) let future scenarios land as card-authoring tasks rather than test-writing tasks.

**Revision 2026-05-12 (post-design-review):** the user added a durability requirement — scenario cards must survive the bootstrap process. Specifically: when fmpl-bootstrap regenerates the parser from `lib/core/fmpl_parser.fmpl`, the same scenario corpus must re-execute against the regenerated parser and produce the same results. This is concrete and testable today; it positions scenarios as durable artifacts that ride the metacircular pipeline (DESIGN-001), not just one-off Rust tests. The original Rust runner stays; a sibling artifact `lib/tests/scenarios/scenarios.fmpl` is compiled from the markdown and consumed by the bootstrap pipeline for post-regeneration verification.

## Goal

Make `docs/superpowers/iterations/behavior-scenarios.md` directly executable, with a corpus that survives parser regeneration.

Two execution surfaces:

1. **Rust-side runner** — for fast pre-bootstrap test execution. Each scenario card carries enough structured data (action type, inputs, expectations) that a thin Rust driver dispatches each case to a step-definition and surfaces per-case pass/fail with line-span back-references into the markdown.
2. **FMPL-side runner** — for post-bootstrap durability verification. The same corpus, compiled to a list-shaped FMPL value at `lib/tests/scenarios/scenarios.fmpl`, is loaded by fmpl-bootstrap and re-executed against the regenerated parser. The two runners' results must agree case-for-case.

The first three consumers — SCENARIO-0104, SCENARIO-0105, SCENARIO-0106 (all from ITER-0004d.1) — migrate from `fmpl-core/tests/structural_invariants.rs` into the runner. That file is deleted once the runner covers the same evidence on both surfaces.

## Durability target

**Parser regeneration is the v1 durability target.** When `fmpl-bootstrap` rebuilds the parser from `lib/core/fmpl_parser.fmpl`, the scenario corpus runs against the regenerated parser and produces the same pass/fail outcomes as the source-tree (legacy) parser. A divergence means either (a) the regeneration introduced a behavior change or (b) one of the two runners has a bug; either way the gate catches it.

This is the only durability guarantee shipped in this iteration. Two later targets are explicitly out of scope but the architecture preserves room for them:

- **Self-compile cycle (ITER-0006).** The corpus validates that stage-N+1 of self-compile behaves identically to stage-N. Requires ITER-0006 to land first.
- **Fjall snapshot persistence (ITER-0005).** The corpus survives image serialization and restore cycles. The compiled `scenarios.fmpl` is a regular FMPL value, so when ITER-0005 lands, the Fjall snapshot machinery handles it like any other value with no scenario-specific work.

## Non-goals

- Migrating scenarios SCENARIO-0001..0077 (most have no step-def coverage today). Migration is opt-in.
- **A full FMPL-grammar-based markdown parser.** The v1 compilation step (markdown → FMPL artifact) runs in Rust (via `fmpl-core/build.rs`). A later iteration can replace it with an FMPL-side parser; the architecture is staged for that.
- Self-compile cycle durability (waits on ITER-0006).
- Fjall-snapshot durability (waits on ITER-0005, but is expected to work without additional scenario-specific work because `scenarios.fmpl` is a regular FMPL value).
- A TUI / visual reporter. `cargo test` output is sufficient.
- Parameterized fixture-style step-defs beyond what the three concrete consumers need.

## Architecture

### Components

```
fmpl-scenario-runner/                  ← new workspace crate (library)
  Cargo.toml                           ← deps: inventory (1.x)
  src/
    lib.rs                             ← re-exports public API
    corpus.rs                          ← markdown corpus parser
    fmpl_emit.rs                       ← compile Vec<Card> → list-shaped FMPL value
                                          (string form written to lib/tests/scenarios/)
    step_def.rs                        ← trait + inventory registry (Rust surface)
    error.rs                           ← StepError

fmpl-core/
  Cargo.toml                           ← [dev-dependencies] fmpl-scenario-runner
                                          [build-dependencies] fmpl-scenario-runner
  build.rs                             ← extended with TWO codegen outputs:
                                          (a) OUT_DIR/scenarios_generated.rs (Rust)
                                          (b) lib/tests/scenarios/scenarios.fmpl (FMPL)
  tests/
    scenario_runner.rs                 ← Rust-surface test target; include!s (a)
    scenario_runner_bootstrap.rs       ← FMPL-surface test target; drives bootstrap
                                          and runs the SAME corpus against the
                                          regenerated parser (loads (b) via io::load)
    steps/                             ← step-def impls (live with the test binary)
      mod.rs                           ← `pub mod parse_rejection; ...`
      parse_rejection.rs               ← struct ParseRejection; impl StepDef
      parse_success.rs                 ← struct ParseSuccess;   impl StepDef
      grep_invariant.rs                ← struct GrepInvariantAbsent;
                                          struct GrepInvariantPresent;
    common/
      comment_strip.rs                 ← moved from structural_invariants.rs;
                                          shared //-line-comment strip helper
    structural_invariants.rs           ← DELETED at iteration end

lib/tests/scenarios/                   ← NEW directory (sibling to lib/core/)
  scenarios.fmpl                       ← compiled corpus artifact (git-tracked).
                                          Read by fmpl-bootstrap for post-
                                          regeneration verification. Format:
                                          one list-shape value per card,
                                          [:Scenario, id, action_type, cases, span]
  dispatch.fmpl                        ← FMPL-side dispatcher (v1 stub: dispatches
                                          parse_rejection, parse_success only;
                                          grep_invariant stays Rust-only for now
                                          since io::read_dir doesn't yet exist)
```

### Public API (`fmpl-scenario-runner`)

```rust
// corpus.rs
pub fn parse_corpus(path: &Path) -> Result<Vec<Card>, CorpusError>;

pub struct Card {
    pub id: String,                    // "SCENARIO-0104"
    pub title: String,
    pub kind: Option<String>,          // "invariant" | "contract" | ...
    pub seam: Option<String>,          // "unit" | "integration" | ...
    pub action_type: Option<String>,   // default for cases without override
    pub cases: Vec<Case>,
    pub owning_stories: Vec<String>,
    pub sources: Vec<String>,
    pub line_start: usize,             // 1-based, inclusive
    pub line_end: usize,
}

pub struct Case {
    pub action: String,                // resolved action type (case override or card default)
    pub fields: BTreeMap<String, Value>,
    pub line_start: usize,
    pub line_end: usize,
}

pub enum Value {
    String(String),
    Bool(bool),
    Int(i64),
    List(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str>;
    pub fn as_bool(&self) -> Option<bool>;
    pub fn as_int(&self) -> Option<i64>;
    pub fn as_list(&self) -> Option<&[Value]>;
}

// step_def.rs
pub trait StepDef: Sync {
    fn action_type(&self) -> &'static str;
    fn run(&self, card: &Card, case: &Case) -> Result<(), StepError>;
}

pub struct StepDefRegistration(pub &'static dyn StepDef);

inventory::collect!(StepDefRegistration);

pub fn dispatch(card: &Card, case: &Case) -> Result<(), DispatchError>;
    // Walks inventory::iter::<StepDefRegistration>(), picks by action_type.
    // Returns DispatchError::Unknown if no step-def matches.
    // Returns DispatchError::Step(StepError) if the step-def returned Err.

// error.rs
pub struct StepError { pub message: String }
impl StepError { pub fn new(msg: impl Into<String>) -> Self }

pub enum DispatchError {
    Unknown(String),         // action_type not registered
    Step(StepError),
}

pub enum CorpusError {
    Io(std::io::Error),
    Malformed { line: usize, message: String },
    DuplicateId { id: String, first_line: usize, dup_line: usize },
}
```

### Test-binary glue

```rust
// fmpl-core/tests/scenario_runner.rs
mod steps;  // imports each step-def submodule so inventory::submit! is reachable

// The build.rs writes scenarios_generated.rs containing per-case #[test] fns
// plus a static SCENARIO_CORPUS for shared lookup.
include!(concat!(env!("OUT_DIR"), "/scenarios_generated.rs"));
```

```rust
// fmpl-core/tests/steps/mod.rs
pub mod parse_rejection;
pub mod parse_success;
pub mod grep_invariant;
```

```rust
// fmpl-core/tests/steps/parse_rejection.rs (sketch)
use fmpl_scenario_runner::{Card, Case, StepDef, StepDefRegistration, StepError};
use fmpl_core::lexer::Lexer;
use fmpl_core::parser::Parser;

pub struct ParseRejection;

impl StepDef for ParseRejection {
    fn action_type(&self) -> &'static str { "parse_rejection" }
    fn run(&self, _card: &Card, case: &Case) -> Result<(), StepError> {
        let source = case.fields.get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StepError::new("case missing required field: source"))?;

        let expect_rejected = case.fields.get("expect_rejected")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let result = (|| -> Result<_, fmpl_core::error::Error> {
            let tokens = Lexer::new(source).tokenize()?;
            Parser::with_source(&tokens, source).parse()
        })();

        if expect_rejected {
            match result {
                Ok(ast) => Err(StepError::new(format!(
                    "expected parse of `{source}` to be rejected, \
                     but parse succeeded with AST: {ast:?}"
                ))),
                Err(e) => {
                    if let Some(phrases) = case.fields.get("expect_error_contains").and_then(|v| v.as_list()) {
                        let msg = format!("{e:?}");
                        for phrase in phrases {
                            let needle = phrase.as_str().ok_or_else(|| StepError::new(
                                "expect_error_contains entries must be strings"
                            ))?;
                            if !msg.contains(needle) {
                                return Err(StepError::new(format!(
                                    "parse rejected, but error message did not contain {needle:?}.\nActual: {msg}"
                                )));
                            }
                        }
                    }
                    Ok(())
                }
            }
        } else {
            result.map(|_| ()).map_err(|e| StepError::new(format!(
                "expected parse of `{source}` to succeed, got: {e:?}"
            )))
        }
    }
}

inventory::submit! { StepDefRegistration(&ParseRejection) }
```

### Codegen (`fmpl-core/build.rs` extension)

```rust
// Pseudocode added to fmpl-core/build.rs:
fn generate_scenario_tests() -> std::io::Result<()> {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let corpus_path = Path::new(&manifest)
        .parent().unwrap()
        .join("docs/superpowers/iterations/behavior-scenarios.md");
    println!("cargo:rerun-if-changed={}", corpus_path.display());

    let cards = fmpl_scenario_runner::corpus::parse_corpus(&corpus_path)
        .map_err(|e| std::io::Error::other(format!("corpus: {e:?}")))?;

    let mut out = String::new();
    out.push_str("// AUTO-GENERATED by fmpl-core/build.rs — DO NOT EDIT\n\n");
    out.push_str("use fmpl_scenario_runner::{Card, Case, dispatch};\n");
    out.push_str("// SCENARIO_CORPUS is a parking lot — each #[test] re-parses\n");
    out.push_str("// the corpus on first access via std::sync::OnceLock.\n\n");

    for card in &cards {
        if card.action_type.is_none() {
            // Skipped: card has no default action type. Cases with explicit
            // overrides could still run, but for simplicity we skip the whole
            // card. (Could revisit in a future iteration.)
            continue;
        }
        for (i, _case) in card.cases.iter().enumerate() {
            let fn_name = format!("scenario_{}_case_{}",
                card.id.trim_start_matches("SCENARIO-"),
                i);
            writeln!(out,
                r#"
#[test]
fn {fn_name}() {{
    let cards = corpus();
    let card = cards.iter().find(|c| c.id == "{id}").unwrap();
    let case = &card.cases[{i}];
    if let Err(e) = dispatch(card, case) {{
        panic!(
            "behavior-scenarios.md:{{}}-{{}} ({id} case {i}): {{}}",
            card.line_start, card.line_end, e
        );
    }}
}}
"#,
                fn_name = fn_name, id = card.id, i = i)?;
        }
    }

    // Helper that lazy-parses the corpus once per test binary.
    out.push_str(r#"
fn corpus() -> &'static [Card] {
    static CORPUS: std::sync::OnceLock<Vec<Card>> = std::sync::OnceLock::new();
    CORPUS.get_or_init(|| {
        fmpl_scenario_runner::corpus::parse_corpus(
            std::path::Path::new("../docs/superpowers/iterations/behavior-scenarios.md")
        ).expect("corpus parse")
    })
}
"#);

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("scenarios_generated.rs");
    fs::write(out_path, out)?;
    Ok(())
}
```

## Data flow

```
docs/superpowers/iterations/behavior-scenarios.md  (source of truth)
                  │
                  ▼  cargo build (fmpl-core test target)
            fmpl-core/build.rs
                  │  invokes fmpl_scenario_runner::corpus::parse_corpus
                  ▼
            Vec<Card>
                  │  build.rs emits one #[test] per (card, case_index)
                  ▼
        OUT_DIR/scenarios_generated.rs
                  │  cargo test compiles + links
                  ▼
        scenario_runner test binary
          ├── tests/scenario_runner.rs   (include!s the generated file)
          ├── tests/steps/{*}.rs          (step-defs; inventory::submit!)
          └── inventory::iter populates the registry at static-init
                  │  each #[test] calls dispatch(card, case)
                  ▼
        Step-def runs the case; returns Result<(), StepError>
          ├── Ok:  test passes; line span shows in test output
          └── Err: panic!("behavior-scenarios.md:NN-MM (SCENARIO-NNNN case M): {err}")
```

## Card format

The runner accepts cards in `behavior-scenarios.md` with this shape:

```markdown
## SCENARIO-NNNN — Title

**Kind:** invariant | contract | surface | failure-recovery
**Proof seam:** unit | integration | e2e | app-level | process-level
**Owning stories:** STORY-NNNN, STORY-MMMM
**Action type:** `parse_rejection`         ← optional; absent ⇒ skipped

**Preconditions:**
- Free-form narrative bullets (informational; runner ignores)

**Action:**
- Free-form narrative bullets (informational; runner ignores)

**Cases:**
- action: `parse_rejection`
  source: `:Foo(1)`
  expect_rejected: true
  expect_error_contains:
    - `use [:Tag]`
    - `instead`
- action: `parse_success`
  source: `:Foo`
- source: `:Bar(1, 2, 3)`               ← inherits action_type from card default

**Expected observables:**
- Free-form narrative bullets (informational; runner ignores)

**Automation status:** implemented
**Execution command:** `cargo test -p fmpl-core --test scenario_runner scenario_NNNN`

**Sources:**
- file:line references
```

### Card-format rules

- `**Action type:**` at the card top is the **default** action type for cases that don't override. It is also the discoverability flag: a card without it is skipped by the runner.
- Each case in `**Cases:**` MAY override with its own `action:` key. If absent, the case inherits the card's default action type (the value of `**Action type:**` at the card top). A case is well-formed if it has either an explicit `action:` key OR the card has a default `**Action type:**`; a case with neither is a corpus error.
- Cases are bullet-list items. A case begins with a `- ` bullet at the indent level under `**Cases:**`. Subsequent more-indented bullets and `key: value` lines belong to that case until the next `- ` at the same indent (or the end of the `**Cases:**` block).
- Keys are `snake_case`. Values are:
  - **Backtick-quoted strings** (`` `:Foo(1)` ``) — preserves whitespace and special characters.
  - **Bare strings** (without backticks) — trimmed.
  - **Booleans** (`true` / `false`).
  - **Integers** (digit sequences).
  - **Indented sub-bullets** — a list of values.
- Line spans `(line_start, line_end)` are inclusive 1-based. Card span runs from its `##` heading to the line before the next `##` (or EOF). Case span runs from its `- ` bullet through the last sub-bullet.

### Cases-shape decision (locked from clarifying answers)

The user's choice for SCENARIO-0106 was **one card with multiple action-type cases**. The card format above accommodates this: each case in `**Cases:**` carries its own `action:` field. The card-level `**Action type:**` is the default for cases that don't specify their own.

## Step definitions

Three step-defs ship with the iteration. Each implements `trait StepDef` and registers via `inventory::submit!`.

### `parse_rejection`

```
Inputs:
  source:                 String       (required)
Expectations:
  expect_rejected:        bool         (default true)
  expect_error_contains:  Vec<String>  (default [])

Behavior:
  1. Tokenize `source` via Lexer.
  2. Parse via Parser.
  3. If expect_rejected:
       - Err   → for each phrase in expect_error_contains, assert it appears
                 in format!("{err:?}").
       - Ok    → fail with the AST in the message.
     Else (expect_rejected = false):
       - Ok    → pass.
       - Err   → fail with the error in the message.
```

### `parse_success`

```
Inputs:
  source:                 String       (required)

Behavior:
  1. Tokenize `source` via Lexer.
  2. Parse via Parser.
  3. Assert Ok; on Err, fail with the error in the message.
```

(Distinct from `parse_rejection` with `expect_rejected: false` for discoverability — control cases read more clearly as `action: parse_success` than as `action: parse_rejection / expect_rejected: false`.)

### `grep_invariant` (two action types: `expect_absent`, `expect_present`)

```
Common inputs:
  needle:                 String       (required)
  scope:                  String       (required; path relative to repo root,
                                        either a file or a directory)
expect_absent expectations:
  (none; the implied expectation is "0 matches")
expect_present expectations:
  min_count:              usize        (default 1)

Behavior:
  1. Resolve `scope`. If a file: load it. If a directory: recursively collect
     all `.rs` files under it.
  2. For each file, walk lines. Strip `//`-line-comments (per the helper moved
     from structural_invariants.rs). Count whole-word matches of `needle`.
  3. For `expect_absent`: assert total count == 0. On failure, list every hit
     as `path:line: text`.
  4. For `expect_present`: assert total count >= min_count. On failure, give
     count + the searched scope.
```

The `comment_strip` helper from `structural_invariants.rs` moves to `fmpl-core/tests/common/comment_strip.rs` so both the (transitional) old test file and the new step-def can call it.

## Error handling

### Three failure modes

| Mode | Cause | Behavior |
|---|---|---|
| **Corpus parse error** | Malformed card (missing required field, bad indent, syntax error) | `build.rs` panics with `[corpus:NN-MM] error: <description>`. Build fails; no tests run. |
| **Dispatch error** | Card has `**Action type:** foo` but no step-def registered for `foo` | The generated `#[test]` panics immediately with `unknown action_type "foo"`. Other scenarios still run. |
| **Case failure** | Step-def returned `Err(StepError)` | Normal `#[test]` panic with the formatted message. Other tests unaffected. |

### Failure output format

```
behavior-scenarios.md:2149-2180 (SCENARIO-0104 case 0):
  expected parse of `:Foo(1)` to be rejected, but parse succeeded with AST:
    Expr::Tagged("Foo", [Expr::Int(1)])
```

First line is machine-parseable (`file:span (id case N): prefix`). Body is the step-def's message, indented two spaces.

### Test name convention

```
scenario_NNNN_case_M
```

`M` is the zero-based case index within the card's `**Cases:**` list. `cargo test scenario_0104` filters to all cases of SCENARIO-0104.

### Skipped-scenarios summary

A `corpus_health_check` test always passes but writes to stderr:

```
[scenario_runner] skipped: 77 cards have no **Action type:** (run with
                  FMPL_SCENARIO_LIST_SKIPPED=1 to see them all)
```

Informational; does not affect test pass/fail.

### Compile-time validation

`build.rs` performs static checks beyond parsing:

- Every card with `**Action type:**` must have at least one case.
- Every case's resolved `action` (override or card default) must be a non-empty string.
- Duplicate scenario IDs are a corpus error.
- The inventory step-def registry isn't visible to build.rs (runtime-only), so the build-time check cannot verify "every action_type has a step-def". That validation happens at runtime via dispatch errors.

## Testing strategy

### `fmpl-scenario-runner` (the crate itself)

- `tests/corpus_parse.rs` — fixture-driven corpus parser tests:
  - Minimal valid card (1 case, default action)
  - Card with mixed-action cases (matches SCENARIO-0106 shape)
  - Card without `**Action type:**` (parses, marked skipped)
  - Malformed-card fixtures (one per error type): missing Cases, duplicate id, bad indent, unterminated case
  - Card with all field types (string, bool, int, list-of-strings)
- `tests/step_dispatch.rs` — exercises StepDef trait + inventory:
  - Registration works.
  - Dispatch picks the right step-def by action_type.
  - Unknown action_type returns DispatchError::Unknown.

### `fmpl-core` integration

- `fmpl-core/tests/build_codegen_check.rs` — parses the real `behavior-scenarios.md`; asserts ≥3 runnable cards exist; asserts the codegen output contains one `#[test]` per case.
- `fmpl-core/tests/scenario_runner.rs` — the test target itself; once SCENARIO-0104/0105/0106 are migrated, this binary produces the 17 evidence tests.
- Step-defs have their own unit tests in their `tests/steps/*.rs` modules against synthetic inputs.

### Sentinel verification

- After migration, `cargo test -p fmpl-core --test scenario_runner` reports the same passing-test count for SCENARIO-0104/0105/0106 evidence as `structural_invariants.rs` does today (17 tests).
- All other sentinels (ast_to_ir_parity, scenario_0103, tavern_demo, no_legacy_fmpl_syntax) still green.

## Order of work (for the implementation plan)

1. Scaffold `fmpl-scenario-runner` crate: Cargo.toml, lib.rs stub, workspace member registration.
2. Implement `corpus.rs` with fixture-driven TDD.
3. Implement `step_def.rs` (trait + inventory plumbing) with synthetic step-def tests.
4. Implement `error.rs` (StepError, DispatchError, CorpusError).
5. Add the codegen path to `fmpl-core/build.rs`.
6. Move comment-strip helper from `structural_invariants.rs` to `tests/common/comment_strip.rs`.
7. Implement the three step-defs in `tests/steps/`.
8. Migrate SCENARIO-0104, 0105, 0106 cards to the new structured format.
9. Run `scenario_runner`; verify 17 evidence tests pass.
10. Delete `structural_invariants.rs`.
11. Update `behavior-corpus.md` execution commands.
12. Update `no_legacy_fmpl_syntax.rs` exclusions.
13. Update progress.md and iteration-log.md.

## Acceptance criteria

- `cargo test -p fmpl-core --test scenario_runner` reports ≥17 passing tests, 0 failed (same evidence count as today's `structural_invariants.rs`).
- `cargo test -p fmpl-core --test scenario_runner scenario_0104` filters correctly.
- A failing case prints `behavior-scenarios.md:NN-MM (SCENARIO-NNNN case M): <msg>`.
- `structural_invariants.rs` is deleted; `fmpl-core/tests/common/comment_strip.rs` retains the comment-strip helper.
- `fmpl-scenario-runner` crate has its own passing tests.
- Full workspace `cargo test` is green.
- `cargo test -p fmpl-core --test no_legacy_fmpl_syntax` still passes (baseline updated for the file shuffle).
- All other sentinels untouched.

## Risks and mitigations

- **`inventory` cross-crate visibility.** Step-defs in `tests/steps/*.rs` only register if the test binary's `mod steps;` declaration is present. Mitigation: `scenario_runner.rs` declares `mod steps;` at the top; the build.rs codegen does not assume otherwise.
- **Corpus parser brittleness on existing cards.** Most of the ~80 existing cards do not have `**Action type:**` and use free-form narrative. The parser must tolerate this and skip such cards without erroring. Mitigation: parse cards leniently; only complain about a card if it declares `**Action type:**` AND then has malformed `**Cases:**`.
- **`cargo:rerun-if-changed` performance.** Rebuilding test infrastructure every time `behavior-scenarios.md` changes is desired but cheap (a single ~60KB file scan). Acceptable cost.
- **Step-def-to-card field-mismatch errors at runtime.** A step-def that expects `source` but the case has `src` produces a runtime failure rather than a compile-time one. Mitigation: each step-def emits a clear `case missing required field: <name>` StepError; this is a small price for the data-driven design.

## Out of scope (deferred)

- Migration of scenarios beyond SCENARIO-0104/0105/0106. Migration is opt-in; future iterations can add cards as their action types become defined.
- A grammar-based scenario parser written in FMPL (on-brand with DESIGN-001 metacircular; a future iteration).
- Parameterized step-defs with fixture libraries.
- Output formatters beyond `cargo test` default.
- Coverage gates (e.g., "every scenario in the corpus must have an action_type by 2026-06-01"). Future iteration.

## Origin and references

- ITER-0004d.1 T19 user review (2026-05-12): "the tests in rust would be required to be even more minimal than what you've got there, as it should just be a driver for the scenario list, like the cucumber system, or the fitnesse.org SLIM framework."
- ITER-0004d.1 T19 user request (2026-05-12): "The scenario runner should probably also emit the span of line numbers for the test cases running."
- `fmpl-core/tests/structural_invariants.rs` — first consumer; deleted at iteration end.
- `docs/superpowers/iterations/behavior-scenarios.md` — corpus source.
- `docs/superpowers/iterations/behavior-corpus.md` — execution index updated by iteration.
- `docs/superpowers/iterations/roadmap.md` — ITER-0004d.4 entry contains additional rationale.
