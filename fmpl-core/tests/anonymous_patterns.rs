//! Tests for anonymous inline pattern blocks in the @ operator.
//!
//! This tests the new syntax: x @ { %{foo: f} => f, _ => default }
//! which provides inline pattern matching (like match expressions)
//! directly in the @ operator context.
//!
//! Task 3.1 adds AST support and parsing for inline pattern blocks.
//! Task 3.2 adds compilation to bytecode and execution tests.

use fmpl_core::Lexer;
use fmpl_core::Parser;
use fmpl_core::ast::{Expr, Pattern};
use fmpl_core::{Value, Vm};

fn parse_with_legacy(src: &str) -> Result<Expr, String> {
    let tokens = Lexer::new(src).tokenize().map_err(|e| e.to_string())?;
    Parser::with_source(&tokens, src)
        .parse()
        .map_err(|e| e.to_string())
}

// =============================================================================
// Parsing tests - verify AST structure (Task 3.1 scope)
// =============================================================================

/// Test parsing of inline pattern block with variable pattern
/// Note: `_` (wildcard) goes to grammar parsing for backward compatibility.
/// Use variable pattern (identifier followed by =>) for inline blocks.
#[test]
fn test_parse_inline_pattern_block() {
    // Verify parsing x @ { n => n + 1 } produces InlinePatternBlock AST
    // (variable pattern followed by => triggers inline parsing)
    let code = r#"42 @ { n => n + 1 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    // Check that it's an InlinePatternBlock
    match expr {
        Expr::InlinePatternBlock { input, cases } => {
            // Input should be 42
            match *input {
                Expr::Int(n) => assert_eq!(n, 42),
                _ => panic!("Expected Int(42) as input, got {:?}", input),
            }
            // Should have one case with variable pattern
            assert_eq!(cases.len(), 1);
            match &cases[0].pattern {
                Pattern::Var(name) => assert_eq!(name.as_str(), "n"),
                _ => panic!("Expected Var pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test parsing of inline pattern block with guard
#[test]
fn test_parse_inline_block_with_guard() {
    // Verify parsing x @ { n when n > 0 => n, _ => 0 }
    let code = r#"5 @ { n when n > 0 => n, _ => 0 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            // Should have two cases
            assert_eq!(cases.len(), 2);
            // First case should have a guard
            assert!(cases[0].guard.is_some(), "First case should have guard");
            // Second case should not have a guard
            assert!(
                cases[1].guard.is_none(),
                "Second case (wildcard) should not have guard"
            );
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test that variable patterns are recognized as inline pattern blocks
#[test]
fn test_variable_pattern_detected() {
    // Variable followed by => should trigger inline pattern parsing
    let code = r#"42 @ { x => x + 1 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            assert_eq!(cases.len(), 1);
            // Should be a variable pattern
            match &cases[0].pattern {
                Pattern::Var(name) => assert_eq!(name.as_str(), "x"),
                _ => panic!("Expected Var pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test map pattern detection
#[test]
fn test_map_pattern_detected() {
    // %{ starts map pattern, should be detected as inline block
    let code = r#"%{a: 1} @ { %{a: x} => x, _ => 0 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            assert_eq!(cases.len(), 2);
            // First case should be a map pattern
            match &cases[0].pattern {
                Pattern::Map(entries) => {
                    assert_eq!(entries.len(), 1);
                    assert_eq!(entries[0].0.as_str(), "a");
                }
                _ => panic!("Expected Map pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test list pattern detection
#[test]
fn test_list_pattern_detected() {
    // [ starts list pattern, should be detected as inline block
    let code = r#"[1, 2] @ { [a, b] => a + b, _ => 0 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            assert_eq!(cases.len(), 2);
            // First case should be a list pattern
            match &cases[0].pattern {
                Pattern::List(pats, _) => assert_eq!(pats.len(), 2),
                _ => panic!("Expected List pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test literal pattern detection
#[test]
fn test_literal_pattern_detected() {
    // Literal followed by => should trigger inline pattern parsing
    let code = r#"1 @ { 1 => "one", _ => "other" }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            assert_eq!(cases.len(), 2);
            // First case should be an int literal pattern
            match &cases[0].pattern {
                Pattern::Int(n) => assert_eq!(*n, 1),
                _ => panic!("Expected Int pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test symbol/constructor pattern detection
#[test]
fn test_symbol_pattern_detected() {
    // :Symbol should trigger inline pattern parsing
    let code = r#":foo @ { :foo => 1, _ => 0 }"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::InlinePatternBlock { cases, .. } => {
            assert_eq!(cases.len(), 2);
            // First case should be a symbol pattern
            match &cases[0].pattern {
                Pattern::Symbol(s) => assert_eq!(s.as_str(), "foo"),
                _ => panic!("Expected Symbol pattern, got {:?}", cases[0].pattern),
            }
        }
        _ => panic!("Expected InlinePatternBlock, got {:?}", expr),
    }
}

/// Test grammar application with named rule (not inline block)
#[test]
fn test_grammar_application_still_works() {
    // Grammar application syntax: input @ grammar.rule
    // This is NOT an inline pattern block - it's a grammar reference
    let code = r#""hello" @ base::parser.word"#;

    let ast = parse_with_legacy(code);
    assert!(ast.is_ok(), "Parse failed: {:?}", ast);

    let expr = ast.unwrap();
    match expr {
        Expr::GrammarApply { grammar, rule, .. } => {
            // Should be a grammar application
            assert_eq!(rule.as_str(), "word");
            match *grammar {
                Expr::Qualified(ref qn) => {
                    assert_eq!(qn.to_string(), "base::parser");
                }
                _ => panic!("Expected Qualified grammar, got {:?}", grammar),
            }
        }
        _ => panic!("Expected GrammarApply, got {:?}", expr),
    }
}

// =============================================================================
// Execution tests - verify inline pattern blocks produce correct results (Task 3.2)
// =============================================================================
//
// NOTE: These tests require the legacy parser (FMPL_USE_LEGACY_PARSER=1) because
// the generated parser doesn't support the `match` keyword yet. The inline pattern
// block `x @ { pattern => body }` compiles to a Match expression internally.
//
// Run with: FMPL_USE_LEGACY_PARSER=1 cargo test -p fmpl-core --test anonymous_patterns

mod execution {
    use super::*;
    use fmpl_core::{Compiler, Lexer, Parser};

    /// Helper to eval with legacy parser
    fn eval_legacy(vm: &mut Vm, source: &str) -> Result<Value, String> {
        let tokens = Lexer::new(source).tokenize().map_err(|e| e.to_string())?;
        let ast = Parser::with_source(&tokens, source)
            .parse()
            .map_err(|e| e.to_string())?;
        let code = Compiler::new().compile(&ast).map_err(|e| e.to_string())?;
        vm.run(&code).map_err(|e| e.to_string())
    }

    /// Test variable pattern extracts value and returns body result
    #[test]
    fn test_variable_pattern_returns_body() {
        let mut vm = Vm::new();
        // Variable pattern `n` binds 42 to n, body returns n + 1 = 43
        let result = eval_legacy(&mut vm, r#"42 @ { n => n + 1 }"#).unwrap();
        assert_eq!(result, Value::Int(43));
    }

    /// Test variable pattern with different expression body
    #[test]
    fn test_variable_pattern_string_concat() {
        let mut vm = Vm::new();
        // Variable pattern with string concatenation
        let result = eval_legacy(&mut vm, r#""world" @ { s => "hello " + s }"#).unwrap();
        assert!(
            matches!(result, Value::String(ref s) if s == "hello world"),
            "got {:?}",
            result
        );
    }

    /// Test wildcard pattern (via inline pattern path)
    /// Note: `_` with no binding goes through grammar path for backward compat.
    /// Use named variable for inline pattern execution.
    #[test]
    fn test_named_wildcard_returns_body() {
        let mut vm = Vm::new();
        // Using `x` as a wildcard-like pattern that discards the value
        let result = eval_legacy(&mut vm, r#"42 @ { x => "matched" }"#).unwrap();
        assert!(
            matches!(result, Value::String(ref s) if s == "matched"),
            "got {:?}",
            result
        );
    }

    /// Test guard clause with variable pattern
    #[test]
    fn test_guard_clause_passes() {
        let mut vm = Vm::new();
        // First case: n when n > 10 => 100 (fails for input 5)
        // Second case: n => n * 2 (succeeds for input 5)
        let result = eval_legacy(&mut vm, r#"5 @ { n when n > 10 => 100, n => n * 2 }"#).unwrap();
        assert_eq!(result, Value::Int(10)); // 5 * 2
    }

    /// Test guard clause that matches
    #[test]
    fn test_guard_clause_matches() {
        let mut vm = Vm::new();
        // First case: n when n > 10 => 100 (succeeds for input 15)
        let result = eval_legacy(&mut vm, r#"15 @ { n when n > 10 => 100, n => n * 2 }"#).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    /// Test multiple cases with guards
    #[test]
    fn test_multiple_guards() {
        let mut vm = Vm::new();
        // Classify numbers: small (< 5), medium (5-10), large (> 10)
        let result = eval_legacy(
            &mut vm,
            r#"7 @ { n when n < 5 => "small", n when n > 10 => "large", n => "medium" }"#,
        )
        .unwrap();
        assert!(
            matches!(result, Value::String(ref s) if s == "medium"),
            "got {:?}",
            result
        );
    }

    /// Test that body expression result is returned, not scrutinee
    #[test]
    fn test_body_expression_computed() {
        let mut vm = Vm::new();
        // Complex body expression
        let result = eval_legacy(&mut vm, r#"10 @ { n => n * n + n }"#).unwrap();
        assert_eq!(result, Value::Int(110)); // 10*10 + 10 = 110
    }

    // =========================================================================
    // Complex pattern tests - these require full pattern compilation support
    // which is planned for future tasks. Currently only variable/wildcard patterns
    // are supported in match compilation.
    // =========================================================================

    /// Test literal pattern matching via inline pattern
    #[test]
    #[ignore = "requires literal pattern compilation (not yet implemented)"]
    fn test_literal_pattern_match() {
        let mut vm = Vm::new();
        // Match literal 1
        let result = eval_legacy(&mut vm, r#"1 @ { 1 => "one", n => "other" }"#).unwrap();
        assert!(
            matches!(result, Value::String(ref s) if s == "one"),
            "got {:?}",
            result
        );
    }

    /// Test literal pattern fallthrough
    #[test]
    #[ignore = "requires literal pattern compilation (not yet implemented)"]
    fn test_literal_pattern_fallthrough() {
        let mut vm = Vm::new();
        // Literal 1 doesn't match 2, falls through to variable pattern
        let result = eval_legacy(&mut vm, r#"2 @ { 1 => "one", n => "other" }"#).unwrap();
        assert!(
            matches!(result, Value::String(ref s) if s == "other"),
            "got {:?}",
            result
        );
    }

    /// Test list pattern extraction
    #[test]
    #[ignore = "requires list pattern compilation (not yet implemented)"]
    fn test_list_pattern_extraction() {
        let mut vm = Vm::new();
        // List pattern extracts elements
        let result = eval_legacy(&mut vm, r#"[1, 2] @ { [a, b] => a + b, x => 0 }"#).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    /// Test symbol pattern matching
    #[test]
    #[ignore = "requires symbol pattern compilation (not yet implemented)"]
    fn test_symbol_pattern_match() {
        let mut vm = Vm::new();
        // Symbol pattern matches :foo
        let result = eval_legacy(&mut vm, r#":foo @ { :foo => 1, x => 0 }"#).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    /// Test symbol pattern fallthrough
    #[test]
    #[ignore = "requires symbol pattern compilation (not yet implemented)"]
    fn test_symbol_pattern_fallthrough() {
        let mut vm = Vm::new();
        // Symbol :bar doesn't match :foo
        let result = eval_legacy(&mut vm, r#":bar @ { :foo => 1, x => 0 }"#).unwrap();
        assert_eq!(result, Value::Int(0));
    }

    /// Test map pattern extraction
    #[test]
    #[ignore = "requires map pattern compilation (not yet implemented)"]
    fn test_map_pattern_extraction() {
        let mut vm = Vm::new();
        // Map pattern extracts value. Note: simpler syntax %{x: a} binds a to the value of x
        let result = eval_legacy(&mut vm, r#"%{x: 10, y: 20} @ { %{x: a} => a, n => 0 }"#).unwrap();
        assert_eq!(result, Value::Int(10));
    }
}
