use fmpl_core::{Compiler, Lexer, Parser, Result, Value, Vm};

fn eval(vm: &mut Vm, source: &str) -> Result<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::with_source(&tokens, source).parse()?;
    let code = Compiler::new().compile(&ast)?;
    vm.run(&code)
}

#[test]
fn parse_stream_from_string_head() {
    let mut vm = Vm::new();
    // stream::new("hello") creates a ParseStream from a string
    // .head() returns the first character as a string
    let result = eval(&mut vm, r#"let s = stream::new("hello"); s.head()"#).unwrap();
    assert_eq!(result, Value::String("h".into()));
}

#[test]
fn parse_stream_from_string_position() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"let s = stream::new("hello"); s.position()"#).unwrap();
    assert_eq!(result, Value::Int(0));
}
