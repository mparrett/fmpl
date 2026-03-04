//! Test that verifies the generated parser is actually being used.
//!
//! This test catches the case where the generated parser fails to compile
//! and the build script silently falls back to the legacy parser.
//!
//! Run with: cargo test -p fmpl-core --test generated_parser_compiles
//!
//! IMPORTANT: If this test fails to compile with "cannot find type `Value` in this scope"
//! or "cannot find type `Arc` in this scope" in generated_parser.rs, it means parser.rs
//! is missing the required imports:
//!   use crate::value::Value;
//!   use std::sync::Arc;
//!
//! This happened previously (Mar 2026) when the generated parser used these types
//! but parser.rs didn't import them, causing silent fallback to the legacy parser.

#[test]
fn test_generated_parser_has_required_imports() {
    // The generated parser uses Value and Arc types.
    // If parser.rs is missing these imports, this test will fail to compile.
    //
    // This test must be in a separate file because it needs to fail at
    // COMPILE time, not at runtime.

    // Force the generated parser to be included
    let _ = fmpl_core::parser::generated_parse("42");

    // If we compile successfully, the imports are present
    // (test passes if compilation succeeds — assertion is intentionally trivial)
}

#[test]
fn test_generated_parser_not_fallback() {
    // Verify we're actually using the generated parser, not the fallback
    let source = "42";

    // This should use the generated parser
    let result = fmpl_core::parser::generated_parse(source);

    // If we're using the fallback (stub implementation), it will return an error
    // The real generated parser should successfully parse "42"
    assert!(
        result.is_ok(),
        "Generated parser failed to parse '42' - may be using fallback implementation"
    );
}

#[test]
fn test_generated_parser_basic_expression() {
    // Test a more complex expression to ensure the generated parser works
    let source = "1 + 2 * 3";

    let result = fmpl_core::parser::generated_parse(source);

    assert!(
        result.is_ok(),
        "Generated parser failed to parse '{}': {:?}",
        source,
        result.err()
    );
}

#[test]
fn test_generated_parser_function_call() {
    // Test function call syntax
    let source = "foo(1, 2, 3)";

    let result = fmpl_core::parser::generated_parse(source);

    assert!(
        result.is_ok(),
        "Generated parser failed to parse '{}': {:?}",
        source,
        result.err()
    );
}

#[test]
fn test_generated_parser_list_literal() {
    // Test list literal syntax
    let source = "[1, 2, 3]";

    let result = fmpl_core::parser::generated_parse(source);

    assert!(
        result.is_ok(),
        "Generated parser failed to parse '{}': {:?}",
        source,
        result.err()
    );
}

#[test]
fn test_generated_parser_qualified_name() {
    // Test qualified name syntax (e.g., std::foo)
    let source = "std::foo::bar(42)";

    let result = fmpl_core::parser::generated_parse(source);

    assert!(
        result.is_ok(),
        "Generated parser failed to parse '{}': {:?}",
        source,
        result.err()
    );
}
