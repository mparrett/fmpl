//! VM-level integration tests for tuplespace operations.

use fmpl_core::eval;

#[test]
fn test_tuplespace_out_and_in() {
    let mut vm = fmpl_core::Vm::new();
    let source = r#"
        let space = tuplespace.new()
        space.out("event", 42)
        let result = space.in("event")
        result.data
    "#;
    let result = eval(&mut vm, source).unwrap();
    assert_eq!(result, fmpl_core::value::Value::Int(42));
}

#[test]
fn test_tuplespace_out_and_rd() {
    let mut vm = fmpl_core::Vm::new();
    let source = r#"
        let space = tuplespace.new()
        space.out("event", 42)
        let result = space.rd("event")
        result.data
    "#;
    let result = eval(&mut vm, source).unwrap();
    assert_eq!(result, fmpl_core::value::Value::Int(42));
}

#[test]
fn test_tuplespace_rd_is_non_destructive() {
    let mut vm = fmpl_core::Vm::new();
    let source = r#"
        let space = tuplespace.new()
        space.out("event", 42)
        let r1 = space.rd("event")
        let r2 = space.rd("event")
        r2.data
    "#;
    let result = eval(&mut vm, source).unwrap();
    assert_eq!(result, fmpl_core::value::Value::Int(42));
}

#[test]
fn test_tuplespace_out_with_keyword_type() {
    let mut vm = fmpl_core::Vm::new();
    let source = r#"
        let space = tuplespace.new()
        space.out(:log, "error message")
        let result = space.in(:log)
        result.data
    "#;
    let result = eval(&mut vm, source).unwrap();
    assert_eq!(
        result,
        fmpl_core::value::Value::String("error message".into())
    );
}

#[test]
fn test_tuplespace_with_map_data() {
    let mut vm = fmpl_core::Vm::new();
    let source = r#"
        let space = tuplespace.new()
        space.out("event", %{level: :error, message: "failed"})
        let result = space.in("event")
        result.data.message
    "#;
    let result = eval(&mut vm, source).unwrap();
    assert_eq!(result, fmpl_core::value::Value::String("failed".into()));
}
