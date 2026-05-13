# Lessons

> _Auto-managed below. Hand-curated preamble + seed lessons above the sentinel are preserved across renders._

## Auto-promoted entries will be appended below

### 2026-05

- When planning a refactor that deletes producer code paths AND also wants to rename surviving consumer code (e.g., bytecode opcodes), enumerate the rename targets AFTER a cargo-check view of what survives the deletion phase, not before. PAR scope review catches citation drift but cannot enumerate dependent sites the planner didn't grep for. The pattern: build a tooling precursor or skeleton-deletion first, let the compiler surface the real consumer set, then plan the rename against that ground truth.  <!-- status=accepted confidence=0.6 evidence=1 id=lesson_f2576de9c008 -->
- FAILURE in claude-code: Command failed: cat > /tmp/test_lex.rs << 'EOF' | THIS SKILL HAS FAILED 10 TIMES IN 14d. Flag for rewrite.  <!-- status=accepted confidence=1.0 evidence=1857 id=lesson_d49365f28837 -->
- FAILURE in claude-code: Command failed: jj new -m "docs(iter-0004d.4): scenario runner design spec (brainstorming output) | THIS SKILL HAS FAILED 14 TIMES IN 14d. Flag for rewrite.  <!-- status=accepted confidence=1.0 evidence=631 id=lesson_70c3e012fa1b -->

### 2026-04

- Always serialize timestamps in UTC to avoid cross-region comparison bugs  <!-- status=accepted confidence=0.46 evidence=1 id=lesson_422695ae5b2d -->
