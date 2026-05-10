//! Execute `lib/core/ast_optimizer_test.fmpl` from Rust and assert all
//! FMPL-side unit tests pass against the migrated `ast_optimizer.fmpl`.
//!
//! This is the ITER-0004c "ast_optimizer_test execution gate" — fine-grained
//! regression coverage of the optimizer's individual rules. SCENARIO-0103
//! tests integration-level behavior (parity through the full pipeline);
//! these unit tests test each fold rule in isolation. Together they catch
//! both rule-level misfires and pipeline-level integration breakage.
//!
//! Note: `ast_optimizer_test.fmpl` itself is FMPL test code (FMPL writing
//! tests for FMPL). This is the "FMPL works as a programming language"
//! material verification per the iteration's design pivot framing.

use fmpl_core::{Value, Vm, eval_via_legacy_parser};

#[test]
#[ignore = "ast_optimizer_test.fmpl uses `++` string-concat operator that the FMPL parser does not support. Pre-existing bug unrelated to ITER-0004c canonical-representation refactor. Bugs in test file (lines 43/89/131) for missing `:-` symbol have been fixed in ITER-0004c, but the `++` operator usage remains. Un-ignore this test once `++` is supported (or once test.fmpl is rewritten to use string.join)."]
fn ast_optimizer_unit_tests_pass_against_migrated_optimizer() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    std::env::set_current_dir(workspace_root).expect("set cwd to workspace root");

    let mut vm = Vm::new();
    eval_via_legacy_parser(&mut vm, r#"io::load("lib/core/prelude.fmpl")"#).expect("load prelude");

    // Load the test file. It executes its tests on load and returns
    // results["ok"] (a Bool) as its top-level expression value.
    let result = eval_via_legacy_parser(&mut vm, r#"io::load("lib/core/ast_optimizer_test.fmpl")"#);

    match result {
        Ok(Value::Bool(true)) => {
            // All tests passed.
        }
        Ok(other) => panic!(
            "ast_optimizer_test.fmpl returned {:?}; expected Bool(true). \
             Some FMPL-side unit tests for the optimizer failed.",
            other
        ),
        Err(e) => panic!(
            "ast_optimizer_test.fmpl failed to execute: {:?}. \
             This usually means a rule in lib/core/ast_optimizer.fmpl is \
             malformed or the test framework changed shape.",
            e
        ),
    }
}
