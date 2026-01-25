//! Tests for stream.create builtin

use fmpl_core::value::Value;
use fmpl_core::{Vm, eval};

#[test]
fn test_stream_create_requires_runtime() {
    let mut vm = Vm::new();
    // stream.create without runtime should fail
    let code = r#"stream.create(lambda () 42)"#;
    let result = eval(&mut vm, code);
    assert!(
        result.is_err(),
        "stream.create should fail without runtime handle: {:?}",
        result
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("runtime handle"),
        "error should mention runtime handle, got: {}",
        err
    );
}

#[tokio::test]
async fn test_stream_create_with_runtime() {
    let mut vm = Vm::with_runtime(tokio::runtime::Handle::current());
    // stream.create with runtime should return AsyncStream
    let code = r#"stream.create(lambda () 42)"#;
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "stream.create should succeed with runtime handle: {:?}",
        result.err()
    );
    match result.unwrap() {
        Value::AsyncStream(_) => {
            // Success - got an AsyncStream
        }
        other => panic!("Expected AsyncStream, got {:?}", other),
    }
}

#[tokio::test]
async fn test_stream_create_validates_lambda() {
    let mut vm = Vm::with_runtime(tokio::runtime::Handle::current());
    // stream.create with non-lambda should fail
    let code = r#"stream.create(42)"#;
    let result = eval(&mut vm, code);
    assert!(
        result.is_err(),
        "stream.create should fail with non-lambda argument"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("lambda"),
        "error should mention lambda, got: {}",
        err
    );
}

#[tokio::test]
async fn test_stream_is_accessible() {
    let mut vm = Vm::new();
    // stream should be accessible as a builtin
    let code = r#"stream"#;
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "stream should be accessible");
    match result.unwrap() {
        Value::Symbol(s) => {
            assert_eq!(
                s.as_str(),
                "__builtin_stream",
                "stream should resolve to __builtin_stream"
            );
        }
        other => panic!("Expected Symbol, got {:?}", other),
    }
}

#[tokio::test]
async fn test_stream_observe_accessible() {
    let mut vm = Vm::new();
    // stream.observe should be accessible
    let code = r#"stream.observe([1, 2, 3])"#;
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "stream.observe should work: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_cursor_is_accessible() {
    let mut vm = Vm::new();
    // cursor should be accessible as a builtin
    let code = r#"cursor"#;
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "cursor should be accessible");
    match result.unwrap() {
        Value::Symbol(s) => {
            assert_eq!(
                s.as_str(),
                "__builtin_cursor",
                "cursor should resolve to __builtin_cursor"
            );
        }
        other => panic!("Expected Symbol, got {:?}", other),
    }
}
