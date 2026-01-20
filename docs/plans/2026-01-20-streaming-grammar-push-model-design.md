# Streaming Grammar Push-Model Design

## Overview

This document describes push-based streaming grammars for FMPL, enabling incremental parsing of async streams (LLM output, HTTP chunks, etc.) with full backtracking and durable suspension.

**Core insight:** Grammars over push streams work like Unix pipes. Each match emits downstream immediately. Backtracking is unlimited with speculative execution. External calls are memoized as part of standard packrat caching.

---

## Core Model

**Streaming grammars operate on push streams with full backtracking.**

```fmpl
llm_stream |> parser.tool_call |> execute_tool
```

- Each value from `llm_stream` pushes into `parser.tool_call`
- When `tool_call` fully matches, its result pushes to `execute_tool`
- The pipe keeps feeding input until upstream ends
- No `*` needed for repetition - implicit in piping (like Unix pipes)

**Backtracking:**

- Grammar can backtrack arbitrarily over buffered input
- Large buffers spill to Fjall (not just memory)
- Speculative results flow downstream
- If backtracking invalidates results, the pipeline rewinds

**Termination:**

The grammar rule itself dictates termination. It keeps parsing until:
1. It fully matches its pattern, or
2. The input stream ends

---

## Memoization

**Standard packrat memoization handles external calls naturally.**

Cache key: `(position, rule_name)`
Cache value: `ParseResult` (includes semantic action results)

When a rule containing `<- curl.get(url)` matches:
- The call executes as part of the semantic action
- The entire rule result (including call result) is memoized
- Backtracking to same `(position, rule)` returns cached result
- No separate call cache needed

This follows the Haskell IO monad model: effects happen when reached, backtracking doesn't undo them, but memoization prevents re-execution.

**Fjall persistence:**

The memo table spills to Fjall alongside the input buffer. Resume a suspended parse = restore memo table + input buffer + parse state.

---

## Implementation Architecture

### 1. Pipeline Wiring

Extend `|>` behavior when left operand is `AsyncStream` and right operand is a grammar rule:

1. Create output channel for match results
2. Spawn parse driver that:
   - Reads from input stream (existing `StreamingInput`)
   - Runs grammar incrementally
   - On each complete match, sends result to output channel
3. Return the output channel as new `AsyncStream`

When `|>` connects `AsyncStream` to a function:
- For each value from stream, apply function
- Send function result to output channel

### 2. Incremental Parse Driver

Current `PegRuntime::parse()` runs to completion. For push streams, the parser must suspend when input isn't available yet.

**New API:**

```rust
impl<'a, 'e, I: PegInput> PegRuntime<'a, 'e, I> {
    /// Start incremental parse
    pub fn start(&mut self, rule: &str) -> ParseState<I::Position>;

    /// Continue parsing from state
    pub fn resume(&mut self, state: ParseState<I::Position>) -> ParseNext<I::Position>;
}

pub enum ParseNext<P> {
    /// Rule matched, here's the result
    Match(Value),
    /// Need more input, here's state to resume
    NeedInput(ParseState<P>),
    /// Input ended
    End,
}
```

**Parse state to capture:**

```rust
pub struct ParseState<P: Clone> {
    /// Current position in input
    position: P,
    /// Rule call stack (for nested rule application)
    rule_stack: Vec<(SmolStr, P)>,
    /// Current bindings
    bindings: HashMap<SmolStr, Value>,
}
```

**The parse driver loop:**

```rust
loop {
    match runtime.resume(state)? {
        ParseNext::Match(value) => {
            output.send(value).await;
            state = runtime.start(rule); // Reset for next match
        },
        ParseNext::NeedInput(new_state) => {
            state = new_state;
            // Wait for more input
        },
        ParseNext::End => break,
    }
}
```

### 3. Fjall Backing for Buffers

**Input buffer persistence:**

Currently `StreamPosition` caches positions in `Vec<Rc<StreamPosition>>`. Add Fjall partition for overflow:

```rust
enum StreamSource {
    Async {
        handle: Mutex<StreamHandle>,
        timeout: Option<Duration>,
        /// In-memory positions (recent)
        positions: Mutex<Vec<Rc<StreamPosition>>>,
        /// Fjall partition for spilled positions
        overflow: Option<fjall::Partition>,
        /// Threshold for spilling to Fjall
        memory_limit: usize,
    },
    // ...
}
```

Position lookup: check memory first, then Fjall.

**Memo table persistence:**

Similarly, the per-position memo tables can spill to Fjall for long-running parses.

### 4. Parse State Serialization

`ParseState` must serialize for durable suspension:

```rust
impl<P: Serialize> Serialize for ParseState<P> { ... }
impl<P: Deserialize> Deserialize for ParseState<P> { ... }
```

Resume a suspended agent = deserialize:
- Input buffer (positions)
- Memo table
- Parse state

Continue exactly where left off.

---

## What Changes

| Component | Change |
|-----------|--------|
| `PegRuntime` | Add `start()` / `resume()` incremental API |
| `StreamPosition` | Add Fjall backing for positions vector |
| `\|>` operator | Extend to wire AsyncStream → grammar → output stream |
| Memo table | Add Fjall persistence |
| `ParseState` | New struct, serializable |

## What Stays the Same

- `PegInput` trait and implementations
- Pattern matching logic
- Semantic action evaluation
- Existing pull-based parsing (still works)

---

## Testing Strategy

1. **Unit tests:** Incremental parse API with mock input
2. **Integration tests:** Full pipeline with simulated LLM token stream
3. **Persistence tests:** Suspend mid-parse, restore from Fjall, continue
4. **Backtracking tests:** Verify memoization prevents duplicate external calls

---

## References

- [Unified Grammars and Agents Design](2026-01-19-unified-grammars-and-agents-design.md) - Agentic patterns
- [Async/Await/Spawn Design](2026-01-20-async-await-spawn-design.md) - Stream infrastructure
- [LogicT](https://hackage.haskell.org/package/logict) - Haskell backtracking with IO
- [OMeta](http://www.tinlizzie.org/~awarth/papers/dls07.pdf) - Packrat parsing with memoization
