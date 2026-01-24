//! Tests for $ (sync) and <- (async) operators with method calls.

use fmpl_core::{Value, Vm, eval};

#[test]
fn test_dollar_operator_with_method_call() {
    let mut vm = Vm::new();

    // Define an object with a method
    let _ = eval(
        &mut vm,
        r#"
object counter {
  get_value(): 42
}
"#,
    )
    .expect("define object");

    // Test that $ operator works with method calls
    // $ is the sync operator - should behave like regular method call in single-vat context
    let result = eval(&mut vm, "$ counter.get_value()").expect("sync method call");

    // Should return the method's return value
    assert!(matches!(result, Value::Int(42)));
}

#[test]
fn test_dollar_operator_equals_regular_method_call() {
    let mut vm = Vm::new();

    let _ = eval(
        &mut vm,
        r#"
object test_obj {
  value(): 123
}
"#,
    )
    .expect("define object");

    // In single-vat context, $ obj.method() should be equivalent to obj.method()
    let sync_result = eval(&mut vm, "$ test_obj.value()").expect("sync call");
    let regular_result = eval(&mut vm, "test_obj.value()").expect("regular call");

    assert_eq!(sync_result, regular_result);
}

#[tokio::test]
async fn test_async_operator_with_method_call() {
    let mut vm = Vm::with_runtime(tokio::runtime::Handle::current());

    // Define an object with a method
    let _ = eval(
        &mut vm,
        r#"
object async_obj {
  get_data(): "async result"
}
"#,
    )
    .expect("define object");

    // Test that <- operator works with method calls and returns a stream
    let result = eval(&mut vm, "<- async_obj.get_data()").expect("async method call");

    // Should return an AsyncStream containing the method's result
    match result {
        Value::AsyncStream(stream) => {
            // Wait a bit for the async operation to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            let mut handle = stream.lock().unwrap();
            if let Some(event) = handle.recv_blocking() {
                match event {
                    fmpl_core::StreamEvent::Ok(Value::String(s)) => {
                        assert_eq!(s.as_str(), "async result");
                    }
                    other => panic!("expected Ok with string, got {:?}", other),
                }
            } else {
                panic!("expected stream to have data");
            }
        }
        other => panic!("expected AsyncStream, got {:?}", other),
    }
}
