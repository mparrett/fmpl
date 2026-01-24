# Phase 9: LLM Tool Execution

**Status**: ✅ COMPLETE (2026-01-24)
**Goal**: Enable the TUI to execute managed tools via LLM requests, making the application truly "agentic" rather than just a chat interface.

**Context**: Phases 6-8 built the infrastructure (panels, tools, LLM integration), but the LLM cannot yet invoke the tools that are managed. This phase closes that loop.

**Size**: L (4-6 hours)

## Tasks

### Task 9.1: Tool Execution Request Parsing (M - 2 hours)

**Goal**: Detect and parse tool execution requests from LLM responses.

**Specification**:
- Define tool request format (e.g., `TOOL:<tool_id>:<args>` or JSON)
- Parse LLM response messages for tool requests
- Extract tool_id and arguments
- Validate tool exists and is enabled
- Return structured request or error

**Implementation**:
- Add `parse_tool_request()` function
- Handle multiple tool request formats:
  - Simple: `TOOL:grep:pattern:src/`
  - JSON: `{"tool": "grep", "args": {"pattern": "...", "path": "..."}}`
  - Natural language: "Search for X in Y" (future)
- Validate tool_id against `app.tools`
- Check `tool.enabled` flag
- Return `ToolRequest { tool_id, args }` or error

**Location**: `fmpl-tui/src/main.rs` after LLM helper functions

### Task 9.2: Synchronous Tool Execution (M - 2 hours)

**Goal**: Execute tools synchronously and display results.

**Specification**:
- Implement `execute_tool()` function
- Handle different tool types:
  - `grep`: shell command execution
  - `file_read`: file reading
  - `bash_execute`: command execution
  - `llm_query`: recursive LLM call (future)
- Capture output (stdout/stderr)
- Handle timeout (from tool.timeout_ms)
- Return `ToolResult { success, output, error }`

**Implementation**:
- Use `std::process::Command` for shell tools
- Use `tokio::fs` for file operations
- Use `tokio::time::timeout` for timeout handling
- Increment `tool.usage_count` on execution
- Save tools after execution (for usage tracking)

**Location**: `fmpl-tui/src/main.rs`

### Task 9.3: Tool Result Display (S - 1 hour)

**Goal**: Display tool execution results in the output panel.

**Specification**:
- Format tool results for display
- Show tool name, arguments, and output
- Highlight errors in red
- Add success/failure indicator
- Append to output panel conversation

**Implementation**:
- Add `format_tool_result()` function
- Format: `Tool: <name> <args>\n<output>` or `Tool: <name> <args> [ERROR]\n<error>`
- Add cyan/blue styling for tool name
- Add red styling for errors
- Append formatted result to output panel
- Scroll to bottom after tool result

**Location**: `fmpl-tui/src/main.rs` in output panel rendering

### Task 9.4: Multi-Tool Execution Pipeline (S - 1 hour)

**Goal**: Execute multiple tools from a single LLM response.

**Specification**:
- Detect multiple tool requests in LLM response
- Execute tools sequentially
- Display results as they complete
- Stop on first error (optional)
- Support tool chaining (output of tool N as input to tool N+1) (future)

**Implementation**:
- Parse all tool requests from LLM response
- Execute in a loop: `for request in tool_requests { execute_tool(); display_result(); }`
- Add "Executing tools..." status message
- Add completion summary (e.g., "Executed 3 tools, 2 succeeded")
- Handle tool execution errors gracefully

**Location**: `fmpl-tui/src/main.rs` in LLM message handling

## Tool Request Format

### Simple Format (Phase 9)
```
TOOL:grep:pattern:src/
TOOL:file_read:path:src/main.rs
TOOL:bash:command:cargo test
```

### JSON Format (Phase 9)
```json
TOOL:{"tool": "grep", "args": {"pattern": "test", "path": "src/"}}
```

### Natural Language (Future - Phase 10+)
"Search for 'test' in the src directory"
"Read the main.rs file"
"Run cargo test"

## Error Handling

- Tool not found: `Error: Tool 'xyz' not found`
- Tool disabled: `Error: Tool 'xyz' is disabled`
- Invalid arguments: `Error: Invalid arguments for tool 'xyz'`
- Execution timeout: `Error: Tool 'xyz' timed out after 5000ms`
- Command failure: `Error: Tool 'xyz' failed with exit code 1`

## User Workflow

1. User enters chat mode (Ctrl+L)
2. User asks LLM: "Find all test functions in src/"
3. LLM responds with: "I'll search for test functions. TOOL:grep:^pub fn test:src/"
4. System detects tool request, executes grep
5. System displays: `Tool: grep ^pub fn test src/\n<grep output>`
6. LLM can now analyze the grep results and respond
7. User can review results in output panel

## Keybindings

- None new (tool execution automatic from LLM responses)
- Future: Ctrl+X to cancel running tool (if async)
- Future: Ctrl+R to retry failed tool

## Data Structures

```rust
struct ToolRequest {
    tool_id: String,
    args: Vec<String>,
}

struct ToolResult {
    success: bool,
    output: String,
    error: Option<String>,
    duration_ms: u64,
}
```

## Testing

- Unit test: `parse_tool_request()` with various formats
- Unit test: Tool execution with mock tools
- Integration test: Full tool execution pipeline
- Manual test: Chat with LLM, request tool execution

## Success Criteria

- [x] LLM can invoke managed tools via TOOL: prefix
- [x] Tool results displayed in output panel
- [x] Tool errors displayed clearly
- [x] Usage count increments on execution
- [x] All tests pass
- [x] Build clean (cargo build --release)

## Future Phases

- Phase 10: Asynchronous tool execution with progress indication
- Phase 11: Natural language tool request parsing
- Phase 12: Tool chaining (output pipelining)
- Phase 13: Tool composition (macros/scripts)

## Dependencies

- Requires: Phase 7 (tool management)
- Requires: Phase 8 (LLM integration)
- Builds on: Existing tool data model
- Builds on: Existing LLM chat infrastructure
