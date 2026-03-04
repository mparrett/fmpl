# Headless Task Loop — Single Iteration

This is a headless automation loop. Skip ALL process/interactive skills (where-was-i, brainstorming, context-snapshot, episodic-memory). Go directly to task selection.

@a study specs/README.md

## State Machine

A hook-enforced state machine controls which tools you can use at each step.
You will see `[STATE -> X]` messages after tool calls indicating transitions,
followed by instructions for the new state. Follow them.

```
                    ┌─── IMPLEMENT → VERIFY → REVIEW → COMMIT
PICK_TASK → TRIAGE ─┤        ↑          │
    ↑          │    └─── RESEARCH → DOCUMENT → COMMIT
    └──close───┘
```

**Two arcs from TRIAGE:**
- **Implementation arc**: Task requires code → IMPLEMENT → VERIFY → REVIEW → COMMIT
- **Research arc**: Task needs discovery first → RESEARCH → DOCUMENT → COMMIT

The health check runs **before you start** (in pre-flight). Your starting state and
instructions are in the user message.

## Pre-flight Context

Your user message contains:
- **Health check results** (pass/fail with test output)
- **Uncommitted changes** (files modified outside the loop)
- **Protected files** that you MUST NOT overwrite or revert
- **Current step instructions** for your starting state

If protected files are listed, the state machine will block Write/Edit on them.

## Rules

- The issue description IS your research. Do NOT re-read files already quoted in it.
- Check `docs/codebase/` before coding — Write/Edit blocked until you do.
- Use Explore subagents for research, not serial Read/Grep.
- `jj status` creates checkpoints. `jj undo` rolls back.
- Consolidate discoveries to `docs/codebase/` before committing.

## Version Control (jj)

- Use `jj describe -m "message"` to describe the current change, NOT `jj new -m "message"` (which creates an empty new change)
- After `jj commit` or `jj describe`, the working copy showing as modified is normal — not an error
- Check `jj diff` before committing to avoid mixing unrelated changes
- Use `jj split` to split commits, not manual workarounds

## Issue Triage

- Before implementing, check if the issue is already done (run tests, grep codebase)
- Do NOT spend multiple loops closing already-completed issues — batch-check upfront
- If 2+ issues in a row are already done, stop and report the pattern

## Task Sizing

- Prefer small, well-scoped issues over large multi-component ones
- If an issue spans multiple crates, implement and commit one crate at a time

## Rust Development

- Run `cargo clippy` after implementation changes, before committing
- Read docs/source for unfamiliar crate APIs — do NOT guess the API shape
- If a dependency API doesn't work after 2 attempts, read the type signatures in the dependency source

## Budget

- **Context goals (soft)**: Keep reference data (file reads, issue descriptions, docs) under ~40% of context. Keep implementation work (edits, test runs, verification) under ~60%. These are guidelines — prioritize task completion over strict limits.
- 3 close-and-pick loops max in triage
- 3-strike rule: same error 3 times → write spec, comment on issue, stop
- Do NOT use TodoWrite — the issue tracker is the task list

## Output

Print exactly one line:
```
COMPLETED:<id> <conventional commit message>
```

Or if blocked:
```
BLOCKED:<id> <reason>
```
