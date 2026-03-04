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

## Budget

- 40 tool calls max per iteration
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
