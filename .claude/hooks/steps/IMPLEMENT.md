You MUST produce code in this step. Triage and decomposition are not deliverables.

**Before coding, check `docs/codebase/`** for existing pattern docs (Write/Edit blocked until you do).

Single-crate task: implement directly.
Multi-crate or large task: decompose into subtasks (`jj issue create`), pick the first, implement it.

**Checkpoints:** `jj status` before risky changes to snapshot. `jj undo` to roll back.

**Subagents:** Use Explore for code structure, codebase-analyzer for call chains, context7 for external API docs. Do NOT use subagents to write files you're also editing.

**Anti-avoidance:** You are committed. No task-switching, no decompose-without-code, no picking easier tasks.

When tests pass (cargo test), you'll advance to VERIFY.
