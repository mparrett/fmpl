# FMPL TUI - Multi-line Code Editor

## Features

### Multi-line Code Editor
- **Arrow keys**: Navigate up/down/left/right
- **Enter**: Insert new line (in EDIT MODE)
- **Esc + Enter**: Execute code (switch to EXECUTE MODE, then Enter)
- **Tab**: Insert 4 spaces (indentation)
- **Backspace**: Delete character, merge lines if at start
- **Delete**: Delete character at cursor
- **Home/End**: Jump to start/end of line
- **Line numbers**: Displayed on left side
- **Cursor**: Yellow highlight on current character
- **Scrolling**: Automatic when cursor moves beyond visible area

### Mode Switching
- **EDIT MODE** (default): Enter inserts new lines
- **EXECUTE MODE**: Press Esc to toggle, then Enter to execute code

### Three-Panel Layout
1. **Research View** - Problem space analysis
2. **Planning View** - Collaborative scope definition
3. **Execution View** - Split into:
   - Code Editor (left)
   - Execution Output (right)

## Testing

Run the TUI:
```bash
cargo run --bin fmpl-tui
```

### Test Program

Enter the following multi-line FMPL code (Esc+Enter to execute):

```
let add = \x \y x + y
let result = add(10, 20)
result
```

Expected output: `30`

### Key Bindings Reference

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Esc` | Toggle EDIT/EXECUTE mode |
| `Enter` | New line (EDIT) or Execute (EXECUTE) |
| `↑↓←→` | Navigate |
| `Home/End` | Jump to line start/end |
| `Tab` | Insert 4 spaces |
| `Backspace` | Delete backward |
| `Delete` | Delete forward |

## Architecture

### Layer 1: Input Layer
- ✅ Three-panel layout (Research, Planning, Execution)
- ✅ Multi-line code editor with cursor management
- ✅ Real-time FMPL execution

### Next Steps (Future Work)

**Layer 2: Contextual Layer**
- [ ] Revision history with VCS-style branching
- [ ] Automated backtrack detection
- [ ] Context compaction and elision

**Layer 3: Agent Description/Dataflow**
- [ ] FMPL language integration
- [ ] Grammar-based agent control

**Layer 4: Tooling Layer**
- [ ] Tool management interface
- [ ] External tool integration (MCP/ACP)

**LLM Integration**
- [ ] Provider switching (Ollama, Anthropic)
- [ ] Multi-turn conversation support
- [ ] Agent→tool loop tracing
