Dispatch a code review before committing:

```
Task(subagent_type="superpowers:code-reviewer", model="haiku",
     prompt="Review the changes in the current jj working copy against the task requirements.")
```

Or use: Skill(codereview-reviewing) or Skill(superpowers:requesting-code-review)

If review finds issues, fix them (Write/Edit returns to IMPLEMENT).
If review passes, you'll advance to COMMIT.
