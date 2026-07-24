# A₀ Computation Substrate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce the trampolined `Computation` substrate (`Step`, `StepCtx`, the `Computation` trait, and driver functions) as a new, self-contained `fmpl-core` module — with zero changes to `Value` or existing behavior.

**Architecture:** A `Computation` advances one bounded step at a time via `step(&mut self, ctx) -> Step`, where `Step` is `Done(Value) | Yield(Value) | Pending`. The computation's own `&mut self` *is* the continuation (resume = call `step` again). Driver functions (`force`, `drain`) pump `step` to completion. This is the foundation of the [computable-value model (A₀)](../design/computable-value-model.md); this plan builds only the substrate — the `Value::Deferred` variant, `snapshot()`/persistence, and the forcing-point audit are **later plans**.

**Tech Stack:** Rust (edition 2024), `fmpl-core` crate. No new dependencies.

## Global Constraints

- **Edition 2024**, workspace dependency inheritance — no new crates for this plan.
- **`Computation: Send + Sync`** (mirrors the `Store` trait bound so a computation can live behind `Arc<Mutex<…>>` in a future `Value::Deferred`).
- **Errors** use the crate's existing alias: `crate::Result<T>` = `Result<T, crate::error::Error>` (defined in `fmpl-core/src/error.rs`).
- **Tests** are inline `#[cfg(test)] mod tests` in the module file (project convention; see `fmpl-core/src/compiler.rs`).
- **No process tags in code comments** (no `A₀`, `STORY-`, `ITER-` in `.rs` files — those go in commit messages).
- **fmt + clippy clean** — a pre-commit hook runs `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` on every commit and will block otherwise.
- **Commit messages** carry no AI attribution (hook-enforced).
- **This module is pure-additive** — it must not modify `value.rs`, `vm.rs`, or any existing file except adding one `pub mod compute;` line to `lib.rs`.

---

### Task 1: Module skeleton — `Step`, `StepCtx`, `Computation` trait, and a `Ready` computation driven by `force`

**Files:**
- Create: `fmpl-core/src/compute.rs`
- Modify: `fmpl-core/src/lib.rs` (add `pub mod compute;` in the module list, alphabetically near `pub mod compiler;`)
- Test: inline `#[cfg(test)] mod tests` in `fmpl-core/src/compute.rs`

**Interfaces:**
- Produces:
  - `pub enum Step { Done(Value), Yield(Value), Pending }`
  - `pub struct StepCtx { /* empty for now; grows in A₂ */ }` with `pub fn new() -> Self`
  - `pub trait Computation: Send + Sync { fn step(&mut self, ctx: &mut StepCtx) -> crate::Result<Step>; }`
  - `pub struct Ready(pub Option<Value>);` implementing `Computation`
  - `pub fn force(c: &mut dyn Computation, ctx: &mut StepCtx) -> crate::Result<Value>`

- [ ] **Step 1: Write the failing test**

Add to `fmpl-core/src/compute.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn force_ready_returns_its_value() {
        let mut ctx = StepCtx::new();
        let mut c = Ready(Some(Value::Int(42)));
        assert_eq!(force(&mut c, &mut ctx).unwrap(), Value::Int(42));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core --lib compute:: 2>&1 | tail -20`
Expected: FAIL — `compute.rs` / module not found (module not yet declared in `lib.rs`, symbols undefined).

- [ ] **Step 3: Write minimal implementation**

Create `fmpl-core/src/compute.rs` (above the test module):

```rust
//! Trampolined computation substrate: a value that is a not-yet-forced
//! computation, advanced one bounded step at a time.

use crate::value::Value;

/// The outcome of advancing a computation by one bounded step.
pub enum Step {
    /// Finished with a final value.
    Done(Value),
    /// Emitted one sequence element; call `step` again for more.
    Yield(Value),
    /// Suspended (awaiting IO or a sub-value); call `step` again when ready.
    Pending,
}

/// Context threaded through `step`. Empty today; grows to carry evaluator
/// re-entrancy, the persistence Store, and id generation.
#[derive(Default)]
pub struct StepCtx {}

impl StepCtx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A value that is a computation. Advanced cooperatively, one bounded step
/// per `step` call; `&mut self` is the continuation.
pub trait Computation: Send + Sync {
    fn step(&mut self, ctx: &mut StepCtx) -> crate::Result<Step>;
}

/// A computation that is already a value. Yields it once as `Done`.
pub struct Ready(pub Option<Value>);

impl Computation for Ready {
    fn step(&mut self, _ctx: &mut StepCtx) -> crate::Result<Step> {
        match self.0.take() {
            Some(v) => Ok(Step::Done(v)),
            None => Ok(Step::Done(Value::Null)),
        }
    }
}

/// Drive a computation to completion, returning its final `Done` value.
/// `Yield`ed elements are discarded (single-value forcing); `Pending`
/// re-steps (Phase-1 computations are synchronous and never stay Pending).
pub fn force(c: &mut dyn Computation, ctx: &mut StepCtx) -> crate::Result<Value> {
    loop {
        match c.step(ctx)? {
            Step::Done(v) => return Ok(v),
            Step::Yield(_) | Step::Pending => continue,
        }
    }
}
```

Add to `fmpl-core/src/lib.rs`, in the module list (alphabetical, right after `pub mod compiler;`):

```rust
pub mod compute;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core --lib compute:: 2>&1 | tail -20`
Expected: PASS — `test compute::tests::force_ready_returns_its_value ... ok`.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(compute): trampolined Computation substrate — Step/StepCtx/Ready/force"
```
(The pre-commit hook runs fmt + clippy; both must pass. `jj` auto-snapshots the working copy into the commit.)

---

### Task 2: A `Range` computation that `Yield`s a sequence, and a `drain` driver

**Files:**
- Modify: `fmpl-core/src/compute.rs`
- Test: inline `#[cfg(test)] mod tests` in `fmpl-core/src/compute.rs`

**Interfaces:**
- Consumes: `Step`, `StepCtx`, `Computation` (Task 1)
- Produces:
  - `pub struct Range { pub next: i64, pub end: i64 }` implementing `Computation`
  - `pub fn drain(c: &mut dyn Computation, ctx: &mut StepCtx) -> crate::Result<Vec<Value>>`

- [ ] **Step 1: Write the failing test**

Add a test to the `tests` module in `fmpl-core/src/compute.rs`:

```rust
#[test]
fn drain_range_collects_yielded_elements_then_done() {
    let mut ctx = StepCtx::new();
    let mut c = Range { next: 0, end: 3 };
    let out = drain(&mut c, &mut ctx).unwrap();
    assert_eq!(out, vec![Value::Int(0), Value::Int(1), Value::Int(2)]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core --lib compute::tests::drain_range 2>&1 | tail -20`
Expected: FAIL — `Range` and `drain` are undefined.

- [ ] **Step 3: Write minimal implementation**

Add to `fmpl-core/src/compute.rs` (below `force`):

```rust
/// A computation that yields `next..end` as `Value::Int`s, then `Done(Null)`.
/// `&mut self` (the `next` cursor) is the continuation.
pub struct Range {
    pub next: i64,
    pub end: i64,
}

impl Computation for Range {
    fn step(&mut self, _ctx: &mut StepCtx) -> crate::Result<Step> {
        if self.next < self.end {
            let v = Value::Int(self.next);
            self.next += 1;
            Ok(Step::Yield(v))
        } else {
            Ok(Step::Done(Value::Null))
        }
    }
}

/// Drive a computation to completion, collecting every `Yield`ed element.
pub fn drain(c: &mut dyn Computation, ctx: &mut StepCtx) -> crate::Result<Vec<Value>> {
    let mut out = Vec::new();
    loop {
        match c.step(ctx)? {
            Step::Yield(v) => out.push(v),
            Step::Pending => continue,
            Step::Done(_) => return Ok(out),
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core --lib compute::tests::drain_range 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(compute): Range computation (Yield) + drain driver"
```

---

### Task 3: Prove bounded, ordered stepping (the trampoline guarantee)

**Files:**
- Test: inline `#[cfg(test)] mod tests` in `fmpl-core/src/compute.rs`

**Interfaces:**
- Consumes: `Range`, `Step`, `StepCtx`, `Computation` (Tasks 1–2)
- Produces: no new public API — this task locks the stepping contract with tests.

- [ ] **Step 1: Write the failing test**

Add two tests to the `tests` module:

```rust
#[test]
fn each_step_advances_exactly_one_element_in_order() {
    let mut ctx = StepCtx::new();
    let mut c = Range { next: 10, end: 13 };
    // One step -> one element, in order.
    assert!(matches!(c.step(&mut ctx).unwrap(), Step::Yield(Value::Int(10))));
    assert!(matches!(c.step(&mut ctx).unwrap(), Step::Yield(Value::Int(11))));
    assert!(matches!(c.step(&mut ctx).unwrap(), Step::Yield(Value::Int(12))));
    // Then Done, and Done is stable on further steps.
    assert!(matches!(c.step(&mut ctx).unwrap(), Step::Done(Value::Null)));
    assert!(matches!(c.step(&mut ctx).unwrap(), Step::Done(Value::Null)));
}

#[test]
fn large_range_drains_without_unbounded_recursion() {
    // A million elements must not blow the stack: the driver loops, it does
    // not recurse. This is the bounded-stack (no_std/wasm) guarantee.
    let mut ctx = StepCtx::new();
    let mut c = Range { next: 0, end: 1_000_000 };
    let out = drain(&mut c, &mut ctx).unwrap();
    assert_eq!(out.len(), 1_000_000);
    assert_eq!(out.first(), Some(&Value::Int(0)));
    assert_eq!(out.last(), Some(&Value::Int(999_999)));
}
```

- [ ] **Step 2: Run tests to verify they pass immediately**

These assert the contract already built in Tasks 1–2, so they should pass on first run (this task is a contract lock, not new behavior).

Run: `cargo test -p fmpl-core --lib compute:: 2>&1 | tail -20`
Expected: PASS — all `compute::tests::*` green (4 tests total).

If either fails, the stepping contract is wrong — fix `Range::step`/`drain` until both pass before committing.

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(compute): lock bounded, ordered stepping contract (per-step + million-element drain)"
```

---

## Self-Review

**Spec coverage (against `docs/design/computable-value-model.md`, substrate only):**
- `Step` enum `Done | Yield | Pending` — Task 1 (Done, Pending) + Task 2 (Yield). ✓
- `Computation` trait with `step(&mut self, ctx)` — Task 1. ✓
- `StepCtx` placeholder (grows in A₂) — Task 1. ✓
- Driver / cooperative forcing (bounded stack) — `force` (Task 1), `drain` (Task 2), bounded-stack proof (Task 3). ✓
- **Deliberately out of scope (later plans):** `snapshot()`/`ComputationSnapshot` (with persistence), `Value::Deferred` variant + forcing-point audit, async `Pending` that actually awaits IO, the phased subsume of `Partial`/`Stream`/etc. Noted in the plan header.

**Placeholder scan:** no TBD/TODO/"handle edge cases" — every step has concrete code and exact commands. ✓

**Type consistency:** `Step`, `StepCtx`, `Computation`, `force`, `drain`, `Ready`, `Range` are used with identical signatures across Tasks 1–3; `crate::Result` and `Value` (`crate::value::Value`) match the codebase. ✓

## Execution Handoff

**Plan complete and saved to `docs/plans/2026-07-24-a0-computation-substrate-implementation.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
