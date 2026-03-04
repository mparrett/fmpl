//! Integration tests for the FMPL compilation pipeline as default backend.
//!
//! Verifies that `eval_via_fmpl_pipeline` produces identical results to
//! `eval_via_legacy_parser` for supported expression types. This is the
//! ITER-0004 milestone: FMPL is its own compiler.

use fmpl_core::{Value, Vm, eval_via_fmpl_pipeline, eval_via_legacy_parser};

/// Run source through both compilers and assert results match.
fn assert_compilers_agree(source: &str) {
    // cargo test sets cwd to the crate directory; the FMPL pipeline needs
    // workspace-root for io::load("lib/core/...").
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(workspace_root).expect("set cwd to workspace root");

    let mut vm_rust = Vm::new();
    let rust_result = eval_via_legacy_parser(&mut vm_rust, source)
        .unwrap_or_else(|e| panic!("Rust compiler failed for {:?}: {}", source, e));

    let mut vm_fmpl = Vm::new();
    let fmpl_result = eval_via_fmpl_pipeline(&mut vm_fmpl, source)
        .unwrap_or_else(|e| panic!("FMPL pipeline failed for {:?}: {}", source, e));

    assert_eq!(
        rust_result, fmpl_result,
        "compilers disagree for {:?}: rust={:?}, fmpl={:?}",
        source, rust_result, fmpl_result
    );
}

#[test]
fn integer_literal() {
    assert_compilers_agree("42");
}

#[test]
fn arithmetic_with_precedence() {
    assert_compilers_agree("1 + 2 * 3");
}

#[test]
fn string_literal() {
    assert_compilers_agree("\"hello\"");
}

#[test]
fn let_binding() {
    assert_compilers_agree("let (x = 42) x + 1");
}

#[test]
fn if_expression() {
    assert_compilers_agree("if true then 1 else 2");
}

#[test]
fn lambda_call() {
    assert_compilers_agree("let (f = \\x x + 1) f(41)");
}

#[test]
fn list_construction() {
    assert_compilers_agree("[1, 2, 3]");
}

#[test]
fn nested_arithmetic() {
    assert_compilers_agree("(1 + 2) * (3 + 4)");
}

#[test]
fn boolean_logic() {
    assert_compilers_agree("true && false");
}

#[test]
fn comparison() {
    assert_compilers_agree("3 < 5");
}

#[test]
fn fmpl_pipeline_caches_bootstrap() {
    // Calling eval_via_fmpl_pipeline twice on the same VM should not reload
    // prelude.fmpl and ast_to_ir.fmpl. This is a smoke test — if the bootstrap
    // marker check works, two calls should both succeed quickly.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(workspace_root).expect("set cwd");

    let mut vm = Vm::new();
    let r1 = eval_via_fmpl_pipeline(&mut vm, "1 + 1").expect("first call");
    let r2 = eval_via_fmpl_pipeline(&mut vm, "2 + 2").expect("second call");
    assert_eq!(r1, Value::Int(2));
    assert_eq!(r2, Value::Int(4));
}
