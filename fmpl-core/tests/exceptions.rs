//! Exception handling tests

use fmpl_core::{Vm, eval};

fn run(src: &str) -> fmpl_core::value::Value {
    let mut vm = Vm::new();
    eval(&mut vm, src).expect("runtime error")
}

fn run_err(src: &str) -> String {
    let mut vm = Vm::new();
    match eval(&mut vm, src) {
        Ok(v) => panic!("expected error, got {:?}", v),
        Err(e) => e.to_string(),
    }
}

#[test]
fn test_try_catch_cross_frame() {
    // Define a function that throws, then call it inside try/catch
    let result = run(r#"
        let (thrower = \x throw "boom")
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
    let result = run(r#"
        let (inner = \x throw "inner error")
        let (outer = \x inner(x))
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
    let result = run(r#"
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
    let err = run_err(
        r#"
        let (thrower = \x throw "no handler")
        thrower(1)
    "#,
    );
    assert!(err.contains("uncaught exception"));
}

#[test]
fn test_cross_frame_execution_continues() {
    // After catching cross-frame exception, execution should continue in catch block
    let result = run(r#"
        let (thrower = \x throw "error")
        let (result = try {
            thrower(1);
            "not reached"
        } catch e {
            "caught"
        })
        result
    "#);
    assert_eq!(result.to_string(), "\"caught\"");
}

#[test]
fn test_cross_frame_with_computation_after() {
    // After catching, we should be able to do more work
    let result = run(r#"
        let (thrower = \x throw x)
        let (a = try {
            thrower("first")
        } catch e {
            e
        })
        let (b = try {
            thrower("second")
        } catch e {
            e
        })
        [a, b]
    "#);
    assert_eq!(result.to_string(), "[\"first\", \"second\"]");
}
