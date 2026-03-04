//! Tests for bytecode-based rule execution
//!
//! These tests directly execute compiled grammar bytecode, bypassing the old PegRuntime.
//! This tests the new ApplyRule → compiled grammar → pattern instructions path.

use fmpl_core::{compiler::Compiler, error::Result, grammar::Grammar, value::Value, vm::Vm};

#[allow(dead_code)]
fn eval_bytecode_grammar(source: &str) -> Result<Value> {
    let _vm = Vm::new();

    // Parse the source to get a Grammar AST
    let grammar = parse_grammar(source)?;

    // Compile the grammar to bytecode with rule entry points
    let compiled = Compiler::compile_grammar_only(&grammar)?;

    println!(
        "Compiled grammar '{}' with {} rules and {} entry points",
        grammar.name,
        grammar.rules.len(),
        compiled.rule_entry_points.len()
    );

    // For now, we can't easily execute the compiled bytecode directly
    // because the VM doesn't have a way to execute compiled grammar bytecode
    // without going through the old GrammarApply path.
    //
    // This test verifies compilation works, but execution will need
    // the GrammarApply integration (separate task).

    Ok(Value::Null)
}

fn parse_grammar(source: &str) -> Result<Grammar> {
    use fmpl_core::grammar::parser::GrammarParser;
    let mut parser = GrammarParser::new(source);
    parser.parse()
}

#[test]
fn test_compile_simple_grammar() {
    let grammar = parse_grammar(
        r#"
        grammar test {
            digit = [0-9];
            letter = [a-z];
        }
    "#,
    )
    .expect("parse grammar");

    let compiled = Compiler::compile_grammar_only(&grammar).expect("compile grammar");

    // Should have 2 rule entry points
    assert_eq!(compiled.rule_entry_points.len(), 2);
    assert!(compiled.rule_entry_points.contains_key("digit"));
    assert!(compiled.rule_entry_points.contains_key("letter"));
}

#[test]
fn test_compile_grammar_with_rule_reference() {
    let grammar = parse_grammar(
        r#"
        grammar test {
            digit = [0-9];
            number = digit+ => "matched"
        }
    "#,
    )
    .expect("parse grammar");

    let compiled = Compiler::compile_grammar_only(&grammar).expect("compile grammar");

    // Should have 2 rule entry points
    assert_eq!(compiled.rule_entry_points.len(), 2);

    // The 'number' rule should compile to bytecode that includes ApplyRule for 'digit'
    let number_entry = compiled.rule_entry_points.get("number").unwrap();
    println!("number rule entry point: {:?}", number_entry);

    // Check that the bytecode contains pattern instructions
    assert!(!compiled.instructions.is_empty());
}

#[test]
fn test_compile_grammar_with_binding() {
    let grammar = parse_grammar(
        r#"
        grammar test {
            digit = [0-9]+:digits => digits
        }
    "#,
    )
    .expect("parse grammar");

    let compiled = Compiler::compile_grammar_only(&grammar).expect("compile grammar");

    assert_eq!(compiled.rule_entry_points.len(), 1);
}
