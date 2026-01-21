# Phase 6: Panel Interactivity for Agentic TUI

## Goal

Transform the static three-panel layout (Research/Planning/Execution) into a functional, interactive agentic development environment that supports the 12-layer human-AI collaboration architecture.

---

## Current State

**Completed Phases:**
- ✅ **Phase 1**: Conversation DAG foundation (undo/redo/edit/branches)
- ✅ **Phase 2**: Backtracking UI (history selection, replay from here, diff view)
- ✅ **Phase 5**: Auto-detection for conversation compaction

**Current TUI Layout (fmpl-tui/src/main.rs:1263-1420):**
```
┌─────────────────────────────────────────────────────────────┐
│                    Research View (33%)                      │
│  - Static text "Research view - Problem space analysis"     │
│  - In LLM mode: Shows conversation history / diff view      │
├─────────────────────────────────────────────────────────────┤
│                    Planning View (33%)                      │
│  - Static text "Planning view - Collaborative scope..."     │
├─────────────────────────────────────────────────────────────┤
│              Execution View (34%) - Split 50/50             │
│  ┌──────────────────────┬──────────────────────────────────┐
│  │   Code Editor        │   Execution Output               │
│  │   (FMPL code input)  │   (LLM responses / eval results) │
│  └──────────────────────┴──────────────────────────────────┘
└─────────────────────────────────────────────────────────────┘
```

**Current Functionality:**
- Code editor with cursor navigation
- FMPL evaluation (Esc+Enter to execute)
- LLM chat mode (Ctrl+L to toggle)
- Conversation history with DAG backtracking
- History selection mode (Ctrl+H)
- Replay from selected node
- Diff view for branch comparison
- Auto-detection of off-track/circular conversations

---

## Vision: Interactive Agentic Panels

Per the 12-layer architecture (docs/plans/12-layer-human-ai-architecture.md):

> **Layer 1 (Input Layer)**: A research view of the problem space, assembled into context for the next layer. A planning view, which given the broader context of the research view, works collaboratively with the user to find the right scope for implementation. Given the planning view, an execution view that breaks down the plan into actionable steps.

The panels should become:
1. **Research Panel**: Problem space exploration (grep results, file analysis, context gathering)
2. **Planning Panel**: Task breakdown and scope definition (checklist, priorities, dependencies)
3. **Execution Panel**: Code editing + LLM interaction (already functional)

---

## Phase 6 Deliverables

### Task 6.1: Panel Focus Navigation (S - 1-2 hours)

**Goal**: Enable user to switch keyboard focus between panels.

**Implementation:**
- Add `focused_panel: PanelType` field to `App` struct
- `PanelType` enum: `Research`, `Planning`, `CodeEditor`, `Output`
- Keybindings:
  - `Ctrl+R`: Focus research panel
  - `Ctrl+P`: Focus planning panel
  - `Ctrl+E`: Focus code editor (default)
  - `Ctrl+O`: Focus output panel
- Visual indicator: Yellow border or `[FOCUSED]` in title

**Success Criteria:**
- Tab focus switches between panels
- Visual indicator shows active panel
- Arrow keys work correctly in each panel context

---

### Task 6.2: Editable Research Panel (M - 2-3 hours)

**Goal**: Allow users to write, edit, and save research notes.

**Implementation:**
- Add `research_lines: Vec<String>` to `App` (like `code_lines`)
- Add `research_cursor: (usize, usize)` for cursor position
- When focused in Research panel:
  - Typing adds text to `research_lines`
  - Enter inserts new line
  - Backspace deletes
  - Arrow keys navigate
- Add save/load functionality:
  - `Ctrl+S`: Save research notes to `.agent/research.md`
  - Auto-load on startup if file exists

**Data Model:**
```rust
struct App {
    // ... existing fields ...
    focused_panel: PanelType,
    research_lines: Vec<String>,
    research_cursor_row: usize,
    research_cursor_col: usize,
    // ... planning fields similar ...
}
```

**Success Criteria:**
- Can type and edit text in research panel
- Saves to `.agent/research.md`
- Loads on startup
- Cursor navigation works

---

### Task 6.3: Editable Planning Panel with Task List (M - 3-4 hours)

**Goal**: Interactive task list for planning implementation steps.

**Implementation:**
- Add `planning_tasks: Vec<PlanningTask>` to `App`
- `PlanningTask` struct:
  ```rust
  struct PlanningTask {
      id: usize,
      description: String,
      status: TaskStatus, // Pending, InProgress, Complete
      priority: Priority, // Low, Medium, High
  }
  ```
- Keybindings when Planning panel focused:
  - `a`: Add new task
  - `e`: Edit selected task description
  - `Enter`: Toggle task status (pending ↔ in-progress ↔ complete)
  - `d`: Delete task
  - `+`/`-`: Increase/decrease priority
  - Arrow up/down: Select task

**Rendering:**
```
┌─ Planning View [FOCUSED] ─────────────────────────────┐
│ [ ] Task 1: Implement panel focus navigation      [L] │
│ [→] Task 2: Make research panel editable           [M] │
│ [✓] Task 3: Bootstrap LLM libraries               [H] │
│                                                        │
│ a:add e:edit Enter:toggle d:del +/-:priority         │
└────────────────────────────────────────────────────────┘
```

**Persistence:**
- Save to `.agent/tasks.md` on change
- Load on startup
- Format: Markdown checkboxes with priority tags

**Success Criteria:**
- Can add/edit/delete tasks
- Can toggle status
- Can set priority
- Persists to `.agent/tasks.md`
- Visual rendering shows status clearly

---

### Task 6.4: Panel-Specific Help Text (S - 1 hour)

**Goal**: Show context-sensitive keybinding help in each panel.

**Implementation:**
- Add `panel_help(panel: PanelType) -> String` method to `App`
- Display at bottom of panel when focused
- Examples:
  - Research: "Ctrl+S: save | Arrow keys: navigate"
  - Planning: "a:add e:edit Enter:toggle d:del"
  - Code: "Esc+Enter: execute | Ctrl+L: LLM mode"
  - Output: "Ctrl+C: copy to clipboard (future)"

**Success Criteria:**
- Each panel shows relevant keybindings
- Help text updates when focus changes

---

## Phase 6 Scope (In vs Out)

### In Scope
- Panel focus navigation (Task 6.1)
- Editable research panel with persistence (Task 6.2)
- Editable planning panel with task list (Task 6.3)
- Context-sensitive help (Task 6.4)

### Out of Scope (Future Phases)
- LLM integration for auto-generating research/planning content
- Tool management interface (Phase 7)
- Context compaction UI (Phase 4)
- Branch switching UI (Phase 3)
- Multi-panel split views
- Mouse support

---

## Implementation Order

**Priority 1 (Foundation):**
1. Task 6.1: Panel focus navigation - enables all subsequent tasks

**Priority 2 (Core Functionality):**
2. Task 6.2: Editable research panel - enables note-taking
3. Task 6.3: Editable planning panel - enables task tracking

**Priority 3 (Polish):**
4. Task 6.4: Context-sensitive help - improves UX

---

## Testing Strategy

### Manual Testing (TUI is hard to unit test)
- Build: `cargo build --release`
- Run: `cargo run --release`
- Test each keybinding in isolation
- Test panel switching preserves state
- Test save/load persists data correctly

### Smoke Tests
- App starts without panic
- Can switch between all panels
- Can type in research panel
- Can manage tasks in planning panel
- Files created in `.agent/`

---

## Success Criteria

Phase 6 is complete when:

1. **Panel Navigation**: Can switch focus between all 4 panels (Research, Planning, Code, Output)
2. **Research Panel**: Can type, edit, save, and load research notes
3. **Planning Panel**: Can add, edit, delete, and toggle tasks with priorities
4. **Persistence**: `.agent/research.md` and `.agent/tasks.md` are created and loaded
5. **Help Text**: Each panel shows relevant keybindings
6. **No Regressions**: All 222 existing tests still pass
7. **Build Clean**: `cargo build --release` succeeds without warnings

---

## Next Steps (Phase 7+)

After Phase 6:
- **Phase 7**: Tool management interface (add/remove/configure tools)
- **Phase 3**: VCS-style operations (branch switching, merge)
- **Phase 4**: Context compaction UI (relevance scoring, elision)
- **Phase 8**: LLM-assisted research/planning generation

---

## References

- [12-Layer Human-AI Architecture](./12-layer-human-ai-architecture.md) - Overall vision
- [Unified Grammars and Agents Design](./2026-01-19-unified-grammars-and-agents-design.md) - Agent control flow
- [FMPL TUI Implementation](../fmpl-tui/src/main.rs) - Current code
- [Ralph Loop Scratchpad](../../.agent/scratchpad.md) - Phase tracking
