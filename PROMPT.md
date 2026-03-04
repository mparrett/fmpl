# Headless Task Loop — Single Iteration

This is a headless automation loop. Skip ALL process/interactive skills (where-was-i, brainstorming, context-snapshot, episodic-memory). Go directly to task selection.

@a study specs/README.md

## Step 1: Pick Task

`jj issue ready | head -2` → pick the top task → `jj issue show <id>`. Tasks that are [open] are ready.

The issue description IS your research. Do NOT re-read files already quoted in the issue.

## Step 2: Quick-Check

If there's an obvious test that would verify completion, run it (ONE test, filtered).
If it passes → `jj issue close <id>` with a comment, then pick next task. Max 3 close-and-pick loops.

## Step 3: Implement

- Read ONLY files not quoted in the issue. Read generously, once per file.
- Use subagents for research beyond what the issue provides.
- Follow TDD. Filter ALL cargo output (see AGENTS.md).
- For external crate APIs, use context7 or docs.rs. Never grep `~/.cargo/registry`.

## Step 4: Verify & Commit

1. ONE `cargo test` run (filtered). Must pass.
2. ONE `cargo clippy` run (filtered). Must pass, zero warnings.
3. Commit with jj (use jj-workflow skill for commit message conventions).

If the build is broken, fix it. Do not declare done while broken.

## Step 5: Output

Print exactly one line:
```
COMPLETED:<id> <conventional commit message>
```

Or if blocked:
```
BLOCKED:<id> <reason>
```

## Budget

- 20 tool calls max per iteration
- 3 close-and-pick loops max in Step 2
- 3-strike rule: same error 3 times → write spec, comment on issue, stop
