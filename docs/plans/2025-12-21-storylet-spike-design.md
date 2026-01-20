# Storylet Spike Design

## Overview

A minimal spike to test storylets as playable web content, using FMPL for authoring and rendering, with Fjall-based persistence. For broader context, see the North Star section in `docs/plans/2025-12-19-fmpl-revival-design.md`.

## Architecture

```
┌─────────────────────────────────────────┐
│  FMPL (storylets, rendering, logic)     │  ← Story content
├─────────────────────────────────────────┤
│  Rust Engine (HTTP, sessions, VM host)  │  ← Plumbing
├─────────────────────────────────────────┤
│  Fjall Store (objects, continuations)   │  ← Persistence
└─────────────────────────────────────────┘
```

**Request flow:**
1. `GET /play/{token}` arrives
2. Engine validates session, looks up token in Fjall → deserializes VM state
3. Engine calls current storylet's `render(ctx)` method
4. FMPL returns HTML string with choice links like `/play/{new_token}`
5. Engine serves HTML

**First visit:** `GET /play` (no token) → Engine creates fresh VM, loads starting storylet, stores snapshot, redirects to `/play/{token}`.

## Storylet Object Model

Sigil-free syntax for approachable authoring:

```fmpl
object crossroads <: storylet {
  prose: "You stand where two paths diverge in a yellow wood."

  choices: [
    %{action: "Take the left path", target: forest_clearing},
    %{action: "Take the right path", target: mountain_pass},
    %{action: "Turn back", target: village}
  ]
}

object forest_clearing <: storylet {
  prose: "Sunlight filters through the canopy. A stream babbles nearby."

  choices: [
    %{action: "Follow the stream", target: waterfall},
    %{action: "Rest here", target: self.rest},
    %{action: "Return to crossroads", target: crossroads}
  ]
}
```

**Choice model:**
- `action`: Label text shown to player
- `target`: Destination object, or object.method
- Defaults: `target: foo` means `foo.enter()`
- Self-reference: `target: self.examine` calls method on current storylet

**Base storylet provides render:**

```fmpl
object storylet {
  render(ctx): {
    "<article class='storylet'>" +
      "<p>" + self.prose + "</p>" +
      "<nav>" + self.render_choices(ctx) + "</nav>" +
    "</article>"
  }

  render_choices(ctx): {
    self.choices
      |> map(\c "<a href='" + ctx.link(c.target) + "'>" + c.action + "</a>")
      |> join("\n")
  }
}
```

## Session & Continuation Model

Continuation sessions - the opaque token points to a server-stored snapshot.

**Serialized state contains:**
- Current storylet reference
- Player object state
- Global bindings
- Full VM stack

**Flow:**
```
Player clicks "Take the left path"
         ↓
GET /play/9Xq5gF...
         ↓
Engine validates session, looks up token in Fjall → deserializes VM state
         ↓
VM state says: current = crossroads, choice target = forest_clearing
         ↓
Engine sets current = forest_clearing
         ↓
Engine calls forest_clearing.render(ctx)
         ↓
During render, ctx.link(waterfall) stores new state → 7cVt1Q...
         ↓
HTML returned with href="/play/7cVt1Q..."
         ↓
Engine stores new snapshot in Fjall, serves HTML
```

**Benefits:**
- Session-scoped safety: tokens are validated against the current session
- Time travel: old URLs still work within retention policy
- Back button works naturally

## Player Model

Player is a persistent object in VM state:

```fmpl
player: {
  name: "Anonymous"
}
```

Accessible in storylets:
```fmpl
prose: "Welcome, " + player.name + "."
```

Via context:
```fmpl
ctx.player.name
```

## Fjall Storage Model

**Image store (objects):**

| Key | Value |
|-----|-------|
| `obj:storylet` | Base storylet object (serialized) |
| `obj:crossroads` | Storylet instance |
| `obj:forest_clearing` | Storylet instance |

**Session store (states):**

| Key | Value |
|-----|-------|
| `cont:{session_id}:{token}` | Snapshot envelope + serialized VM state |

**Startup flow:**
1. Open Fjall store
2. Check if `obj:storylet` exists
3. If empty: load seed `.fmpl` file, parse, store objects
4. If populated: ready to serve

## Context Builtins

Engine passes `ctx` object to FMPL with:

```fmpl
ctx.link(target)           -- generate continuation URL
ctx.link(target.method)    -- with explicit method
ctx.static("path/to/file") -- static asset URL
ctx.store_blob(data, mime) -- store blob, return URL (future)
ctx.player                 -- current player object
ctx.current                -- current storylet reference
```

## HTTP Routes

```
GET  /play           → fresh game, redirect to /play/{token}
GET  /play/{token}   → validate session, lookup state, render current storylet
GET  /static/{path}  → serve static files
GET  /blob/{hash}    → serve stored blobs (future)
```

## Event Streams (Spike Summary)

The spike uses the stream model described in the North Star design: streams are primitive values created with `stream { ... }`, and operators (`map`, `filter`, `flatMap`, `reduce`) are core language primitives. For the spike, the only event source is HTTP; each request is a tick, and evaluation is synchronous within that tick. Grammar application can be used inside streams via `parse(g.rule)` or `flatMap(\x x @ g.rule)`; parse failures emit nothing.

## Spike Scope

**Building:**
- Fjall image store for objects
- Fjall continuation store for snapshots
- Seed loader (parse `.fmpl` → store in Fjall if empty)
- VM/object serialization
- Context builtins: `ctx.link()`, `ctx.static()`, `ctx.player`
- Base `storylet` object with default render
- 3-4 sample storylets
- HTML shell with basic styling
- Single-vat execution model (no cross-vat or eventual-send semantics)
- Stream primitives (map/filter/flatMap/reduce) available, but limited to HTTP event sources

**NOT building (yet):**
- Storylet DSL grammar (use full FMPL syntax)
- Qualities/action economy
- Blob storage
- Live editing
- String interpolation / embedded HTML
- Multiple players

## Success Criteria

- First run: seed from `.fmpl` file into Fjall
- Subsequent runs: load from Fjall directly
- Navigate storylets via links
- Back button works (within retention policy)
- Restart server → state persists

## Serialization

**Primary:** rkyv for zero-copy deserialization (fast snapshot lookups)
**Secondary (optional):** serde/JSON for human-readable dumps

Types should default to `rkyv::Archive`. Only add `serde::Serialize` when a JSON/debug export path is required.

## Snapshot Envelope

All continuation snapshots are stored with a versioned envelope to support migrations.

```text
SnapshotEnvelope {
  schema_version: u16,
  bytecode_version: u16,
  engine_version: u32,
  created_at: u64,
  payload_format: "rkyv-v1" | "serde-json-v1",
  payload: bytes
}
```

- `schema_version`: changes to VM/object graph layout
- `bytecode_version`: changes to opcode table or operand encoding
- `engine_version`: compatibility guard for runtime behavior

## Reflection

FMPL is a reflective system - all values are inspectable from within the language:

```fmpl
all_objects()                 -- enumerate all objects
object.properties()           -- list property names
object.methods()              -- list method names
object.parent                 -- walk inheritance
lambda.source()               -- view source code
object.method_source(:greet)  -- view method source
```

**Spike:** Store original source with compiled code (source blobs live alongside bytecode in the image store)
**Future:** Bytecode decompiler (like LambdaMOO) for normalized output

## Future Enhancements

| Feature | Description |
|---------|-------------|
| Bytecode decompiler | Reconstruct source from bytecode + metadata (LambdaMOO style) |
| String interpolation | `"Welcome, ${player.name}."` |
| Embedded HTML | Rich prose without escaping |
| Storylet DSL | Simpler authoring syntax with grammar |
| Qualities | Stats, inventory, progress tracking |
| Action economy | Energy pools (Focus, Stamina, Social) |
| Live editing | Modify objects via REPL while running |
| Multiple players | Separate player objects, shared world |
