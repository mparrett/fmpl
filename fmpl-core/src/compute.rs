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

    #[test]
    fn drain_range_collects_yielded_elements_then_done() {
        let mut ctx = StepCtx::new();
        let mut c = Range { next: 0, end: 3 };
        let out = drain(&mut c, &mut ctx).unwrap();
        assert_eq!(out, vec![Value::Int(0), Value::Int(1), Value::Int(2)]);
    }

    #[test]
    fn each_step_advances_exactly_one_element_in_order() {
        let mut ctx = StepCtx::new();
        let mut c = Range { next: 10, end: 13 };
        // One step -> one element, in order.
        assert!(matches!(
            c.step(&mut ctx).unwrap(),
            Step::Yield(Value::Int(10))
        ));
        assert!(matches!(
            c.step(&mut ctx).unwrap(),
            Step::Yield(Value::Int(11))
        ));
        assert!(matches!(
            c.step(&mut ctx).unwrap(),
            Step::Yield(Value::Int(12))
        ));
        // Then Done, and Done is stable on further steps.
        assert!(matches!(c.step(&mut ctx).unwrap(), Step::Done(Value::Null)));
        assert!(matches!(c.step(&mut ctx).unwrap(), Step::Done(Value::Null)));
    }

    #[test]
    fn large_range_drains_without_unbounded_recursion() {
        // A million elements must not blow the stack: the driver loops, it does
        // not recurse. This is the bounded-stack (no_std/wasm) guarantee.
        let mut ctx = StepCtx::new();
        let mut c = Range {
            next: 0,
            end: 1_000_000,
        };
        let out = drain(&mut c, &mut ctx).unwrap();
        assert_eq!(out.len(), 1_000_000);
        assert_eq!(out.first(), Some(&Value::Int(0)));
        assert_eq!(out.last(), Some(&Value::Int(999_999)));
    }
}
