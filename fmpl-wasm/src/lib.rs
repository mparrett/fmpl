//! Browser bindings for the FMPL VM (fork issue #3).
//!
//! Exposes one persistent VM per wasm instance so `let` bindings, objects,
//! and grammars survive across REPL inputs — the same session model as the
//! CLI REPL. Built for wasm32-unknown-unknown and loaded by
//! `docs/repl.html` on the GitHub Pages site.

use fmpl_core::{Vm, eval, is_complete};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static VM: RefCell<Vm> = RefCell::new(Vm::new());
}

/// Evaluate FMPL source against the page's persistent VM.
///
/// Returns the REPL-style result line: `=> value` on success or
/// `Error: ...` on failure, matching fmpl-cli's output format.
#[wasm_bindgen]
pub fn repl_eval(source: &str) -> String {
    VM.with(|vm| match eval(&mut vm.borrow_mut(), source) {
        Ok(value) => format!("=> {value}"),
        Err(e) => format!("Error: {e}"),
    })
}

/// True if `source` is a complete expression/statement sequence. The page
/// uses this for multi-line continuation, like the CLI REPL. A source that
/// fails to lex counts as complete so evaluation surfaces the real error.
#[wasm_bindgen]
pub fn repl_is_complete(source: &str) -> bool {
    is_complete(source).unwrap_or(true)
}

/// Discard the persistent VM, starting a fresh session.
#[wasm_bindgen]
pub fn repl_reset() {
    VM.with(|vm| *vm.borrow_mut() = Vm::new());
}

/// An independent VM instance, for pages that need more than the one global
/// session — e.g. the language guide's run-in-place blocks, where each run
/// gets a fresh VM (the doctest harness's semantics) while a console keeps
/// its own long-lived session.
#[wasm_bindgen]
pub struct ReplVm {
    vm: Vm,
}

#[wasm_bindgen]
impl ReplVm {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)] // wasm-bindgen constructors are `new` by contract
    pub fn new() -> ReplVm {
        ReplVm { vm: Vm::new() }
    }

    /// Evaluate source against this instance's VM. Same output contract as
    /// [`repl_eval`]: `=> value` or `Error: ...`.
    pub fn eval(&mut self, source: &str) -> String {
        match eval(&mut self.vm, source) {
            Ok(value) => format!("=> {value}"),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Same completeness check as [`repl_is_complete`].
    pub fn is_complete(&self, source: &str) -> bool {
        is_complete(source).unwrap_or(true)
    }
}
