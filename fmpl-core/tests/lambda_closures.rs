//! Tests for lambda parameter binding and closures

use fmpl_core::{Value, Vm, eval};

#[test]
fn test_simple_lambda_parameter_binding() {
    let mut vm = Vm::new();
    // Simple lambda with single parameter
    let result = eval(&mut vm, "let (f = lambda (x) x + 1) f(5)").unwrap();
    assert_eq!(result, Value::Int(6));
}

#[test]
fn test_multi_parameter_lambda() {
    let mut vm = Vm::new();
    // Lambda with multiple parameters
    let result = eval(&mut vm, "let (add = lambda (a, b) a + b) add(3, 4)").unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_lambda_with_inner_bindings() {
    let mut vm = Vm::new();
    // Lambda with inner let binding
    let result = eval(
        &mut vm,
        "let (result = lambda (x) let (y = x + 1) y) result(5)",
    )
    .unwrap();
    assert_eq!(result, Value::Int(6));
}

// Tests for closures - these should work after implementation

#[test]
fn test_basic_closure() {
    let mut vm = Vm::new();
    // Basic closure: lambda captures variable from outer scope
    let result = eval(&mut vm, "let (x = 10) let (f = lambda () x) f()").unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_lambda_with_parameter_and_capture() {
    let mut vm = Vm::new();
    // Lambda with both parameter and captured variable
    let result = eval(&mut vm, "let (y = 20) let (f = lambda (x) x + y) f(5)").unwrap();
    assert_eq!(result, Value::Int(25));
}

#[test]
fn test_nested_closures() {
    let mut vm = Vm::new();
    // Nested closures
    let result = eval(
        &mut vm,
        "let (z = 30)
         let (f = lambda (x)
             let (g = lambda (y) x + y + z)
             g(10))
         f(5)",
    )
    .unwrap();
    assert_eq!(result, Value::Int(45)); // 5 + 10 + 30 = 45
}

#[test]
fn test_capture_with_shadowing() {
    let mut vm = Vm::new();
    // Capture with shadowing: inner lambda should capture outer variable
    let result = eval(
        &mut vm,
        "let (x = 100)
         let (f = lambda (y)
             let (x = 200)  // Shadow outer x
             lambda () x + y  // Should capture shadowed x and parameter y
         )
         let (g = f(5)) g()",
    )
    .unwrap();
    assert_eq!(result, Value::Int(205)); // 200 (shadowed) + 5 = 205
}

#[test]
fn test_multiple_captures() {
    let mut vm = Vm::new();
    // Lambda captures multiple variables from different scopes
    let result = eval(
        &mut vm,
        "let (a = 1)
         let (b = 2)
         let (c = 3)
         let (f = lambda (x) a + b + c + x)
         f(4)",
    )
    .unwrap();
    assert_eq!(result, Value::Int(10)); // 1 + 2 + 3 + 4 = 10
}

#[test]
fn test_closure_returned_from_function() {
    let mut vm = Vm::new();
    // Return a closure from a function
    let result = eval(
        &mut vm,
        "let (make_adder = lambda (n)
             lambda (x) x + n)
         let (add5 = make_adder(5))
         add5(10)",
    )
    .unwrap();
    assert_eq!(result, Value::Int(15));
}

// NOTE: These tests are skipped due to current language limitations.
// See specs/parser-limitations.md for planned features like recursive let
// bindings and mutable closure captures (bcom pattern).
#[test]
#[ignore]
fn test_mutated_closure() {
    let mut vm = Vm::new();
    // Closure that captures a mutable variable
    let result = eval(
        &mut vm,
        "let (counter = 0)
         let (inc = lambda ()
             let (old = counter)
             counter = counter + 1
             old)
         inc(); inc(); inc()",
    )
    .unwrap();
    assert_eq!(result, Value::Int(2)); // Returns 0, then 1, then 2
}

#[test]
#[ignore]
fn test_self_referential_closure() {
    let mut vm = Vm::new();
    // Complex closure with recursion
    let result = eval(
        &mut vm,
        "let (make_factorial = lambda ()
             let (fact = lambda (n)
                 if (n <= 1) then 1 else n * fact(n - 1))
             fact)
         let (f = make_factorial())
         f(5)",
    )
    .unwrap();
    assert_eq!(result, Value::Int(120)); // 5! = 120
}
