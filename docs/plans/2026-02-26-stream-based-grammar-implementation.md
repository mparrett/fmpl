# Stream-Based Grammar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a `ParseStream` type with 8 primitives and combinators, tested via TDD, proving that grammars can compile to function calls against streams.

**Architecture:** Add `ParseStream` as a new `Value` variant wrapping an input value with position, checkpoint stack, and memo table. Dispatch 8 methods via the existing `call_method` pattern in `vm.rs`. Combinators are Rust functions callable from FMPL. Everything is tested end-to-end through the existing `eval()` pattern.

**Tech Stack:** Rust, `SmolStr` for interned symbols, existing `Value` enum, existing VM `call_method` dispatch.

---

### Task 1: Create ParseStream Rust Type

**Files:**
- Create: `fmpl-core/src/parse_stream.rs`
- Modify: `fmpl-core/src/lib.rs:28` (add module declaration)
- Modify: `fmpl-core/src/value.rs:16-66` (add Value variant)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

Create `fmpl-core/tests/stream_parsing.rs`:

```rust
use fmpl_core::{Compiler, Lexer, Parser, Result, Value, Vm};

fn eval(vm: &mut Vm, source: &str) -> Result<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::with_source(&tokens, source).parse()?;
    let code = Compiler::new().compile(&ast)?;
    vm.run(&code)
}

#[test]
fn parse_stream_from_string_head() {
    let mut vm = Vm::new();
    // stream::new("hello") creates a ParseStream from a string
    // .head() returns the first character as a string
    let result = eval(&mut vm, r#"let s = stream::new("hello"); s.head()"#).unwrap();
    assert_eq!(result, Value::String("h".into()));
}

#[test]
fn parse_stream_from_string_position() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"let s = stream::new("hello"); s.position()"#).unwrap();
    assert_eq!(result, Value::Int(0));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `stream::new` is not a recognized builtin

**Step 3: Create the ParseStream type**

Create `fmpl-core/src/parse_stream.rs`:

```rust
//! ParseStream: unified stream type for grammar parsing.
//!
//! Wraps any iterable Value (String, List, Tagged) with position tracking,
//! checkpoint/restore for backtracking, and a packrat memo table.

use crate::error::{Error, Result};
use crate::grammar::input::MemoEntry;
use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

/// Key for the packrat memo table.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoKey {
    pub position: usize,
    pub rule_id: u64,
}

/// A checkpoint for backtracking.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub position: usize,
}

/// Unified parse stream over any iterable Value.
#[derive(Debug, Clone)]
pub struct ParseStream {
    /// The source value being streamed over.
    source: Value,
    /// Current position in the source.
    position: usize,
    /// Packrat memoization table.
    memo: HashMap<MemoKey, MemoEntry>,
}

impl ParseStream {
    /// Create a new ParseStream from any value.
    pub fn new(source: Value) -> Self {
        Self {
            source,
            position: 0,
            memo: HashMap::new(),
        }
    }

    /// Get the current item without consuming it.
    /// Returns Null at end of input.
    pub fn head(&self) -> Value {
        match &self.source {
            Value::String(s) => {
                if let Some(ch) = s[self.position..].chars().next() {
                    Value::String(SmolStr::new(ch.to_string()))
                } else {
                    Value::Null
                }
            }
            Value::List(items) => {
                if self.position < items.len() {
                    items[self.position].clone()
                } else {
                    Value::Null
                }
            }
            Value::Tagged(_, _) => {
                if self.position == 0 {
                    self.source.clone()
                } else {
                    Value::Null
                }
            }
            // Single value: treat as one-element stream
            other => {
                if self.position == 0 {
                    other.clone()
                } else {
                    Value::Null
                }
            }
        }
    }

    /// Advance position by n items.
    pub fn advance(&mut self, n: usize) {
        match &self.source {
            Value::String(s) => {
                // Advance by n characters (not bytes)
                let mut chars = s[self.position..].chars();
                let mut bytes = 0;
                for _ in 0..n {
                    if let Some(ch) = chars.next() {
                        bytes += ch.len_utf8();
                    }
                }
                self.position += bytes;
            }
            _ => {
                self.position += n;
            }
        }
    }

    /// Save current position for backtracking.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            position: self.position,
        }
    }

    /// Restore to a previously saved checkpoint.
    pub fn restore(&mut self, cp: &Checkpoint) {
        self.position = cp.position;
    }

    /// Get current position.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if at end of input.
    pub fn is_at_end(&self) -> bool {
        match &self.source {
            Value::String(s) => self.position >= s.len(),
            Value::List(items) => self.position >= items.len(),
            Value::Tagged(_, _) => self.position >= 1,
            _ => self.position >= 1,
        }
    }

    /// Access the memo table (for apply).
    pub fn get_memo(&self, key: &MemoKey) -> Option<&MemoEntry> {
        self.memo.get(key)
    }

    /// Store in the memo table (for apply).
    pub fn set_memo(&mut self, key: MemoKey, entry: MemoEntry) {
        self.memo.insert(key, entry);
    }
}
```

**Step 4: Add Value variant and module declaration**

In `fmpl-core/src/lib.rs`, add after `pub mod parser;` (around line 26):

```rust
pub mod parse_stream;
```

And in the exports section (around line 42), add:

```rust
pub use parse_stream::ParseStream;
```

In `fmpl-core/src/value.rs`, add to the Value enum (after the `Cursor` variant, around line 62):

```rust
ParseStream(Arc<std::sync::Mutex<crate::parse_stream::ParseStream>>),
```

The `Mutex` is needed because methods like `advance()` and `restore()` mutate the stream.

**Step 5: Add `stream::new` builtin**

In `fmpl-core/src/vm.rs`, find the `call_builtin` method. Add a new case for `"__builtin_stream"`:

```rust
"__builtin_stream" => {
    match name {
        "new" => {
            if args.len() != 1 {
                return Err(Error::Runtime("stream::new requires 1 argument".into()));
            }
            let ps = crate::parse_stream::ParseStream::new(args.into_iter().next().unwrap());
            let val = Value::ParseStream(Arc::new(std::sync::Mutex::new(ps)));
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(val);
        }
        _ => return Err(Error::UndefinedMethod(name.to_string())),
    }
}
```

Also register `"stream"` as a builtin symbol in the VM's `new()` or initialization method, mapping it to `Value::Symbol(SmolStr::new("__builtin_stream"))`.

**Step 6: Add method dispatch for ParseStream**

In `fmpl-core/src/vm.rs`, in the `call_method` function (after the `Value::String` match arm around line 3838), add:

```rust
Value::ParseStream(ps) => {
    let result = match name {
        "head" => {
            let stream = ps.lock().unwrap();
            stream.head()
        }
        "position" => {
            let stream = ps.lock().unwrap();
            Value::Int(stream.position() as i64)
        }
        _ => return Err(Error::UndefinedMethod(name.to_string())),
    };
    let frame = self.frames.last_mut().unwrap();
    frame.set_current(result);
}
```

**Step 7: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: 2 tests PASS

**Step 8: Commit**

```bash
jj describe -m "feat(stream): add ParseStream type with head() and position()"
jj new
```

---

### Task 2: Advance and Checkpoint/Restore

**Files:**
- Modify: `fmpl-core/src/vm.rs` (add method arms)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing tests**

Add to `fmpl-core/tests/stream_parsing.rs`:

```rust
#[test]
fn parse_stream_advance_then_head() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("hello")
        s.advance(1)
        s.head()
    "#).unwrap();
    assert_eq!(result, Value::String("e".into()));
}

#[test]
fn parse_stream_checkpoint_restore() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("hello")
        s.advance(2)
        let cp = s.checkpoint()
        s.advance(2)
        s.restore(cp)
        s.head()
    "#).unwrap();
    // After advance(2), position is at 'l' (index 2)
    // checkpoint saves position 2
    // advance(2) moves to 'o' (index 4)
    // restore goes back to position 2
    // head() returns 'l'
    assert_eq!(result, Value::String("l".into()));
}

#[test]
fn parse_stream_is_at_end() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("hi")
        s.advance(2)
        s.head()
    "#).unwrap();
    // At end of input, head() returns null
    assert_eq!(result, Value::Null);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `advance`, `checkpoint`, `restore` not yet dispatched

**Step 3: Add method dispatch**

In the `Value::ParseStream(ps)` match arm in `call_method`, add:

```rust
"advance" => {
    let n = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        _ => 1,
    };
    let mut stream = ps.lock().unwrap();
    stream.advance(n);
    Value::Null
}
"checkpoint" => {
    let stream = ps.lock().unwrap();
    let cp = stream.checkpoint();
    // Store checkpoint as an Int (position) for now
    Value::Int(cp.position as i64)
}
"restore" => {
    let pos = match args.first() {
        Some(Value::Int(pos)) => *pos as usize,
        _ => return Err(Error::Runtime("restore() requires a checkpoint value".into())),
    };
    let mut stream = ps.lock().unwrap();
    stream.restore(&crate::parse_stream::Checkpoint { position: pos });
    Value::Null
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add advance(), checkpoint(), restore() to ParseStream"
jj new
```

---

### Task 3: List Input Support

**Files:**
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parse_stream_from_list_head() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new([10, 20, 30])
        s.head()
    "#).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn parse_stream_from_list_advance_head() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new([10, 20, 30])
        s.advance(1)
        s.head()
    "#).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn parse_stream_from_list_checkpoint_restore() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new([10, 20, 30])
        s.advance(1)
        let cp = s.checkpoint()
        s.advance(2)
        s.restore(cp)
        s.head()
    "#).unwrap();
    assert_eq!(result, Value::Int(20));
}
```

**Step 2: Run tests — expect PASS**

These should already pass because `ParseStream::head()` and `advance()` already handle `Value::List`. If they pass, no implementation work needed.

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: 8 tests PASS

**Step 3: Commit**

```bash
jj describe -m "test(stream): add list input tests for ParseStream"
jj new
```

---

### Task 4: Apply with Memoization

**Files:**
- Modify: `fmpl-core/src/parse_stream.rs` (add `apply` logic)
- Modify: `fmpl-core/src/vm.rs` (add `apply` dispatch)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parse_stream_apply_calls_rule() {
    let mut vm = Vm::new();
    // Define a rule as a lambda that takes a stream,
    // reads head, advances, returns it
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let rule = \s { let c = s.head(); s.advance(1); c }
        s.apply(rule)
    "#).unwrap();
    assert_eq!(result, Value::String("a".into()));
}

#[test]
fn parse_stream_apply_memoizes() {
    let mut vm = Vm::new();
    // Call apply twice at the same position with the same rule.
    // Second call should return cached result without advancing.
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let call_count = [0]
        let rule = \s {
            call_count[0] = call_count[0] + 1
            let c = s.head()
            s.advance(1)
            c
        }
        let r1 = s.apply(rule)
        s.restore(0)
        let r2 = s.apply(rule)
        %{r1: r1, r2: r2, calls: call_count[0]}
    "#).unwrap();
    // Both results should be "a", and rule should only be called once
    match &result {
        Value::Map(m) => {
            assert_eq!(m.get("r1"), Some(&Value::String("a".into())));
            assert_eq!(m.get("r2"), Some(&Value::String("a".into())));
            assert_eq!(m.get("calls"), Some(&Value::Int(1)));
        }
        _ => panic!("expected map, got {:?}", result),
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::parse_stream_apply -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `apply` not dispatched

**Step 3: Implement apply dispatch**

`apply()` is the most complex method. It must:
1. Compute a memo key from `(position, rule_identity)`
2. Check the memo table
3. If miss: call the rule lambda with the stream as argument, store result
4. If hit: restore end position, return cached result

In `fmpl-core/src/vm.rs`, the `apply` arm in `call_method` for `ParseStream`:

```rust
"apply" => {
    let rule = match args.into_iter().next() {
        Some(rule) => rule,
        None => return Err(Error::Runtime("apply() requires a rule argument".into())),
    };

    // Compute rule identity for memo key
    let rule_id = crate::parse_stream::compute_rule_identity(&rule);

    let (memo_hit, position) = {
        let stream = ps.lock().unwrap();
        let position = stream.position();
        let key = crate::parse_stream::MemoKey { position, rule_id };
        match stream.get_memo(&key) {
            Some(MemoEntry::Done(Some(value), end_pos)) => {
                // Cache hit: return value and advance to end position
                let value = value.clone();
                let end_pos = *end_pos;
                drop(stream);
                let mut stream = ps.lock().unwrap();
                stream.restore(&crate::parse_stream::Checkpoint { position: end_pos });
                let frame = self.frames.last_mut().unwrap();
                frame.set_current(value);
                return Ok(());
            }
            Some(MemoEntry::Done(None, _)) => {
                // Cached failure
                return Err(Error::ParseFailed {
                    position,
                    message: "memoized parse failure".into(),
                });
            }
            Some(MemoEntry::InProgress) => {
                // Left recursion
                return Err(Error::ParseFailed {
                    position,
                    message: "left recursion detected".into(),
                });
            }
            None => (false, position),
        }
    };

    if !memo_hit {
        // Mark in progress
        {
            let mut stream = ps.lock().unwrap();
            let key = crate::parse_stream::MemoKey { position, rule_id };
            stream.set_memo(key, MemoEntry::InProgress);
        }

        // Call the rule: push a frame that calls rule(stream)
        let stream_val = Value::ParseStream(ps.clone());
        // Use the existing call mechanism
        self.call_value(rule, vec![stream_val])?;

        // After call completes, memoize the result
        // (This needs to happen in the return path — see step 4)
    }
    return Ok(());
}
```

**Note**: The exact implementation of `apply` will require careful integration with the VM's frame-based execution model. The rule lambda is called via `call_value`, which pushes a new frame. The memo store must happen after the frame returns. This may require a special instruction or a post-return hook. The test will guide the exact implementation.

**Step 4: Add `compute_rule_identity` to parse_stream.rs**

```rust
/// Compute an identity hash for a rule value (for memo keying).
pub fn compute_rule_identity(rule: &Value) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    match rule {
        Value::Lambda(lambda) => Arc::as_ptr(lambda) as u64,
        Value::Partial(partial) => Arc::as_ptr(partial) as u64,
        _ => {
            let mut hasher = DefaultHasher::new();
            format!("{:?}", rule).hash(&mut hasher);
            hasher.finish()
        }
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing::parse_stream_apply -- --nocapture 2>&1 | tail -20`
Expected: PASS (the memoization test may need iteration)

**Step 6: Commit**

```bash
jj describe -m "feat(stream): add apply() with packrat memoization to ParseStream"
jj new
```

---

### Task 5: Parse Failure as Structured Return

**Files:**
- Modify: `fmpl-core/src/vm.rs` (failure handling in apply)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parse_stream_apply_failure_does_not_crash() {
    let mut vm = Vm::new();
    // A rule that always fails should not crash — it returns a parse failure
    // that the caller can handle
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let fail_rule = \s { stream::fail("expected digit") }
        try { s.apply(fail_rule) } catch e { "caught: " + e }
    "#).unwrap();
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn parse_stream_apply_failure_memoized() {
    let mut vm = Vm::new();
    // After a failure, calling apply at the same position returns cached failure
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let count = [0]
        let fail_rule = \s {
            count[0] = count[0] + 1
            stream::fail("nope")
        }
        try { s.apply(fail_rule) } catch e1 { null }
        try { s.apply(fail_rule) } catch e2 { null }
        count[0]
    "#).unwrap();
    // Rule should only be called once — second call returns memoized failure
    assert_eq!(result, Value::Int(1));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::parse_stream_apply_failure -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `stream::fail` not implemented

**Step 3: Add `stream::fail` builtin**

In the `"__builtin_stream"` arm of `call_builtin`:

```rust
"fail" => {
    let msg = match args.first() {
        Some(Value::String(s)) => s.clone(),
        _ => SmolStr::new("parse failure"),
    };
    return Err(Error::ParseFailed {
        position: 0,
        message: msg.to_string(),
    });
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::fail() and failure memoization"
jj new
```

---

### Task 6: match_char Combinator

**Files:**
- Modify: `fmpl-core/src/vm.rs` (add `stream::match_char` builtin)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn match_char_success() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        stream::match_char(s, "a")
    "#).unwrap();
    assert_eq!(result, Value::String("a".into()));
}

#[test]
fn match_char_failure() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        try { stream::match_char(s, "x") } catch e { "fail" }
    "#).unwrap();
    assert_eq!(result, Value::String("fail".into()));
}

#[test]
fn match_char_advances_position() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        stream::match_char(s, "a")
        s.head()
    "#).unwrap();
    assert_eq!(result, Value::String("b".into()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::match_char -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `stream::match_char` not implemented

**Step 3: Implement match_char**

In the `"__builtin_stream"` arm:

```rust
"match_char" => {
    if args.len() != 2 {
        return Err(Error::Runtime("match_char requires (stream, char)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("match_char: first arg must be a stream".into())),
    };
    let expected = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err(Error::Runtime("match_char: second arg must be a string".into())),
    };
    let mut stream = ps.lock().unwrap();
    let head = stream.head();
    match head {
        Value::String(ref c) if *c == expected => {
            stream.advance(1);
            drop(stream);
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(Value::String(expected));
        }
        _ => {
            let pos = stream.position();
            return Err(Error::ParseFailed {
                position: pos,
                message: format!("expected '{}', got {:?}", expected, head),
            });
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::match_char() combinator"
jj new
```

---

### Task 7: match_class Combinator

**Files:**
- Modify: `fmpl-core/src/vm.rs` (add `stream::match_class` builtin)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn match_class_digit() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("3abc")
        stream::match_class(s, "0-9")
    "#).unwrap();
    assert_eq!(result, Value::String("3".into()));
}

#[test]
fn match_class_letter() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        stream::match_class(s, "a-z")
    "#).unwrap();
    assert_eq!(result, Value::String("a".into()));
}

#[test]
fn match_class_failure() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        try { stream::match_class(s, "0-9") } catch e { "fail" }
    "#).unwrap();
    assert_eq!(result, Value::String("fail".into()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::match_class -- --nocapture 2>&1 | tail -20`
Expected: FAIL

**Step 3: Implement match_class**

Add a helper to `parse_stream.rs`:

```rust
/// Check if a character matches a class specification like "0-9", "a-zA-Z", etc.
pub fn char_in_class(ch: char, class: &str) -> bool {
    let chars: Vec<char> = class.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            // Range: a-z
            if ch >= chars[i] && ch <= chars[i + 2] {
                return true;
            }
            i += 3;
        } else {
            // Single char
            if ch == chars[i] {
                return true;
            }
            i += 1;
        }
    }
    false
}
```

In the `"__builtin_stream"` arm:

```rust
"match_class" => {
    if args.len() != 2 {
        return Err(Error::Runtime("match_class requires (stream, class)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("match_class: first arg must be a stream".into())),
    };
    let class = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err(Error::Runtime("match_class: second arg must be a string".into())),
    };
    let mut stream = ps.lock().unwrap();
    let head = stream.head();
    match head {
        Value::String(ref c) => {
            let ch = c.chars().next().unwrap_or('\0');
            if crate::parse_stream::char_in_class(ch, &class) {
                stream.advance(1);
                drop(stream);
                let frame = self.frames.last_mut().unwrap();
                frame.set_current(Value::String(c.clone()));
            } else {
                let pos = stream.position();
                return Err(Error::ParseFailed {
                    position: pos,
                    message: format!("expected [{}], got '{}'", class, c),
                });
            }
        }
        _ => {
            let pos = stream.position();
            return Err(Error::ParseFailed {
                position: pos,
                message: format!("expected [{}], got {:?}", class, head),
            });
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::match_class() combinator"
jj new
```

---

### Task 8: choice Combinator

**Files:**
- Modify: `fmpl-core/src/vm.rs` (add `stream::choice` builtin)
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn choice_first_matches() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let r1 = \s { stream::match_char(s, "a") }
        let r2 = \s { stream::match_char(s, "b") }
        stream::choice(s, [r1, r2])
    "#).unwrap();
    assert_eq!(result, Value::String("a".into()));
}

#[test]
fn choice_second_matches() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("bcd")
        let r1 = \s { stream::match_char(s, "a") }
        let r2 = \s { stream::match_char(s, "b") }
        stream::choice(s, [r1, r2])
    "#).unwrap();
    assert_eq!(result, Value::String("b".into()));
}

#[test]
fn choice_restores_on_failure() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("bcd")
        let r1 = \s { stream::match_char(s, "a"); stream::match_char(s, "b") }
        let r2 = \s { stream::match_char(s, "b") }
        stream::choice(s, [r1, r2])
        s.position()
    "#).unwrap();
    // r1 fails, position restores, r2 matches "b", position is 1
    assert_eq!(result, Value::Int(1));
}

#[test]
fn choice_all_fail() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("xyz")
        let r1 = \s { stream::match_char(s, "a") }
        let r2 = \s { stream::match_char(s, "b") }
        try { stream::choice(s, [r1, r2]) } catch e { "all failed" }
    "#).unwrap();
    assert_eq!(result, Value::String("all failed".into()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::choice -- --nocapture 2>&1 | tail -20`
Expected: FAIL

**Step 3: Implement choice**

`choice` takes a stream and a list of rule lambdas. It tries each in order, checkpointing before each attempt and restoring on failure.

In the `"__builtin_stream"` arm:

```rust
"choice" => {
    if args.len() != 2 {
        return Err(Error::Runtime("choice requires (stream, alternatives)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("choice: first arg must be a stream".into())),
    };
    let alternatives = match &args[1] {
        Value::List(items) => (**items).clone(),
        _ => return Err(Error::Runtime("choice: second arg must be a list of rules".into())),
    };

    let position = {
        let stream = ps.lock().unwrap();
        stream.position()
    };

    for alt in &alternatives {
        // Save checkpoint
        {
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
        }

        // Try calling the alternative
        let stream_val = Value::ParseStream(ps.clone());
        match self.call_value_and_wait(alt.clone(), vec![stream_val]) {
            Ok(result) => {
                let frame = self.frames.last_mut().unwrap();
                frame.set_current(result);
                return Ok(());
            }
            Err(Error::ParseFailed { .. }) => {
                // Restore and try next
                let mut stream = ps.lock().unwrap();
                stream.restore(&crate::parse_stream::Checkpoint { position });
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::ParseFailed {
        position,
        message: "all alternatives failed".into(),
    })
}
```

**Note**: `call_value_and_wait` is a synchronous call helper that pushes a frame, runs until it returns, and collects the result. If it doesn't exist, it may need to be implemented. The test will guide the exact mechanism — this could also work by catching errors from `call_value` using the existing try/catch infrastructure.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::choice() combinator with backtracking"
jj new
```

---

### Task 9: star and plus Combinators

**Files:**
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn star_zero_matches() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::star(s, digit)
    "#).unwrap();
    // Zero matches returns empty list
    assert_eq!(result, Value::List(Arc::new(vec![])));
}

#[test]
fn star_multiple_matches() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("123abc")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::star(s, digit)
    "#).unwrap();
    assert_eq!(
        result,
        Value::List(Arc::new(vec![
            Value::String("1".into()),
            Value::String("2".into()),
            Value::String("3".into()),
        ]))
    );
}

#[test]
fn plus_requires_one() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let digit = \s { stream::match_class(s, "0-9") }
        try { stream::plus(s, digit) } catch e { "need at least one" }
    "#).unwrap();
    assert_eq!(result, Value::String("need at least one".into()));
}

#[test]
fn plus_multiple_matches() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("123abc")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::plus(s, digit)
    "#).unwrap();
    assert_eq!(
        result,
        Value::List(Arc::new(vec![
            Value::String("1".into()),
            Value::String("2".into()),
            Value::String("3".into()),
        ]))
    );
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::star -- --nocapture 2>&1 | tail -20`
Expected: FAIL

**Step 3: Implement star and plus**

`star` is a loop: checkpoint, try rule, if fail restore + break, if success collect result.

`plus` is: call `star`, check result is non-empty.

In the `"__builtin_stream"` arm:

```rust
"star" => {
    if args.len() != 2 {
        return Err(Error::Runtime("star requires (stream, rule)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("star: first arg must be a stream".into())),
    };
    let rule = args[1].clone();

    let mut results = Vec::new();
    loop {
        let position = {
            let stream = ps.lock().unwrap();
            stream.position()
        };

        let stream_val = Value::ParseStream(ps.clone());
        match self.call_value_and_wait(rule.clone(), vec![stream_val]) {
            Ok(result) => {
                // Guard against zero-length match (infinite loop prevention)
                let new_pos = {
                    let stream = ps.lock().unwrap();
                    stream.position()
                };
                results.push(result);
                if new_pos == position {
                    break; // Zero-length match, stop
                }
            }
            Err(Error::ParseFailed { .. }) => {
                let mut stream = ps.lock().unwrap();
                stream.restore(&crate::parse_stream::Checkpoint { position });
                break;
            }
            Err(e) => return Err(e),
        }
    }

    let frame = self.frames.last_mut().unwrap();
    frame.set_current(Value::List(Arc::new(results)));
}
"plus" => {
    if args.len() != 2 {
        return Err(Error::Runtime("plus requires (stream, rule)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("plus: first arg must be a stream".into())),
    };
    let rule = args[1].clone();

    // Reuse star logic
    // (In practice, call the star implementation and check non-empty)
    let mut results = Vec::new();
    loop {
        let position = {
            let stream = ps.lock().unwrap();
            stream.position()
        };

        let stream_val = Value::ParseStream(ps.clone());
        match self.call_value_and_wait(rule.clone(), vec![stream_val]) {
            Ok(result) => {
                let new_pos = {
                    let stream = ps.lock().unwrap();
                    stream.position()
                };
                results.push(result);
                if new_pos == position {
                    break;
                }
            }
            Err(Error::ParseFailed { .. }) => {
                let mut stream = ps.lock().unwrap();
                stream.restore(&crate::parse_stream::Checkpoint { position });
                break;
            }
            Err(e) => return Err(e),
        }
    }

    if results.is_empty() {
        let pos = {
            let stream = ps.lock().unwrap();
            stream.position()
        };
        return Err(Error::ParseFailed {
            position: pos,
            message: "plus: expected at least one match".into(),
        });
    }

    let frame = self.frames.last_mut().unwrap();
    frame.set_current(Value::List(Arc::new(results)));
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::star() and stream::plus() combinators"
jj new
```

---

### Task 10: seq Combinator and Full Parse Test

**Files:**
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn seq_all_match() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let ra = \s { stream::match_char(s, "a") }
        let rb = \s { stream::match_char(s, "b") }
        let rc = \s { stream::match_char(s, "c") }
        stream::seq(s, [ra, rb, rc])
    "#).unwrap();
    assert_eq!(
        result,
        Value::List(Arc::new(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]))
    );
}

#[test]
fn seq_partial_fails_and_restores() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let ra = \s { stream::match_char(s, "a") }
        let rx = \s { stream::match_char(s, "x") }
        try { stream::seq(s, [ra, rx]) } catch e { s.position() }
    "#).unwrap();
    // seq should restore position after partial match failure
    assert_eq!(result, Value::Int(0));
}

#[test]
fn full_parse_digits_to_number() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("42")
        let digit = \s { stream::match_class(s, "0-9") }
        let digits = stream::plus(s, digit)
        let chars = digits
        let num = chars[0] + chars[1]
        num
    "#).unwrap();
    // "4" + "2" = "42" (string concat)
    assert_eq!(result, Value::String("42".into()));
}

#[test]
fn full_parse_with_semantic_action() {
    let mut vm = Vm::new();
    // Parse "123" as digit+, convert each char to int, fold into number
    let result = eval(&mut vm, r#"
        let s = stream::new("123")
        let digit = \s {
            let c = stream::match_class(s, "0-9")
            int(c)
        }
        let digits = stream::plus(s, digit)
        digits
    "#).unwrap();
    assert_eq!(
        result,
        Value::List(Arc::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]))
    );
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing::seq -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `stream::seq` not implemented

**Step 3: Implement seq**

`seq` takes a stream and a list of rules. It runs each in order, collecting results. If any fails, the whole seq fails and position restores to where seq started.

In the `"__builtin_stream"` arm:

```rust
"seq" => {
    if args.len() != 2 {
        return Err(Error::Runtime("seq requires (stream, rules)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("seq: first arg must be a stream".into())),
    };
    let rules = match &args[1] {
        Value::List(items) => (**items).clone(),
        _ => return Err(Error::Runtime("seq: second arg must be a list of rules".into())),
    };

    let start_pos = {
        let stream = ps.lock().unwrap();
        stream.position()
    };

    let mut results = Vec::new();
    for rule in &rules {
        let stream_val = Value::ParseStream(ps.clone());
        match self.call_value_and_wait(rule.clone(), vec![stream_val]) {
            Ok(result) => results.push(result),
            Err(Error::ParseFailed { position, message }) => {
                // Restore to start of seq
                let mut stream = ps.lock().unwrap();
                stream.restore(&crate::parse_stream::Checkpoint { position: start_pos });
                return Err(Error::ParseFailed { position, message });
            }
            Err(e) => return Err(e),
        }
    }

    let frame = self.frames.last_mut().unwrap();
    frame.set_current(Value::List(Arc::new(results)));
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::seq() and full digit parsing tests"
jj new
```

---

### Task 11: not and lookahead Combinators

**Files:**
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/tests/stream_parsing.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn not_succeeds_when_rule_fails() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::not(s, digit)
        s.position()
    "#).unwrap();
    // not doesn't consume input
    assert_eq!(result, Value::Int(0));
}

#[test]
fn not_fails_when_rule_succeeds() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("123")
        let digit = \s { stream::match_class(s, "0-9") }
        try { stream::not(s, digit) } catch e { "matched" }
    "#).unwrap();
    assert_eq!(result, Value::String("matched".into()));
}

#[test]
fn lookahead_succeeds_without_consuming() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let letter = \s { stream::match_class(s, "a-z") }
        stream::lookahead(s, letter)
        s.position()
    "#).unwrap();
    // lookahead doesn't consume input
    assert_eq!(result, Value::Int(0));
}

#[test]
fn optional_returns_null_on_failure() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("abc")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::optional(s, digit)
    "#).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn optional_returns_value_on_success() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let s = stream::new("123")
        let digit = \s { stream::match_class(s, "0-9") }
        stream::optional(s, digit)
    "#).unwrap();
    assert_eq!(result, Value::String("1".into()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: FAIL

**Step 3: Implement not, lookahead, optional**

In the `"__builtin_stream"` arm:

```rust
"not" => {
    if args.len() != 2 {
        return Err(Error::Runtime("not requires (stream, rule)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("not: first arg must be a stream".into())),
    };
    let rule = args[1].clone();

    let position = {
        let stream = ps.lock().unwrap();
        stream.position()
    };

    let stream_val = Value::ParseStream(ps.clone());
    match self.call_value_and_wait(rule, vec![stream_val]) {
        Ok(_) => {
            // Rule succeeded — not fails, restore position
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
            Err(Error::ParseFailed {
                position,
                message: "negative lookahead matched".into(),
            })
        }
        Err(Error::ParseFailed { .. }) => {
            // Rule failed — not succeeds, restore position
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(Value::Null);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
"lookahead" => {
    if args.len() != 2 {
        return Err(Error::Runtime("lookahead requires (stream, rule)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("lookahead: first arg must be a stream".into())),
    };
    let rule = args[1].clone();

    let position = {
        let stream = ps.lock().unwrap();
        stream.position()
    };

    let stream_val = Value::ParseStream(ps.clone());
    match self.call_value_and_wait(rule, vec![stream_val]) {
        Ok(result) => {
            // Succeeded — restore position (lookahead doesn't consume)
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(result);
            Ok(())
        }
        Err(e) => {
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
            Err(e)
        }
    }
}
"optional" => {
    if args.len() != 2 {
        return Err(Error::Runtime("optional requires (stream, rule)".into()));
    }
    let ps = match &args[0] {
        Value::ParseStream(ps) => ps.clone(),
        _ => return Err(Error::Runtime("optional: first arg must be a stream".into())),
    };
    let rule = args[1].clone();

    let position = {
        let stream = ps.lock().unwrap();
        stream.position()
    };

    let stream_val = Value::ParseStream(ps.clone());
    match self.call_value_and_wait(rule, vec![stream_val]) {
        Ok(result) => {
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(result);
            Ok(())
        }
        Err(Error::ParseFailed { .. }) => {
            let mut stream = ps.lock().unwrap();
            stream.restore(&crate::parse_stream::Checkpoint { position });
            let frame = self.frames.last_mut().unwrap();
            frame.set_current(Value::Null);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core stream_parsing -- --nocapture 2>&1 | tail -20`
Expected: All tests PASS

**Step 5: Commit**

```bash
jj describe -m "feat(stream): add stream::not(), stream::lookahead(), stream::optional()"
jj new
```

---

## Verification

After all tasks:

```bash
cargo test -p fmpl-core stream_parsing   # All new stream tests pass
cargo test -p fmpl-core                   # No regressions in existing tests
```

## Implementation Notes

### `call_value_and_wait`

Several combinators need to call a lambda synchronously and get the result back. The existing `call_value` pushes a frame but returns before execution completes. A synchronous wrapper is needed that:

1. Saves the current frame count
2. Calls `call_value` to push the lambda frame
3. Runs the VM loop until the frame count returns to the saved value
4. Returns the result

If this helper doesn't exist, implement it as part of Task 8 (choice), since that's the first combinator that needs it.

### Value::ParseStream Display

Add a `Display` impl for `ParseStream` and handle it in `value.rs` formatting. Something like `<stream@5 "hello">` showing type, position, and source preview.

### Mutex on ParseStream

The `ParseStream` uses `Mutex` because methods like `advance()` and `restore()` mutate it, but the stream is shared (passed to rule lambdas as an argument). This is safe because FMPL is single-threaded — the mutex prevents concurrent access but never contends in practice. A `RefCell` would also work but `Mutex` is consistent with `AsyncStream`.
