//! Integration tests for tuple space operations.

use fmpl_core::{Value, Vm, eval};

#[test]
fn test_tuplespace_builtin_exists() {
    let mut vm = Vm::new();

    // Verify tuplespace symbol is recognized
    let result = eval(&mut vm, "tuplespace").unwrap();

    // Should be a symbol (the builtin)
    match result {
        Value::Symbol(s) => {
            assert_eq!(s.as_str(), "__builtin_tuplespace");
        }
        other => panic!("expected Symbol, got {:?}", other),
    }
}

#[test]
fn test_tuplespace_new_creates_tuple_space() {
    let mut vm = Vm::new();

    // Call tuplespace.new() to create a tuple space
    let result = eval(&mut vm, "tuplespace.new()").unwrap();

    // Should be a TupleSpace value
    match result {
        Value::TupleSpace(_) => {
            // Successfully created a tuple space
        }
        other => panic!("expected TupleSpace, got {:?}", other),
    }
}

#[test]
fn test_tuplespace_bind_and_use() {
    let mut vm = Vm::new();

    // Create a tuple space and bind it to a variable
    let _ = eval(&mut vm, "let space = tuplespace.new()").unwrap();

    // Verify the variable holds a TupleSpace
    let result = eval(&mut vm, "space").unwrap();

    match result {
        Value::TupleSpace(_) => {
            // Success
        }
        other => panic!("expected TupleSpace, got {:?}", other),
    }
}

#[test]
fn test_tuplespace_display() {
    let mut vm = Vm::new();

    // Create a tuple space
    let result = eval(&mut vm, "tuplespace.new()").unwrap();

    // Verify it displays correctly
    assert_eq!(format!("{}", result), "<tuplespace>");
}

#[test]
fn test_tuplespace_type_name() {
    let mut vm = Vm::new();

    // Create a tuple space
    let result = eval(&mut vm, "tuplespace.new()").unwrap();

    // Verify type name
    assert_eq!(result.type_name(), "tuplespace");
}

#[test]
fn test_tuplespace_is_truthy() {
    let mut vm = Vm::new();

    // Create a tuple space
    let result = eval(&mut vm, "tuplespace.new()").unwrap();

    // TupleSpace should be truthy
    assert!(result.is_truthy());
    assert!(!result.is_falsy());
}

#[test]
fn test_multiple_tuple_spaces() {
    let mut vm = Vm::new();

    // Create multiple tuple spaces
    let _ = eval(&mut vm, "let space1 = tuplespace.new()").unwrap();
    let _ = eval(&mut vm, "let space2 = tuplespace.new()").unwrap();

    // Verify both are TupleSpaces
    let result1 = eval(&mut vm, "space1").unwrap();
    let result2 = eval(&mut vm, "space2").unwrap();

    match (&result1, &result2) {
        (Value::TupleSpace(_), Value::TupleSpace(_)) => {
            // Both are tuple spaces (different instances)
        }
        other => panic!("expected both to be TupleSpace, got {:?}", other),
    }
}
