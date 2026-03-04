Check if the issue is already done. Max 3 close-and-pick loops:

- If comments say work is done -> verify with one test -> if pass, close and loop to PICK_TASK.
- If subtasks exist and are all closed -> close parent, loop to PICK_TASK.
- If a test already passes -> close and loop to PICK_TASK.

If not already done, choose the right arc:

- **Implementation** (default): The issue asks for code changes -> start writing code (Write/Edit auto-transitions to IMPLEMENT).
- **Research**: The issue needs codebase discovery before implementation, OR lacks code snippets/patterns -> use `jj issue comment <id> "RESEARCH: <reason>"` to enter the research arc.
