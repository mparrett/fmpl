//! Tests for fold, foldr, and comprehension syntax

use fmpl_core::{Value, eval};

#[test]
fn test_fold_sum() {
    let mut vm = fmpl_core::Vm::new();
    // fold (\acc \elem acc + elem), 0, [1, 2, 3]
    // Using curried short lambdas
    let code = r#"fold (\acc \elem acc + elem), 0, [1, 2, 3]"#;
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "fold should succeed: {:?}", result.err());
    match result {
        Ok(Value::Int(6)) => {}
        other => panic!("Expected Int(6), got {:?}", other),
    }
}

#[test]
fn test_foldr_product() {
    let mut vm = fmpl_core::Vm::new();
    // foldr (\acc \elem elem * acc), 1, [2, 3, 4]
    let code = r#"foldr (\acc \elem elem * acc), 1, [2, 3, 4]"#;
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "foldr should succeed: {:?}", result.err());
    match result {
        Ok(Value::Int(24)) => {}
        other => panic!("Expected Int(24), got {:?}", other),
    }
}

#[test]
fn test_fold_with_constant() {
    let mut vm = fmpl_core::Vm::new();
    // fold (\acc \_ acc + 1), 0, [1, 2, 3] - adds 1 for each element (3 times)
    let code = r#"fold (\acc \_ acc + 1), 0, [1, 2, 3]"#;
    let result = eval(&mut vm, code);
    assert!(result.is_ok(), "fold should succeed: {:?}", result.err());
    match result {
        Ok(Value::Int(3)) => {}
        other => panic!("Expected Int(3), got {:?}", other),
    }
}

#[test]
fn test_map_double() {
    let mut vm = fmpl_core::Vm::new();
    // Comprehension: [x * 2 for x in [1, 2, 3]]
    let code = "[x * 2 for x in [1, 2, 3]]";
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "map comprehension should succeed: {:?}",
        result.err()
    );
    match result {
        Ok(Value::List(lst)) => {
            assert_eq!(lst.len(), 3, "result should have 3 elements");
            assert_eq!(lst[0], Value::Int(2));
            assert_eq!(lst[1], Value::Int(4));
            assert_eq!(lst[2], Value::Int(6));
        }
        Ok(other) => panic!("Expected list, got {:?}", other),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_filter_evens() {
    let mut vm = fmpl_core::Vm::new();
    // Comprehension: [x for x in [1, 2, 3, 4, 5] if x % 2 == 0]
    let code = "[x for x in [1, 2, 3, 4, 5] if x % 2 == 0]";
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "filter comprehension should succeed: {:?}",
        result.err()
    );
    match result {
        Ok(Value::List(lst)) => {
            assert_eq!(lst.len(), 2, "result should have 2 elements");
            assert_eq!(lst[0], Value::Int(2));
            assert_eq!(lst[1], Value::Int(4));
        }
        Ok(other) => panic!("Expected list, got {:?}", other),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_map_with_keyword_function_first() {
    let mut vm = fmpl_core::Vm::new();
    // Keyword syntax: map \x x * 2, [1, 2, 3]
    let code = "map \\x x * 2, [1, 2, 3]";
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "map keyword should succeed: {:?}",
        result.err()
    );
    match result {
        Ok(Value::List(lst)) => {
            assert_eq!(lst.len(), 3);
            assert_eq!(lst[0], Value::Int(2));
            assert_eq!(lst[1], Value::Int(4));
            assert_eq!(lst[2], Value::Int(6));
        }
        Ok(other) => panic!("Expected list, got {:?}", other),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_filter_with_keyword_function_first() {
    let mut vm = fmpl_core::Vm::new();
    // Keyword syntax: filter \x x % 2 == 0, [1, 2, 3, 4, 5]
    let code = "filter \\x x % 2 == 0, [1, 2, 3, 4, 5]";
    let result = eval(&mut vm, code);
    assert!(
        result.is_ok(),
        "filter keyword should succeed: {:?}",
        result.err()
    );
    match result {
        Ok(Value::List(lst)) => {
            assert_eq!(lst.len(), 2);
            assert_eq!(lst[0], Value::Int(2));
            assert_eq!(lst[1], Value::Int(4));
        }
        Ok(other) => panic!("Expected list, got {:?}", other),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
