# Phase 7: Tool Management Interface for Agentic TUI

## Goal

Enable dynamic tool management within the agentic TUI, allowing users to add, remove, configure, and monitor LLM tools (e.g., grep, file read, bash execution) at runtime. This transforms the TUI from a static code editor into a true agentic development environment.

---

## Current State

**Completed Phases:**
- ✅ **Phase 1**: Conversation DAG foundation (undo/redo/edit/branches)
- ✅ **Phase 2**: Backtracking UI (history selection, replay from here, diff view)
- ✅ **Phase 5**: Auto-detection for conversation compaction
- ✅ **Phase 6**: Panel interactivity (focus navigation, editable research, planning tasks, help text)

**Current Tool System (fmpl-tui/src/main.rs):**
- Tools are hardcoded in the LLM system
- No runtime configuration
- No visibility into available tools
- No tool usage monitoring

---

## Vision: Dynamic Tool Management

Per the 12-layer architecture and unified gramars design, an agentic system needs:
1. **Tool Discovery**: See what tools are available
2. **Tool Configuration**: Add/remove/modify tools at runtime
3. **Tool Monitoring**: Track which tools are used and how often
4. **Tool Safety**: Configure execution timeouts, safety checks

This enables the TUI to adapt to different workflows without code changes.

---

## Phase 7 Deliverables

### Task 7.1: Tool Data Model (S - 1 hour)

**Goal**: Define the tool representation in the TUI.

**Implementation:**
- Add `Tool` struct to main.rs:
  ```rust
  struct Tool {
      id: String,
      name: String,
      description: String,
      enabled: bool,
      timeout_ms: u64,
      requires_confirmation: bool, // For dangerous operations
      usage_count: usize,
  }
  ```
- Add `tools: Vec<Tool>` field to `App` struct
- Initialize with default tools on startup:
  - `grep`: Search codebase
  - `file_read`: Read file contents
  - `bash_execute`: Run shell commands
  - `llm_query`: Query LLM

**Success Criteria:**
- Tool struct defined
- Default tools initialized
- Can access tools from App

---

### Task 7.2: Tool Management Panel (M - 2-3 hours)

**Goal**: Add a new panel for viewing and managing tools.

**Implementation:**
- Add `Tools` variant to `PanelType` enum
- Add `Ctrl+T` keybinding to focus tools panel
- Panel layout:
  ```
  ┌─ Tools [FOCUSED] ──────────────────────────────────┐
  │ ID  Name           Enabled  Timeout  Confirm  Use   │
  │ ─── ─────────────  ───────  ──────  ───────  ───   │
  │ 1   grep           ✓        30s      ✗        15   │
  │ 2   file_read      ✓        10s      ✗        42   │
  │ 3   bash_execute   ✓        60s      ✓        7    │
  │ 4   llm_query      ✓        120s     ✗        23   │
  │                                                        │
  │ Enter: toggle | e: edit | a: add | d: delete           │
  └────────────────────────────────────────────────────────┘
  ```

**Keybindings when Tools panel focused:**
- `Enter`: Toggle tool enabled/disabled
- `e`: Edit tool configuration (timeout, confirmation)
- `a`: Add new tool
- `d`: Delete tool
- Arrow up/down: Select tool

**Success Criteria:**
- Tools panel displays all tools
- Can navigate tool list
- Visual indicator for enabled/disabled
- Shows usage statistics

---

### Task 7.3: Tool Configuration UI (M - 2-3 hours)

**Goal**: Allow editing tool settings.

**Implementation:**
- Add `tool_config_mode: bool` to `App`
- When `e` pressed on selected tool:
  - Show configuration form in panel
  - Editable fields: name, description, timeout, confirmation
  - `Esc`: Cancel, `Enter`: Save

**Configuration UI:**
```
┌─ Tool Configuration ────────────────────────────────┐
│ Name:        bash_execute                          │
│ Description: Execute shell commands                │
│ Timeout:     60s                                   │
│ Confirm:     [✓] Required                          │
│                                                        │
│ Esc: cancel | Enter: save                            │
└────────────────────────────────────────────────────┘
```

**Success Criteria:**
- Can edit tool name/description
- Can adjust timeout
- Can toggle confirmation requirement
- Changes persist to `.agent/tools.json`

---

### Task 7.4: Tool Persistence (S - 1 hour)

**Goal**: Save/load tool configuration.

**Implementation:**
- Save to `.agent/tools.json` on change
- Load on startup
- Format:
  ```json
  {
    "tools": [
      {
        "id": "grep",
        "name": "grep",
        "description": "Search codebase",
        "enabled": true,
        "timeout_ms": 30000,
        "requires_confirmation": false,
        "usage_count": 15
      }
    ]
  }
  ```

**Success Criteria:**
- Tools saved to `.agent/tools.json`
- Loaded on startup
- Format is human-editable

---

### Task 7.5: Tool Usage Tracking (S - 1 hour)

**Goal**: Monitor and display tool usage.

**Implementation:**
- Increment `usage_count` when tool is invoked
- Display in tools panel
- Add `u` keybinding to reset usage stats

**Success Criteria:**
- Usage count increments on tool use
- Displayed in tools panel
- Can reset stats

---

## Phase 7 Scope (In vs Out)

### In Scope
- Tool data model (Task 7.1)
- Tools panel with list view (Task 7.2)
- Tool configuration UI (Task 7.3)
- Persistence to `.agent/tools.json` (Task 7.4)
- Usage tracking (Task 7.5)

### Out of Scope (Future Phases)
- Actual tool execution (already exists)
- Tool result caching
- Tool chaining/pipelines
- Custom tool scripting
- Tool marketplace/sharing

---

## Implementation Order

**Priority 1 (Foundation):**
1. Task 7.1: Tool data model - enables all other tasks
2. Task 7.2: Tools panel - UI for tool management

**Priority 2 (Functionality):**
3. Task 7.4: Tool persistence - saves configuration
4. Task 7.3: Tool configuration UI - editing tools

**Priority 3 (Monitoring):**
5. Task 7.5: Usage tracking - analytics

---

## Testing Strategy

### Manual Testing
- Build: `cargo build --release`
- Run: `cargo run --release`
- Test: Can switch to tools panel (Ctrl+T)
- Test: Can toggle tools (Enter)
- Test: Can edit tools (e)
- Test: Can add/delete tools
- Test: Changes persist to `.agent/tools.json`

### Smoke Tests
- App starts without panic
- Tools panel displays default tools
- Can navigate tool list
- Configuration form appears/disappears correctly
- `.agent/tools.json` created

---

## Success Criteria

Phase 7 is complete when:

1. **Tool Panel**: Can access tools panel via Ctrl+T
2. **Tool List**: Default tools displayed with all fields
3. **Tool Toggle**: Can enable/disable tools with Enter
4. **Tool Edit**: Can edit tool configuration (timeout, confirmation)
5. **Persistence**: `.agent/tools.json` created and loaded
6. **Usage Tracking**: Usage count displayed and increments
7. **No Regressions**: All 222 existing tests pass
8. **Build Clean**: `cargo build --release` succeeds

---

## Next Steps (Phase 8+)

After Phase 7:
- **Phase 8**: LLM-assisted research/planning generation (use tools to auto-generate content)
- **Phase 3**: VCS-style operations (branch switching, merge)
- **Phase 4**: Context compaction UI (relevance scoring, elision)

---

## References

- [Phase 6: Panel Interactivity](./2026-01-21-phase6-panel-interactivity.md) - Panel navigation system
- [Unified Grammars and Agents Design](./2026-01-19-unified-grammars-and-agents-design.md) - Agent control flow
- [12-Layer Human-AI Architecture](./12-layer-human-ai-architecture.md) - Overall vision
- [FMPL TUI Implementation](../fmpl-tui/src/main.rs) - Current code
