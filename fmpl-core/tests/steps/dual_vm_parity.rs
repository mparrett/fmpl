//! `dual_vm_parity` step-def for SCENARIO-0109 (ITER-0004x).
//!
//! Compiles a FMPL source string through the in-tree FMPL pipeline
//! (Lexer → Parser → Compiler → `CompiledCode`), then runs the resulting
//! bytecode through BOTH paths:
//!
//! - **Path A (in-tree):** `fmpl_core::Vm::new().run(&code)` →
//!   `fmpl_core::Value`.
//! - **Path B (cross-compiled):** `cross_compile(&code)` →
//!   `execution_tape::verifier::VerifiedProgram` →
//!   `execution_tape::vm::Vm::run(..)` → `Vec<execution_tape::vm::Value>`,
//!   then mapped back to `fmpl_core::Value` via the table documented in
//!   `cross_compile.rs`.
//!
//! Result equality is asserted; mismatches fail the case with a diff of
//! the two values.
//!
//! Required field on each case:
//!   - `source` (String) — FMPL source the parity gate evaluates.
//!
//! Feature-gated behind `cross_compile` because the `execution_tape`
//! dependency is optional. The crate-level `#![cfg(...)]` keeps this
//! file out of default-feature builds entirely.

#![cfg(feature = "cross_compile")]

use fmpl_core::Vm;
use fmpl_core::compiler::{CompiledCode, Compiler};
use fmpl_core::cross_compile::cross_compile;
use fmpl_core::lexer::Lexer;
use fmpl_core::parser::Parser;
use fmpl_core::value::Value as FmplValue;
use fmpl_scenario_runner::corpus::{Card, Case};
use fmpl_scenario_runner::error::StepError;
use fmpl_scenario_runner::step_def::{StepDef, StepDefRegistration};

use execution_tape::host::{AccessSink, Host, HostError, SigHash, ValueRef};
use execution_tape::trace::TraceMask;
use execution_tape::value::{FuncId, Value as TapeValue};
use execution_tape::vm::{Limits, Vm as TapeVm};

/// Minimal `Host` impl for parity-gate runs. The supported-opcode subset
/// of `cross_compile` does not emit any host calls, so this never fires
/// in normal SCENARIO-0109 cases. We still need *some* `Host` to satisfy
/// `Vm::new`'s signature; returning `UnknownSymbol` makes any unexpected
/// host call trap loudly rather than silently succeeding.
struct NullHost;

impl Host for NullHost {
    fn call(
        &mut self,
        _symbol: &str,
        _sig_hash: SigHash,
        _args: &[ValueRef<'_>],
        _rets: &mut [TapeValue],
        _access: Option<&mut dyn AccessSink>,
    ) -> Result<u64, HostError> {
        Err(HostError::UnknownSymbol)
    }
}

pub struct DualVmParity;

impl StepDef for DualVmParity {
    fn action_type(&self) -> &'static str {
        "dual_vm_parity"
    }

    fn run(&self, _card: &Card, case: &Case) -> Result<(), StepError> {
        let source = case
            .fields
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                StepError::new("dual_vm_parity: case missing required field `source`")
            })?;

        // === Step 1: compile FMPL source to CompiledCode ===
        let code = compile_source(source).map_err(|e| {
            StepError::new(format!("dual_vm_parity: failed to compile `{source}`: {e}"))
        })?;

        // === Step 2: run via in-tree Vm ===
        let mut fmpl_vm = Vm::new();
        let fmpl_result: FmplValue = fmpl_vm.run(&code).map_err(|e| {
            StepError::new(format!(
                "dual_vm_parity: in-tree Vm failed on `{source}`: {e:?}"
            ))
        })?;

        // === Step 3: cross-compile to execution_tape ===
        let tape_program = cross_compile(&code).map_err(|e| {
            StepError::new(format!(
                "dual_vm_parity: cross_compile failed on `{source}`: {e}"
            ))
        })?;

        // === Step 4: run via execution_tape Vm ===
        // The single function emitted by cross_compile is FuncId(0); no args;
        // tracing disabled.
        let mut tape_vm = TapeVm::new(NullHost, Limits::default());
        let tape_results = tape_vm
            .run(&tape_program, FuncId(0), &[], TraceMask::NONE, None)
            .map_err(|e| {
                StepError::new(format!(
                    "dual_vm_parity: execution_tape Vm trapped on `{source}`: {e:?}"
                ))
            })?;

        // === Step 5: convert tape result to fmpl value ===
        let tape_result_value: FmplValue = tape_results
            .first()
            .map(tape_to_fmpl_value)
            .unwrap_or(FmplValue::Null);

        // === Step 6: compare ===
        if fmpl_result != tape_result_value {
            return Err(StepError::new(format!(
                "dual_vm_parity: VMs disagree on `{source}`:\n  \
                 in-tree:        {fmpl_result:?}\n  \
                 execution_tape: {tape_result_value:?}"
            )));
        }

        Ok(())
    }
}

/// Map an `execution_tape` runtime `Value` to a `fmpl-core` `Value` for
/// parity comparison.
///
/// The mapping table is documented in `fmpl-core/src/cross_compile.rs`
/// (Value-equality semantics section). Variants outside the SCENARIO-0109
/// supported subset fall through to `Null` — those cases shouldn't arise
/// because the supported-opcode subset can only produce the listed
/// scalar variants.
fn tape_to_fmpl_value(v: &TapeValue) -> FmplValue {
    match v {
        TapeValue::Unit => FmplValue::Null,
        TapeValue::Bool(b) => FmplValue::Bool(*b),
        TapeValue::I64(n) => FmplValue::Int(*n),
        TapeValue::F64(f) => FmplValue::Float(*f),
        TapeValue::Str(s) => FmplValue::String(s.as_str().into()),
        // U64, Decimal, Bytes, Obj, Agg, Func: not produced by the
        // supported-opcode subset of cross_compile. Map to Null so the
        // comparison surfaces a clean mismatch if the assumption breaks.
        _ => FmplValue::Null,
    }
}

/// Compile FMPL source through the in-tree pipeline to `CompiledCode`.
///
/// Pipeline: `Lexer::new(src).tokenize()` →
/// `Parser::with_source(&tokens, src).parse()` →
/// `Compiler::new().compile(&ast)`. Mirrors `eval_via_legacy_parser` in
/// `lib.rs` but returns the `CompiledCode` instead of running it.
fn compile_source(source: &str) -> Result<CompiledCode, String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| format!("lex: {e:?}"))?;
    let ast = Parser::with_source(&tokens, source)
        .parse()
        .map_err(|e| format!("parse: {e:?}"))?;
    Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {e:?}"))
}

inventory::submit! { StepDefRegistration(&DualVmParity) }
