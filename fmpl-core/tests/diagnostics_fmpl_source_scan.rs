//! Unit tests for `fmpl_core::diagnostics::scan_fmpl_source` and the
//! `scan_rust_strings` test helper (ITER-0004d.0).
//!
//! These fixtures deliberately contain `:Foo(1, 2)` style strings — that's
//! the point. The CI gate (`no_legacy_fmpl_syntax.rs`) explicitly excludes
//! this file from its `tests/*.rs` scan surface to avoid flagging the
//! fixtures as real legacy hits.

use std::path::PathBuf;

use fmpl_core::diagnostics::{SourceKind, scan_fmpl_source};

mod common;

fn fmpl_source(path: &str) -> SourceKind {
    SourceKind::FmplFile {
        path: PathBuf::from(path),
    }
}

#[test]
fn scan_fmpl_source_uppercase_tagged_constructor_has_one_hit() {
    let src = r#"let x = :Foo(1, 2)"#;
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert_eq!(hits.len(), 1, "expected 1 hit, got {:?}", hits);
    assert_eq!(hits[0].tag.as_str(), "Foo");
}

#[test]
fn scan_fmpl_source_lowercase_tagged_constructor_has_one_hit() {
    let src = r#"let x = :foo(1, 2)"#;
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert_eq!(hits.len(), 1, "expected 1 hit, got {:?}", hits);
    assert_eq!(hits[0].tag.as_str(), "foo");
}

#[test]
fn scan_fmpl_source_list_pattern_has_zero_hits() {
    // `[:Foo, 1, 2]` — Symbol("Foo") is followed by Comma, not LParen.
    let src = r#"let x = [:Foo, 1, 2]"#;
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert!(hits.is_empty(), "expected 0 hits, got {:?}", hits);
}

#[test]
fn scan_fmpl_source_bare_symbol_has_zero_hits() {
    let src = r#"let x = :foo"#;
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert!(hits.is_empty(), "expected 0 hits, got {:?}", hits);
}

#[test]
fn scan_fmpl_source_comment_has_zero_hits() {
    let src = "-- :Foo(1, 2) in a comment\nlet x = 1";
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert!(hits.is_empty(), "expected 0 hits, got {:?}", hits);
}

#[test]
fn scan_fmpl_source_lexer_error_is_propagated() {
    // BELL char (\x07) outside any string literal triggers a lexer error,
    // which `scan_fmpl_source` surfaces as `DiagnosticsError::LexerError`.
    let src = "let x = \x07";
    let result = scan_fmpl_source(src, fmpl_source("test.fmpl"));
    assert!(
        result.is_err(),
        "expected lexer error to propagate, got {:?}",
        result
    );
}

#[test]
fn scan_fmpl_source_multiple_hits_preserves_order_and_offsets() {
    let src = r#":Foo(1) :Bar(2)"#;
    let hits = scan_fmpl_source(src, fmpl_source("test.fmpl")).expect("lex ok");
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].tag.as_str(), "Foo");
    assert_eq!(hits[1].tag.as_str(), "Bar");
    assert!(
        hits[0].byte_offset < hits[1].byte_offset,
        "offsets must preserve source order"
    );
}

// --- scan_rust_strings (test-only helper) ---

#[test]
fn scan_rust_strings_finds_hit_in_string_literal_only() {
    // Rust source: a string literal containing `:Foo(1, 2)` (one real hit) AND
    // a Rust qualified path `Pattern::Constructor("Foo", vec![])` (NOT a hit —
    // qualified paths are syntax-tree-path nodes, not string literals).
    let rust_src = r##"
fn example() {
    let s = "let x = :Foo(1, 2)";
    let _ = s;
}
"##;
    let hits =
        common::rust_string_scanner::scan_rust_strings(rust_src, &PathBuf::from("example.rs"))
            .expect("syn parses example.rs");
    assert_eq!(hits.len(), 1, "expected 1 hit, got {:?}", hits);
    assert_eq!(hits[0].tag.as_str(), "Foo");
}

#[test]
fn scan_rust_strings_ignores_rust_qualified_paths() {
    // No string literal contains `:Foo(`. The `Pattern::Constructor(` is a
    // Rust qualified path — never reaches scan_fmpl_source.
    let rust_src = r##"
enum Pattern { Constructor(String, Vec<i32>) }
fn example() {
    let _p = Pattern::Constructor("Foo".to_string(), vec![]);
}
"##;
    let hits =
        common::rust_string_scanner::scan_rust_strings(rust_src, &PathBuf::from("example.rs"))
            .expect("syn parses example.rs");
    assert!(hits.is_empty(), "expected 0 hits, got {:?}", hits);
}

#[test]
fn scan_rust_strings_raw_string_with_list_pattern_has_zero_hits() {
    let rust_src = r####"
fn example() {
    let _r = r#"[:Foo, 1, 2]"#;
}
"####;
    let hits =
        common::rust_string_scanner::scan_rust_strings(rust_src, &PathBuf::from("example.rs"))
            .expect("syn parses example.rs");
    assert!(hits.is_empty(), "expected 0 hits, got {:?}", hits);
}

#[test]
fn scan_rust_strings_swallows_lexer_errors_silently() {
    // String literal containing a BELL char — FMPL lexer fails. Per the
    // documented swallow policy (roadmap line 349), scan_rust_strings returns
    // Ok with the hits it COULD find from other literals (here: none).
    let rust_src = "fn example() { let s = \"bell \\x07 char\"; let _ = s; }";
    let hits =
        common::rust_string_scanner::scan_rust_strings(rust_src, &PathBuf::from("example.rs"))
            .expect("syn parses example.rs");
    assert!(
        hits.is_empty(),
        "expected 0 hits (swallow policy), got {:?}",
        hits
    );
}
