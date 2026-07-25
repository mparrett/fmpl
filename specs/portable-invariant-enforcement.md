# Portable Invariant Enforcement

**Status**: Proposed
**Date**: 2026-07-24
**Type**: Process / Tooling Spec

---

## Summary

Several development invariants that the upstream agent harness enforced *in code*
survive in this fork only as prose in `AGENTS.md`. This spec establishes what is
already enforced, enumerates the real gap, proposes the cheapest durable mechanism
for each item, and lists the ones no repo-resident mechanism can enforce.

The conclusion is smaller than the question suggests. **The baseline is much
stronger than `AGENTS.md` implies**; most archived gates should not be restored.
One conversion (parser-epoch bump detection) plus two one-line consistency fixes are worth doing; the rest is covered or unenforceable.
Archived evidence lives on `archive/agent-harness`. `.claude/hooks/*.py` and
`ralph.sh` are **symlinks at the branch tip** pointing at a path that existed only
on the author's machine; the real sources come from history
(`git show 2a6db48:.claude/hooks/ralph-gate.py`, `…ralph-preflight.py`).
## Baseline: what is already enforced

**CI — `.github/workflows/ci.yml`**, every PR and every push to `main`: bootstraps
the canonical parser, then `cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`. Toolchain is
`dtolnay/rust-toolchain@stable`, so "CI runs the latest stable toolchain" is a fact,
not an aspiration. CI is also a fresh clone — what actually catches machine-specific
breakage of the `execution_tape` kind.

**`fmpl-core/build.rs`** gates more than its name suggests. Because CI sets `CI`, it
turns on `enforce_freshness` (a stale parser with no `fmpl-bootstrap` available is a
hard `panic!`, not a warning) and `enforce_determinism` (the generator runs twice;
outputs must be byte-identical). It regenerates on `PARSER_EPOCH` mismatch, and
generates the scenario suite from `docs/behavior-scenarios.md` — so "don't move that
file" is enforced by build failure, not trust.

**Repo-resident tests already encode invariants** — the house idiom, and the model
for anything new: `canonical_pipeline_parity::canonical_pipeline_must_be_active`
(asserts `IS_GENERATED_PARSER`, so a silently substituted fallback fails loudly
instead of making the suite pass trivially); `postlude_arm_contract.rs` (same guard,
scoped to the postlude arms); `bootstrap_determinism.rs`; `no_legacy_fmpl_syntax.rs`
and `persistence_schema_anti_rot.rs` (universally quantified structural scans
asserting a zero hit count); `doc_examples.rs` (executes ```` ```fmpl ```` blocks in
`TUTORIAL.md` / `DEMO.md` / `README.md`); and the compile-time
`const _: () = assert!(PARSER_EPOCH == GENERATED_PARSER_EPOCH)` in `parser.rs`.

**`.pre-commit-config.yaml`** checks whitespace, EOF newline, YAML syntax, and large
files — nothing Rust-specific. It is opt-in (nothing installs it) and, decisively,
**jj does not run git hooks**, so for the `jj describe` workflow `AGENTS.md`
prescribes this tier is inert. In the **`justfile`**, `build` and `test` perform the
bootstrap, but `just clippy` omits `-- -D warnings` — a locally clean `just clippy`
is not proof of a green CI.
## Mechanism ranking by portability

1. **Repo-resident tests** — travel to every clone, agent, human, and CI provider;
   no installation, no opt-in, nothing to forget. The only tier that reaches a
   contributor *before* they open a PR without asking them to configure anything.
2. **CI gates** — travel to every PR, never to local work. Good backstop, poor
   feedback loop; an agent can burn an hour on a red tree first.
3. **`justfile` recipes** — repo-resident and discoverable, but advisory.
4. **Pre-commit hooks** — local, opt-in, `--no-verify`-able, skipped by jj.
5. **Agent-specific hooks** (Cursor `hooks.json`, Claude Code `PreToolUse`) — one
   tool only. What the archived harness used, and why its rules evaporated when
   the harness moved.
6. **Prose in `AGENTS.md`** — zero enforcement. Correct only when the invariant is
   genuinely about in-session behavior.
## The delta

| `AGENTS.md` asks | Verified by | Proposal |
|---|---|---|
| Green build before commit | CI (on PR) | Prose — see below |
| Zero clippy warnings | CI `-D warnings` | Add `-- -D warnings` to `just clippy` |
| Bump `PARSER_EPOCH` on postlude change | Nothing | Fingerprint test (below) |
| Parser changes go to *both* parsers | `parser_equivalence.rs`, `canonical_pipeline_parity` | Broaden corpus if a divergence escapes |
| `#[allow]` at file top with a comment | Nothing | Structural scan, low priority |
| TDD; don't fix tests by changing them | Nothing | Unenforceable — prose |
| Read `docs/codebase/` first | Nothing | Unenforceable — prose |
| Specs < 200 lines | Nothing | Not worth a gate |

Two items absent from `AGENTS.md` but enforced upstream:

- **`cargo fmt`.** The archived preflight auto-ran it and VERIFY required it
  (`2a6db48:.claude/hooks/ralph-preflight.py`). This fork has no fmt gate anywhere
  and no fmt rule; `cargo fmt --all -- --check` in CI is among the cheapest gates
  available.
- **A protected-files list** (`PROMPT.md`, `AGENTS.md`, `.claude/settings*.json`)
  that stopped the loop reverting human edits. That risk was specific to an
  unattended loop rewriting its own instructions; with no loop there is nothing to
  protect against. **Not worth restoring.**

**Green build before commit.** Upstream blocked `jj describe` until a full
`cargo test` had passed in-session (`ralph-gate.py`, COMMIT state). Repo-side the
only equivalent is a pre-push/pre-commit hook running `just test` — which jj skips,
which is bypassable, and which costs a full suite on every commit. CI already blocks
the merge. **Leave this as prose**: the cure costs more than the disease.
## Highest-value conversion: the parser-epoch bump policy

`fmpl-core/src/parser_epoch.rs` spends ~35 lines of prose on which changes require
a `+1` bump. Both existing mechanisms check the *wrong direction*:

- The `const _` assert in `parser.rs` and the build script's epoch comparison catch
  **epoch bumped, parser not regenerated**.
- Nothing catches **postlude edited, epoch not bumped** — because
  `rerun-if-changed=src/builtins/ir_to_rust.rs` regenerates the parser, and the
  fresh parser embeds the current un-bumped epoch. The two agree, the check passes,
  and a consumer with a cached parser gets a confusing `E0599` deep in
  `out/generated_parser.rs` — the exact failure the policy exists to prevent.

**Sketch.** A test asserts that a fingerprint of the postlude source matches a
blessed value for the current `PARSER_EPOCH`:

```rust
// tests/parser_epoch_fingerprint.rs (sketch)
const BLESSED: &[(u32, &str)] = &[(9, "b3:…")];
// hash the postlude surface, look up PARSER_EPOCH, compare
```

**Prerequisite.** The postlude is not one string: `GRAMMAR_HELPERS`
(`ir_to_rust.rs:62`), `RUNTIME_PRELUDE` (`:261`), and two anonymous inline `r#"…"#`
literals (`:1143`, `:1207`). Promoting the two anonymous literals to named
`const &str` makes the fingerprint surface exactly four constants a test can hash
directly, with no source parsing. Without that refactor the test must scrape the
file or hash all of `ir_to_rust.rs`, whose generator logic changes for reasons
unrelated to the postlude — guaranteeing false positives.

**Tradeoffs, honestly.**

- *False positives on cosmetic edits.* A typo fix inside a postlude string changes
  the hash and demands a bump. The policy already says "when in doubt, bump," and a
  spurious bump costs one regeneration, so failing closed is right — it will still
  annoy someone.
- *A blessing workflow is required.* The failure message must spell out: bump
  `PARSER_EPOCH`, add a bump-history entry, update the blessed hash. An
  `FMPL_BLESS_POSTLUDE=1` auto-regeneration path leaves the gate one keystroke from
  meaningless; prefer printing the hash for the developer to paste.
- *It does not cover every documented trigger.* AST-surface, value-encoding, and
  `Instruction` changes can invalidate a cached parser without touching the
  postlude. Five of the nine recorded bumps cite a postlude edit, so this covers the
  most common trigger and no more. A ratchet, not a proof.
## What cannot be enforced repo-side, and why

These were real gates upstream. They governed **agent behavior inside a session**,
which no file in a git repository can observe. Restoring them means an
agent-specific hook for one tool on one machine — the failure mode this exercise is
reacting to.

- **Context budget / filtered cargo output.** Upstream blocked unfiltered
  `cargo build|test|check|clippy` in IMPLEMENT, VERIFY, and COMMIT (`ralph-gate.py`;
  also `2a6db48:.claude/settings.json`). A repo cannot see how much context a tool
  call consumed, and modern agent runtimes already truncate large command output.
- **Read `docs/codebase/` before the first `Write`.** A repo cannot know what you
  read. `DEV.md` already points there, and the directory holds two files — small
  payoff even if it were enforceable.
- **The 3-close-and-pick ceiling and `health_fix` mode**, which bounded loop thrash
  and kept feature work out of a repair (`git show 0895d71`). Properties of a
  driving loop, not a worktree. They also depend on `jj issue`, not a stock jj
  subcommand — another local-only dependency, like the symlinked hooks.
- **TDD ordering.** Detectable only from commit shape; any heuristic strong enough
  to catch real violations will misclassify legitimate refactors.
## One genuine inversion, as a caution

Upstream **blocked** reading `~/.cargo/registry`: `2a6db48:.claude/settings.json`
exits 2 on any command or file path containing `.cargo/registry`, and
`archive/agent-harness:AGENTS.md:149` says "Do not grep through
`~/.cargo/registry/src/`." Our `AGENTS.md` says the opposite — read registry source
when a dependency API resists after two attempts.

Neither is wrong. His loop ran unattended against a hard context budget with a
docs-retrieval MCP available, so a registry grep was a context bomb with a cheaper
substitute. We run interactively without that substitute, and ground truth beats
guessing. A rule-set encodes the constraints of the environment that produced it:
**port the invariant, re-derive the mechanism.**
## Adoption order

1. **`just clippy` gains `-- -D warnings`.** One line; removes the trap where local
   green ≠ CI green. Do this first.
2. **Name the two anonymous postlude raw-strings** in `ir_to_rust.rs`. Pure
   refactor; unblocks step 3.
3. **Add the postlude fingerprint test**, blessed for epoch 9, with a failure
   message that spells out the bump-and-bless procedure.
4. **Add `cargo fmt --all -- --check` to CI**, a matching `just fmt-check`, and one
   line in the `AGENTS.md` quality gates.
5. **Only if they actually bite**: broaden the two-parser differential corpus, and
   add an `#[allow]`-placement scan modeled on `persistence_schema_anti_rot.rs`.

Explicitly **not** proposed: pre-commit Rust hooks (jj skips them), a
protected-files mechanism, a local pre-commit test gate, a `docs/codebase/`
read-gate, a cargo-output filter, a spec line-count check, and any Cursor- or
Claude-specific `hooks.json`.
## References

- `.github/workflows/ci.yml`, `.pre-commit-config.yaml`, `justfile`, `fmpl-core/build.rs`
- `fmpl-core/src/parser_epoch.rs` (bump policy); `fmpl-core/src/builtins/ir_to_rust.rs`
  (postlude surface: `:62`, `:261`, `:1143`, `:1207`)
- `fmpl-core/tests/` — `canonical_pipeline_parity.rs`, `postlude_arm_contract.rs`,
  `no_legacy_fmpl_syntax.rs`, `persistence_schema_anti_rot.rs`, `bootstrap_determinism.rs`
- `archive/agent-harness` — `.claude/settings.json`, `AGENTS.md:149`; hook sources via
  history (`2a6db48`, `0895d71`, `00ec15d`), not the branch tip (dead symlinks)
