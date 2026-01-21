# Phase 8: LLM-Assisted Research & Planning

**Status**: Draft
**Size**: M (2-3 hours)
**Dependencies**: Phase 6 (Panel Interactivity), Phase 7 (Tool Management)

## Overview

Add LLM-powered assistance to the TUI for generating research notes and planning tasks based on the current conversation context. This creates an intelligent assistant that can help users organize their thoughts and create actionable plans.

## Tasks

### Task 8.1: LLM Integration Helper Functions (S - 1 hour)

**Goal**: Create helper functions for calling the LLM with the current conversation context.

**Implementation**:
- Add `generate_research_summary(conversation: &ConversationNode) -> Result<String>`
  - Extracts recent messages from the conversation DAG
  - Formats them as a prompt: "Summarize the key points from this conversation for research notes"
  - Calls the LLM via the existing `llm_query` tool
  - Returns the summary as a String

- Add `generate_planning_tasks(conversation: &ConversationNode) -> Result<Vec<String>>`
  - Extracts recent messages and current context
  - Formats prompt: "Based on this conversation, generate a list of actionable tasks"
  - Calls LLM and parses response into task descriptions
  - Returns Vec<String> of task descriptions

**Location**: `fmpl-tui/src/main.rs`

**Data Flow**:
```
ConversationNode → format_prompt() → llm_query() → parse_response() → String/Vec<String>
```

### Task 8.2: Research Panel LLM Assistance (S - 30 min)

**Goal**: Add Ctrl+G (Generate) keybinding to research panel to auto-generate research notes.

**Implementation**:
- Add `g` keybinding to research panel (when focused)
  - Calls `generate_research_summary(current_node)`
  - Appends generated text to `research_lines`
  - Shows status message: "Generated research summary"

- Add visual indicator in research panel header when generation is available
  - Show hint: "Ctrl+G: Generate summary"

**Location**: `fmpl-tui/src/main.rs` keybinding handler

**User Experience**:
1. User has conversation in history
2. User focuses Research panel (Ctrl+R)
3. User presses `g`
4. LLM analyzes conversation and generates summary
5. Summary appears in research panel for editing

### Task 8.3: Planning Panel LLM Assistance (S - 30 min)

**Goal**: Add Ctrl+G (Generate) keybinding to planning panel to auto-generate tasks.

**Implementation**:
- Add `g` keybinding to planning panel (when focused)
  - Calls `generate_planning_tasks(current_node)`
  - Creates new `PlanningTask` objects from generated descriptions
  - Adds tasks to `planning_tasks` vector
  - Shows status message: "Generated N tasks"

- Add visual indicator in planning panel header when generation is available
  - Show hint: "Ctrl+G: Generate tasks"

**Location**: `fmpl-tui/src/main.rs` keybinding handler

**User Experience**:
1. User has conversation about what needs to be done
2. User focuses Planning panel (Ctrl+P)
3. User presses `g`
4. LLM analyzes conversation and generates task list
5. Tasks appear in planning panel for editing/prioritization

### Task 8.4: LLM Response Display (S - 30 min)

**Goal**: Show LLM generation in progress and handle errors gracefully.

**Implementation**:
- Add `llm_generation_status: Option<String>` to App struct
  - Tracks current LLM operation ("Generating research summary...", "Generating tasks...")
  - Cleared when generation completes

- Add error handling for LLM failures
  - Show error message in status bar: "LLM generation failed: {error}"
  - Don't crash or leave panel in bad state

- Add visual indicator during generation
  - Show loading spinner or status in panel header
  - Disable other keybindings during generation (or queue them)

**Location**: `fmpl-tui/src/main.rs` App struct and render functions

## Data Structures

### App Struct Additions

```rust
struct App {
    // ... existing fields ...

    // LLM generation state
    llm_generation_status: Option<String>,
}
```

### Helper Functions

```rust
impl App {
    // Generate research summary from conversation context
    fn generate_research_summary(&mut self) -> Result<()> {
        let conversation_context = self.format_conversation_for_llm();
        let prompt = format!(
            "Summarize the key points from this conversation for research notes:\n\n{}",
            conversation_context
        );

        let summary = self.call_llm(&prompt)?;
        self.research_lines.extend(summary.lines().map(|s| s.to_string()));
        self.save_research()?;
        Ok(())
    }

    // Generate planning tasks from conversation context
    fn generate_planning_tasks(&mut self) -> Result<()> {
        let conversation_context = self.format_conversation_for_llm();
        let prompt = format!(
            "Based on this conversation, generate a list of actionable tasks (one per line):\n\n{}",
            conversation_context
        );

        let tasks = self.call_llm(&prompt)?;
        for task_description in tasks.lines() {
            if !task_description.is_empty() {
                self.planning_tasks.push(PlanningTask {
                    id: self.next_task_id,
                    description: task_description.to_string(),
                    status: TaskStatus::Pending,
                    priority: TaskPriority::Medium,
                });
                self.next_task_id += 1;
            }
        }
        self.save_tasks()?;
        Ok(())
    }

    // Format current conversation context for LLM
    fn format_conversation_for_llm(&self) -> String {
        // Extract recent messages from current branch
        // Format as readable transcript
        // Return as string
    }

    // Call LLM with prompt (using existing llm_query tool)
    fn call_llm(&self, prompt: &str) -> Result<String> {
        // Use existing LLM integration
        // Return response as string
    }
}
```

## Keybindings

| Panel | Keybinding | Action |
|-------|-----------|--------|
| Research | `g` | Generate research summary from conversation |
| Planning | `g` | Generate tasks from conversation |

## Error Handling

- LLM call fails → Show error in status bar, don't modify data
- Empty conversation → Show "No conversation to analyze"
- LLM returns empty response → Show "LLM returned empty response"

## Testing

- Generate research summary from sample conversation
- Generate planning tasks from sample conversation
- Test error handling (LLM unavailable, network error)
- Test with empty conversation
- Test with long conversation (verify context window handling)

## Success Criteria

- [ ] Press `g` in Research panel generates summary from conversation
- [ ] Press `g` in Planning panel generates tasks from conversation
- [ ] Generated content is editable and persists
- [ ] Errors are handled gracefully
- [ ] Build clean, all tests passing

## Future Enhancements

- Allow custom prompts for generation (e.g., "Generate pros/cons list")
- Add regeneration options (replace vs append)
- Add conversation history filtering (only last N messages)
- Add task dependency detection from conversation
- Add priority inference from conversation tone

## Related Specs

- [Phase 6: Panel Interactivity](2026-01-21-phase6-panel-interactivity.md) - Panel focus and editing
- [Phase 7: Tool Management](2026-01-21-phase7-tool-management.md) - Tool registry
- [LLM Tool Calling](../specs/llm-tool-calling.md) - LLM integration
