//! AC-7 runnable check (ITER-0004c): the optimizer treats the enumerated
//! pass-through node kinds as opaque — they pass through structurally
//! unchanged.
//!
//! AC-7 documentation half: a TODO comment in `lib/core/ast_optimizer.fmpl`
//! enumerates the AST node kinds that fall through unchanged: `:Lambda`,
//! `:Let`, `:Match`, `:Call`, `:List`, `:Map`, `:Block`. This file is the
//! behavior half — for each enumerated kind, we construct an AST whose top
//! level is that kind and whose interior contains a foldable `:Binary`. The
//! optimizer should NOT recurse into the body, so the inner Binary is NOT
//! folded. The output equals the input structurally.
//!
//! This test is intentionally a hedge: if a future change adds recurse-into
//! rules for any of these kinds, the inner Binary would fold and the
//! structural-identity assertion would fail. That failure forces the
//! AC-7 TODO comment to be updated in lockstep with the behavior change.
//!
//! Verified 2026-05-10 against `lib/core/ast_optimizer.fmpl`:
//! `constant_fold` and `algebraic_simp` only have rewrite rules for
//! `:Binary`, `:Unary`, `:If`. Every other node kind hits `_:x => x` and is
//! returned unchanged.

use fmpl_core::{Vm, eval_via_legacy_parser};

fn setup() -> Vm {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::env::set_current_dir(workspace_root).expect("set cwd");

    let mut vm = Vm::new();
    eval_via_legacy_parser(&mut vm, r#"io::load("lib/core/prelude.fmpl")"#).expect("load prelude");
    eval_via_legacy_parser(
        &mut vm,
        r#"let ast_optimizer = io::load("lib/core/ast_optimizer.fmpl")"#,
    )
    .expect("load ast_optimizer");
    vm
}

/// For each pass-through kind, assert `optimize(input) == input` where
/// `input` has a foldable inner `[:Binary, :+, [:Int, 1], [:Int, 2]]` that
/// would fold to `[:Int, 3]` if the optimizer recursed.
fn assert_pass_through(vm: &mut Vm, input_expr: &str) {
    let optimized =
        eval_via_legacy_parser(vm, &format!(r#"ast_optimizer["optimize"]({})"#, input_expr))
            .unwrap_or_else(|e| panic!("optimize({}) failed: {:?}", input_expr, e));
    let expected = eval_via_legacy_parser(vm, input_expr)
        .unwrap_or_else(|e| panic!("eval({}) failed: {:?}", input_expr, e));
    assert_eq!(
        optimized, expected,
        "Pass-through invariant violated for {}: optimizer returned {:?}, expected {:?} (input unchanged)",
        input_expr, optimized, expected
    );
}

#[test]
fn lambda_body_passes_through_unchanged() {
    let mut vm = setup();
    // [:Lambda, [], [:Binary, :+, [:Int, 1], [:Int, 2]]]
    // The inner Binary would fold to [:Int, 3] IF the optimizer recursed
    // into Lambda bodies. It does not (per ast_optimizer.fmpl line 35 / 68
    // catch-all `_:x => x`), so the AST passes through unchanged.
    assert_pass_through(
        &mut vm,
        "[:Lambda, [], [:Binary, :+, [:Int, 1], [:Int, 2]]]",
    );
}

#[test]
fn let_body_passes_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(
        &mut vm,
        "[:Let, [[:Binding, :x, [:Int, 5]]], [:Binary, :+, [:Int, 1], [:Int, 2]]]",
    );
}

#[test]
fn match_arms_pass_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(
        &mut vm,
        "[:Match, [:Int, 42], [[:Case, [:PatternWildcard], null, [:Binary, :+, [:Int, 1], [:Int, 2]]]]]",
    );
}

#[test]
fn call_args_pass_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(
        &mut vm,
        "[:Call, [:Var, :f], [[:Binary, :+, [:Int, 1], [:Int, 2]]]]",
    );
}

#[test]
fn list_elements_pass_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(&mut vm, "[:List, [[:Binary, :+, [:Int, 1], [:Int, 2]]]]");
}

#[test]
fn map_values_pass_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(
        &mut vm,
        "[:Map, [[\"a\", [:Binary, :+, [:Int, 1], [:Int, 2]]]]]",
    );
}

#[test]
fn block_stmts_pass_through_unchanged() {
    let mut vm = setup();
    assert_pass_through(&mut vm, "[:Block, [[:Binary, :+, [:Int, 1], [:Int, 2]]]]");
}

/// Sanity check (anti-test): :Binary itself IS folded. If this fails, the
/// optimizer is broken and the pass-through tests above would also be
/// meaningless (they'd all pass trivially because nothing folds).
#[test]
fn sanity_binary_does_get_folded() {
    let mut vm = setup();
    let optimized = eval_via_legacy_parser(
        &mut vm,
        r#"ast_optimizer["optimize"]([:Binary, :+, [:Int, 1], [:Int, 2]])"#,
    )
    .expect("optimize binary");
    let expected = eval_via_legacy_parser(&mut vm, "[:Int, 3]").expect("build expected");
    assert_eq!(
        optimized, expected,
        "Sanity check failed: :Binary should fold to [:Int, 3] but got {:?}",
        optimized
    );
}
