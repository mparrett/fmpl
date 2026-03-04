  # Headless Task Loop — Single Iteration

  Skip all interactive skills. No summaries. No exploration beyond what's needed.

  ## Step 1: Pick Task
  `jj issue ready | head -1` → `jj issue show <id>`

  ## Step 2: Quick-Check
  Run the test that would verify completion. If it passes → close, pick next (max 3).

  ## Step 3: Implement
  Read ONLY files not quoted in the issue. Follow TDD. Filter ALL cargo output.

  ## Step 4: Verify & Commit
  One `cargo test` (filtered). One `cargo clippy` (filtered). Commit with jj.

  ## Step 5: Output
  Print: COMPLETED:<id> <conventional commit message>

  Budget: 15 tool calls max. 3 task-selection loops max.
