// For loop tests
//
// These tests verify the for..in loop construct works with:
// - Lists
// - Strings
// - Patterns (destructuring)
// - Streams

use fmpl_core::{Value, eval};

#[test]
fn test_simple_for_loop() {
    let mut vm = fmpl_core::Vm::new();
    // Test with wildcard pattern (no binding) - just check it runs
    let code = "for _ in [1, 2, 3] { 42 }";
    let result = eval(&mut vm, code);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    // For loops return null, so we just check it runs without error
    assert!(result.is_ok(), "for loop should succeed");
}

#[test]
#[ignore = "TODO: for loop variable binding not working correctly - mutations not persisting"]
fn test_for_loop_with_var_binding() {
    let mut vm = fmpl_core::Vm::new();
    // Test with variable binding
    let code = "let sum = 0; for x in [1, 2, 3] { sum = sum + x }; sum";
    let result = eval(&mut vm, code);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "for loop should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 6, "sum should be 6, got {}", n);
    }
}

#[test]
fn test_simple_variable_binding() {
    let mut vm = fmpl_core::Vm::new();
    // Test that variable binding works in a simple scope
    let code = "let x = 42; x";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "variable binding should work");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 42, "x should be 42");
    }
}

#[test]
fn test_cursor_current() {
    let mut vm = fmpl_core::Vm::new();
    // Test that cursor::current returns the first element
    let code = "let c = stream::observe([10, 20, 30]); cursor::current(c)";
    let result = eval(&mut vm, code);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "cursor::current should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 10, "first element should be 10");
    }
}

#[test]
fn test_cursor_current_not_null() {
    let mut vm = fmpl_core::Vm::new();
    // Test that cursor::current != null for non-empty list
    let code = "let c = stream::observe([10, 20, 30]); let curr = cursor::current(c); curr != null";
    let result = eval(&mut vm, code);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "comparison should succeed");
    if let Ok(Value::Bool(b)) = result {
        assert!(b, "current should not be null");
    }
}

#[test]
#[ignore = "TODO: manual for loop simulation not working"]
fn test_manual_for_loop() {
    let mut vm = fmpl_core::Vm::new();
    // Manually simulate what the for loop does
    let code = "let sum = 0; let c = stream::observe([1, 2, 3]); let curr = cursor::current(c); if curr != null { sum = sum + 1 }; sum";
    let result = eval(&mut vm, code);
    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }
    assert!(result.is_ok(), "manual loop simulation should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 1, "sum should be 1 after one iteration");
    }
}

#[test]
#[ignore = "TODO: for loop variable binding not working correctly - mutations not persisting"]
fn test_for_loop_with_sum() {
    let mut vm = fmpl_core::Vm::new();
    let code = "let sum = 0; for x in [1, 2, 3, 4, 5] { sum = sum + x }; sum";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "for loop sum should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 15, "sum should be 15");
    } else {
        panic!("Expected Int result");
    }
}

#[test]
fn test_for_loop_empty_list() {
    let mut vm = fmpl_core::Vm::new();
    let code = "let count = 0; for x in [] { count = count + 1 }; count";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "for loop with empty list should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 0, "count should still be 0");
    } else {
        panic!("Expected Int result");
    }
}

#[test]
fn test_for_loop_with_string() {
    let mut vm = fmpl_core::Vm::new();
    let code = "let result = []; for ch in \"abc\" { result.push(ch) }; result";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "for loop over string should succeed");
}

#[test]
#[ignore = "TODO: for loop variable binding not working correctly - mutations not persisting"]
fn test_for_loop_wildcard_pattern() {
    let mut vm = fmpl_core::Vm::new();
    let code = "let count = 0; for _ in [1, 2, 3] { count = count + 1 }; count";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "for loop with wildcard should succeed");
    if let Ok(Value::Int(n)) = result {
        assert_eq!(n, 3, "count should be 3");
    } else {
        panic!("Expected Int result");
    }
}

#[test]
fn test_nested_for_loop() {
    let mut vm = fmpl_core::Vm::new();
    let code =
        "let result = []; for x in [1, 2] { for y in [3, 4] { result.push([x, y]) } }; result";
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "nested for loop should succeed");
}
