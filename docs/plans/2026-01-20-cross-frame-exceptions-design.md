# Cross-Frame Exception Handling Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix exception handling to work correctly when exceptions are thrown from nested function calls.

**Architecture:** Store frame depth with each exception handler. On throw, unwind frames to the handler's frame, then jump to catch_ip.

**Tech Stack:** Rust, fmpl-core VM

---

## Problem

Current handler tuple `(catch_ip, stack_depth, scope_depth)` doesn't track which frame installed the handler. When an exception is thrown from a nested call:

```
try {
    foo()  // foo throws
} catch e {
    // never reached
}
```

The VM tries to jump to `catch_ip` in the *current* frame (foo's frame), not the frame that installed the handler.

## Solution

Change handler tuple to `(catch_ip, stack_depth, frame_depth)`:
- `catch_ip` - instruction pointer in the handler's frame
- `stack_depth` - value stack depth to restore
- `frame_depth` - `self.frames.len()` when handler was installed

On throw:
1. Pop handlers until we find one at or above current frame depth
2. Unwind frames to `frame_depth`
3. Restore stack to `stack_depth`
4. Push error value
5. Set IP to `catch_ip`

---

### Task 1: Add cross-frame exception test

**Files:**
- Create: `fmpl-core/tests/exceptions.rs`

**Step 1: Write the failing test**

```rust
//! Exception handling tests

use fmpl_core::compiler::Compiler;
use fmpl_core::parser::Parser;
use fmpl_core::vm::Vm;

fn eval(src: &str) -> fmpl_core::value::Value {
    let mut parser = Parser::new(src);
    let expr = parser.parse().expect("parse error");
    let mut compiler = Compiler::new();
    let code = compiler.compile(&expr).expect("compile error");
    let mut vm = Vm::new();
    vm.run(&code).expect("runtime error")
}

fn eval_err(src: &str) -> String {
    let mut parser = Parser::new(src);
    let expr = parser.parse().expect("parse error");
    let mut compiler = Compiler::new();
    let code = compiler.compile(&expr).expect("compile error");
    let mut vm = Vm::new();
    match vm.run(&code) {
        Ok(v) => panic!("expected error, got {:?}", v),
        Err(e) => e.to_string(),
    }
}

#[test]
fn test_try_catch_cross_frame() {
    // Define a function that throws, then call it inside try/catch
    let result = eval(r#"
        let thrower = \x -> throw "boom";
        try {
            thrower(1)
        } catch e {
            e
        }
    "#);
    assert_eq!(result.to_string(), "\"boom\"");
}

#[test]
fn test_try_catch_nested_calls() {
    // Multiple levels of nesting
    let result = eval(r#"
        let inner = \x -> throw "inner error";
        let outer = \x -> inner(x);
        try {
            outer(1)
        } catch e {
            e
        }
    "#);
    assert_eq!(result.to_string(), "\"inner error\"");
}

#[test]
fn test_try_catch_same_frame() {
    // Sanity check: same-frame still works
    let result = eval(r#"
        try {
            throw "direct"
        } catch e {
            e
        }
    "#);
    assert_eq!(result.to_string(), "\"direct\"");
}

#[test]
fn test_uncaught_exception_propagates() {
    // No handler should still produce error
    let err = eval_err(r#"
        let thrower = \x -> throw "no handler";
        thrower(1)
    "#);
    assert!(err.contains("uncaught exception"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core --test exceptions`
Expected: FAIL - `test_try_catch_cross_frame` fails with uncaught exception

**Step 3: Commit failing test**

```bash
git add fmpl-core/tests/exceptions.rs
git commit -m "test: add cross-frame exception tests (currently failing)"
```

---

### Task 2: Update exception handler tuple

**Files:**
- Modify: `fmpl-core/src/vm.rs:54-55` (handler type)
- Modify: `fmpl-core/src/vm.rs:69` (initialization)

**Step 1: Change handler type comment and tuple**

Change from:
```rust
/// Exception handler stack: (catch_ip, stack_depth, scope_depth)
exception_handlers: Vec<(usize, usize, usize)>,
```

To:
```rust
/// Exception handler stack: (catch_ip, stack_depth, frame_depth)
exception_handlers: Vec<(usize, usize, usize)>,
```

Note: The tuple type stays the same, but the *meaning* of the third element changes from scope_depth to frame_depth.

**Step 2: Verify compiles**

Run: `cargo check -p fmpl-core`

**Step 3: Commit**

```bash
git add fmpl-core/src/vm.rs
git commit -m "docs: clarify exception handler tuple semantics"
```

---

### Task 3: Fix PushHandler to store frame_depth

**Files:**
- Modify: `fmpl-core/src/vm.rs:698-701` (PushHandler instruction)

**Step 1: Update PushHandler**

Change from:
```rust
Instruction::PushHandler(catch_ip) => {
    let stack_depth = self.stack.len();
    let scope_depth = self.frames.last().map(|f| f.locals.len()).unwrap_or(0);
    self.exception_handlers.push((catch_ip, stack_depth, scope_depth));
}
```

To:
```rust
Instruction::PushHandler(catch_ip) => {
    let stack_depth = self.stack.len();
    let frame_depth = self.frames.len();
    self.exception_handlers.push((catch_ip, stack_depth, frame_depth));
}
```

**Step 2: Verify compiles**

Run: `cargo check -p fmpl-core`

**Step 3: Commit**

```bash
git add fmpl-core/src/vm.rs
git commit -m "fix: PushHandler stores frame_depth instead of scope_depth"
```

---

### Task 4: Fix throw_exception to unwind frames

**Files:**
- Modify: `fmpl-core/src/vm.rs:764-780` (throw_exception method)

**Step 1: Rewrite throw_exception**

Change from:
```rust
fn throw_exception(&mut self, error: Value) -> Result<()> {
    if let Some((catch_ip, stack_depth, _scope_depth)) = self.exception_handlers.pop() {
        // Unwind stack to handler depth
        while self.stack.len() > stack_depth {
            self.stack.pop();
        }
        // Push error value for binding
        self.stack.push(error);
        // Jump to catch block
        if let Some(frame) = self.frames.last_mut() {
            frame.ip = catch_ip;
        }
        Ok(())
    } else {
        // No handler - propagate as Rust error
        Err(Error::Runtime(format!("uncaught exception: {}", error)))
    }
}
```

To:
```rust
fn throw_exception(&mut self, error: Value) -> Result<()> {
    if let Some((catch_ip, stack_depth, frame_depth)) = self.exception_handlers.pop() {
        // Unwind frames to handler's frame
        while self.frames.len() > frame_depth {
            self.frames.pop();
        }
        // Unwind value stack to handler depth
        while self.stack.len() > stack_depth {
            self.stack.pop();
        }
        // Push error value for catch binding
        self.stack.push(error);
        // Jump to catch block in handler's frame
        if let Some(frame) = self.frames.last_mut() {
            frame.ip = catch_ip;
        }
        Ok(())
    } else {
        // No handler - propagate as Rust error
        Err(Error::Runtime(format!("uncaught exception: {}", error)))
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p fmpl-core --test exceptions`
Expected: All 4 tests pass

**Step 3: Run full test suite**

Run: `cargo test -p fmpl-core`
Expected: All tests pass

**Step 4: Commit**

```bash
git add fmpl-core/src/vm.rs
git commit -m "fix: throw_exception unwinds frames for cross-frame exceptions"
```

---

### Task 5: Final verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Amend failing test commit message**

The test from Task 1 now passes, so update its commit message:

```bash
git rebase -i HEAD~4
# Change first commit from "test: ... (currently failing)" to "test: add cross-frame exception tests"
```

Or leave as-is if preferred (documents TDD process).

---

## Summary

This fix changes the exception handler tuple's third element from `scope_depth` (which tracked local variables in the current frame - not useful) to `frame_depth` (which tracks how deep in the call stack we were).

The key change is in `throw_exception`: before restoring the stack, we first unwind frames until we're back at the frame that installed the handler. This ensures `catch_ip` is valid for the current frame's code.
