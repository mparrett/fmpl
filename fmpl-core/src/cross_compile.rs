//! Cross-compiler from FMPL Indexed RPN IR to execution_tape bytecode.
//!
//! This module provides a direct compilation path from FMPL's IR to
//! execution_tape's verified bytecode, originally introduced for
//! performance comparison. As of ITER-0004x it is also a **dual-VM
//! parity gate** — `tests/scenario_runner.rs` runs SCENARIO-0109
//! cases through both the in-tree `Vm` AND the execution_tape VM
//! (via `cross_compile` + `execution_tape::vm::Vm::run`) and asserts
//! result equality. The parity gate is correctness-only (not perf).
//!
//! # Supported `Instruction` variants
//!
//! As of ITER-0004x the cross-compiler handles the FMPL "Indexed RPN"
//! subset of bytecode. Specifically these `Instruction` variants
//! compile cleanly:
//!
//! - **Literals:** `LoadInt`, `LoadFloat`, `LoadBool`, `LoadNull`,
//!   `LoadString`, `LoadSymbol` (Symbol maps to Str in execution_tape).
//! - **Variables:** `LoadVar` (resolves through the cross_compile-local
//!   name→bind-idx map populated during the codegen pass), `Bind`,
//!   `NameRef`.
//! - **Arithmetic:** `Add`, `Sub`, `Mul`, `Div`, `Mod`.
//! - **Comparison:** `Eq`, `NotEq`, `Lt`, `Gt`, `LtEq`, `GtEq`.
//! - **Unary:** `Neg`, `Not`.
//! - **List literals:** `MakeList`.
//! - **Register move:** `Copy { source }` (lowers to `mov`).
//! - **Scope markers (no-op):** `PushScope`, `PopScope`. The cross-compile
//!   target is a flat register file — no dynamic scope chain — so these
//!   instructions emit nothing. The return-value selector also skips
//!   them when picking the function's final value-producing instruction.
//!
//! # Return-type inference
//!
//! The function's `ret_types` is inferred from the `TapeType` of the
//! last value-producing instruction (i.e., the last instruction that is
//! not `PushScope`/`PopScope`). I64 / F64 / Bool / Str / Unit map to the
//! corresponding `ValueType`; `Unknown` falls back to `I64` (the
//! dual-VM parity gate in SCENARIO-0109 will catch a mismatch loudly if
//! the assumption is wrong). Pre-ITER-0004x the return type was
//! hardcoded `I64`, which silently broke Bool / F64 results — fixed by
//! the SCENARIO-0109 parity gate exercise.
//!
//! # NOT supported (returns `CrossCompileError::UnsupportedInstruction`)
//!
//! The following `Instruction` variants currently fall through to the
//! catch-all error arm. Extending coverage to any of them is a separate
//! iteration (likely ITER-0005's persistence work surfaces it, or a
//! dedicated ITER-0004x.1):
//!
//! - **Tagged-list-node opcodes:** `MakeListNode`, `ExtractListChild`,
//!   `MatchListNode`, `MatchListNodeWithBindings`, `MatchTag`. These
//!   were renamed in ITER-0004d.2 (see SCENARIO-0107); cross_compile
//!   doesn't model the list-shape constructor semantics yet.
//! - **Pattern matching:** any `Match*` opcode beyond `MatchTag` (which
//!   is itself unsupported).
//! - **Control flow:** `Jump`, `JumpIfFalse`, `Call`, `Return`,
//!   anything that introduces basic blocks beyond a single straight-line
//!   function. This also rules out `&&` / `||` short-circuit operators
//!   (they lower to `JumpIfFalse`) and `if`/`match` expressions.
//! - **Parse-state instructions:** `ParsePush`, `ParseCheckpoint`,
//!   `ParseRestore`, the grammar-engine instruction family.
//! - **Object/method dispatch:** `LoadProp`, `StoreProp`, `CallMethod`,
//!   `DefineObject`, `DefineMethod`.
//! - **Streams + map literals:** `MakeMap`, `CoerceStream`.
//!
//! # Value-equality semantics (parity gate)
//!
//! The parity test maps execution_tape's typed `Value` to fmpl-core's
//! `Value` as follows:
//!
//! | execution_tape | fmpl-core |
//! |---|---|
//! | `Value::I64(n)` | `Value::Int(n)` |
//! | `Value::F64(f)` | `Value::Float(f)` |
//! | `Value::Bool(b)` | `Value::Bool(b)` |
//! | `Value::Str(s)` | `Value::String(s)` (string semantics may diverge subtly — strings excluded from SCENARIO-0109's input subset) |
//! | `Value::Unit` | `Value::Null` |
//!
//! Verified at SCENARIO-0109 (29 cases, all green as of ITER-0004x):
//! Bool / F64 / I64 results round-trip cleanly; Str excluded by
//! input-subset choice. Unit is produced by no current case.
//!
//! See `tests/steps/dual_vm_parity.rs` for the runtime conversion.

use crate::compiler::{CompiledCode, Instruction};
use execution_tape::asm::{Asm, FunctionSig, ProgramBuilder};
use execution_tape::program::{Const, ValueType};
use execution_tape::verifier::VerifiedProgram;
use std::collections::HashMap;

/// Errors during cross-compilation.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CrossCompileError {
    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// Inferred type for a register, used for type-dispatched instruction selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TapeType {
    I64,
    F64,
    Bool,
    Str,
    Unit,
    Unknown,
}

/// Run a forward pass over FMPL instructions to infer types for each instruction index.
/// Single-binding scope: the most recent `Bind { name, ... }` shadows any earlier
/// binding of the same name. Sufficient for SCENARIO-0109's let-binding subset;
/// nested or branching scopes need a real environment stack.
fn infer_types(instructions: &[Instruction]) -> Vec<TapeType> {
    let mut types = vec![TapeType::Unknown; instructions.len()];
    let mut name_to_bind: HashMap<String, usize> = HashMap::new();

    for (idx, instr) in instructions.iter().enumerate() {
        if let Instruction::Bind { name, .. } = instr {
            name_to_bind.insert(name.to_string(), idx);
        }
        types[idx] = match instr {
            Instruction::LoadInt(_) => TapeType::I64,
            Instruction::LoadFloat(_) => TapeType::F64,
            Instruction::LoadBool(_) => TapeType::Bool,
            Instruction::LoadNull => TapeType::Unit,
            Instruction::LoadString(_) | Instruction::LoadSymbol(_) => TapeType::Str,
            Instruction::LoadVar(name) => name_to_bind
                .get(name.as_str())
                .and_then(|&bind_idx| types.get(bind_idx).copied())
                .unwrap_or(TapeType::Unknown),

            // Arithmetic: promote to F64 if either operand is F64
            Instruction::Add { lhs, rhs }
            | Instruction::Sub { lhs, rhs }
            | Instruction::Mul { lhs, rhs }
            | Instruction::Div { lhs, rhs }
            | Instruction::Mod { lhs, rhs } => {
                let lt = types.get(lhs.0).copied().unwrap_or(TapeType::Unknown);
                let rt = types.get(rhs.0).copied().unwrap_or(TapeType::Unknown);
                if lt == TapeType::F64 || rt == TapeType::F64 {
                    TapeType::F64
                } else {
                    TapeType::I64
                }
            }

            // Negation inherits operand type
            Instruction::Neg { operand } => {
                let ot = types.get(operand.0).copied().unwrap_or(TapeType::Unknown);
                if ot == TapeType::F64 {
                    TapeType::F64
                } else {
                    TapeType::I64
                }
            }

            // Not always produces Bool
            Instruction::Not { .. } => TapeType::Bool,

            // Comparisons always produce Bool
            Instruction::Eq { .. }
            | Instruction::NotEq { .. }
            | Instruction::Lt { .. }
            | Instruction::Gt { .. }
            | Instruction::LtEq { .. }
            | Instruction::GtEq { .. } => TapeType::Bool,

            // Bind propagates the value's type
            Instruction::Bind { value, .. } => {
                types.get(value.0).copied().unwrap_or(TapeType::Unknown)
            }

            // NameRef propagates the bind's type
            Instruction::NameRef { bind } => {
                types.get(bind.0).copied().unwrap_or(TapeType::Unknown)
            }

            // Copy propagates the source's type
            Instruction::Copy { source } => {
                types.get(source.0).copied().unwrap_or(TapeType::Unknown)
            }

            _ => TapeType::Unknown,
        };
    }

    types
}

/// Determine the operand type for a binary operation by looking at its operands' inferred types.
fn operand_type(types: &[TapeType], lhs: usize, rhs: usize) -> TapeType {
    let lt = types.get(lhs).copied().unwrap_or(TapeType::Unknown);
    let rt = types.get(rhs).copied().unwrap_or(TapeType::Unknown);
    if lt == TapeType::F64 || rt == TapeType::F64 {
        TapeType::F64
    } else if lt == TapeType::Str || rt == TapeType::Str {
        TapeType::Str
    } else {
        TapeType::I64
    }
}

/// Cross-compile FMPL Indexed RPN IR to execution_tape bytecode.
pub fn cross_compile(code: &CompiledCode) -> Result<VerifiedProgram, CrossCompileError> {
    let types = infer_types(&code.instructions);

    let mut pb = ProgramBuilder::new();
    let mut asm = Asm::new();

    // Map from FMPL instruction index to execution_tape register (u32)
    // Register 0 is reserved for the effect register (used by ret), so data starts at 1
    let mut reg_map: HashMap<usize, u32> = HashMap::new();
    let mut next_reg: u32 = 1;

    // Name → most-recent `Bind` instruction index. Used to resolve
    // `LoadVar(name)` to its bind's register. Single-binding scope only —
    // matches the assumption in `infer_types`.
    let mut name_to_bind: HashMap<String, usize> = HashMap::new();

    // Helper: allocate a register for an instruction index
    let alloc_reg = |idx: usize, reg_map: &mut HashMap<usize, u32>, next_reg: &mut u32| -> u32 {
        *reg_map.entry(idx).or_insert_with(|| {
            let r = *next_reg;
            *next_reg += 1;
            r
        })
    };

    // Helper: allocate a temporary register
    let alloc_tmp = |next_reg: &mut u32| -> u32 {
        let r = *next_reg;
        *next_reg += 1;
        r
    };

    // Helper: get previously-assigned register
    let get_reg = |reg_map: &HashMap<usize, u32>, idx: usize| -> Result<u32, CrossCompileError> {
        reg_map.get(&idx).copied().ok_or_else(|| {
            CrossCompileError::UnsupportedInstruction(format!(
                "Reference to undefined instruction index: {}",
                idx
            ))
        })
    };

    for (idx, instr) in code.instructions.iter().enumerate() {
        let result_reg = alloc_reg(idx, &mut reg_map, &mut next_reg);

        match instr {
            // --- Literals ---
            Instruction::LoadInt(n) => {
                asm.const_i64(result_reg, *n);
            }
            Instruction::LoadFloat(f) => {
                asm.const_f64(result_reg, *f);
            }
            Instruction::LoadBool(b) => {
                asm.const_bool(result_reg, *b);
            }
            Instruction::LoadNull => {
                asm.const_unit(result_reg);
            }
            Instruction::LoadString(s) => {
                let cid = pb.constant(Const::Str(s.to_string()));
                asm.const_pool(result_reg, cid);
            }
            Instruction::LoadSymbol(s) => {
                let cid = pb.constant(Const::Str(s.to_string()));
                asm.const_pool(result_reg, cid);
            }

            // Resolve `LoadVar(name)` through the bind map. Falls back to
            // the historical zero-placeholder ONLY if the name is unknown
            // (free variable) — but on the parity gate this should never
            // happen for the SCENARIO-0109 subset; an unknown LoadVar would
            // surface as a result mismatch.
            Instruction::LoadVar(name) => {
                if let Some(&bind_idx) = name_to_bind.get(name.as_str()) {
                    let bind_reg = get_reg(&reg_map, bind_idx)?;
                    asm.mov(result_reg, bind_reg);
                } else {
                    asm.const_i64(result_reg, 0); // legacy placeholder
                }
            }

            // --- Bind / NameRef ---
            Instruction::Bind { name, value } => {
                let val_reg = get_reg(&reg_map, value.0)?;
                asm.mov(result_reg, val_reg);
                name_to_bind.insert(name.to_string(), idx);
            }
            Instruction::NameRef { bind } => {
                let bind_reg = get_reg(&reg_map, bind.0)?;
                asm.mov(result_reg, bind_reg);
            }

            // --- Binary arithmetic (type-dispatched) ---
            Instruction::Add { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_add(result_reg, l, r);
                    }
                    TapeType::Str => {
                        asm.str_concat(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_add(result_reg, l, r);
                    }
                }
            }
            Instruction::Sub { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_sub(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_sub(result_reg, l, r);
                    }
                }
            }
            Instruction::Mul { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_mul(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_mul(result_reg, l, r);
                    }
                }
            }
            Instruction::Div { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_div(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_div(result_reg, l, r);
                    }
                }
            }
            Instruction::Mod { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_rem(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_rem(result_reg, l, r);
                    }
                }
            }

            // --- Comparisons (type-dispatched) ---
            Instruction::Eq { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_eq(result_reg, l, r);
                    }
                    TapeType::Str => {
                        asm.str_eq(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_eq(result_reg, l, r);
                    }
                }
            }
            Instruction::NotEq { lhs, rhs } => {
                // No ne opcode — emit eq then bool_not
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                let tmp = alloc_tmp(&mut next_reg);
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_eq(tmp, l, r);
                    }
                    TapeType::Str => {
                        asm.str_eq(tmp, l, r);
                    }
                    _ => {
                        asm.i64_eq(tmp, l, r);
                    }
                }
                asm.bool_not(result_reg, tmp);
            }
            Instruction::Lt { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_lt(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_lt(result_reg, l, r);
                    }
                }
            }
            Instruction::Gt { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_gt(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_gt(result_reg, l, r);
                    }
                }
            }
            Instruction::LtEq { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_le(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_le(result_reg, l, r);
                    }
                }
            }
            Instruction::GtEq { lhs, rhs } => {
                let l = get_reg(&reg_map, lhs.0)?;
                let r = get_reg(&reg_map, rhs.0)?;
                match operand_type(&types, lhs.0, rhs.0) {
                    TapeType::F64 => {
                        asm.f64_ge(result_reg, l, r);
                    }
                    _ => {
                        asm.i64_ge(result_reg, l, r);
                    }
                }
            }

            // --- Unary ---
            Instruction::Neg { operand } => {
                let op = get_reg(&reg_map, operand.0)?;
                let ot = types.get(operand.0).copied().unwrap_or(TapeType::Unknown);
                if ot == TapeType::F64 {
                    asm.f64_neg(result_reg, op);
                } else {
                    // No i64_neg — emit: tmp = 0; result = 0 - op
                    let tmp = alloc_tmp(&mut next_reg);
                    asm.const_i64(tmp, 0);
                    asm.i64_sub(result_reg, tmp, op);
                }
            }
            Instruction::Not { operand } => {
                let op = get_reg(&reg_map, operand.0)?;
                asm.bool_not(result_reg, op);
            }

            // --- Data structures ---
            Instruction::MakeList { elements } => {
                let elem_regs: Vec<u32> = elements
                    .iter()
                    .map(|e| get_reg(&reg_map, e.0))
                    .collect::<Result<_, _>>()?;
                asm.tuple_new(result_reg, &elem_regs);
            }

            // PushScope/PopScope are scope-chain fixtures for the in-tree Vm's
            // dynamic environment. The cross-compile target uses a flat
            // register file (Indexed RPN) — `Bind` writes directly to the
            // result register, and `NameRef` reads from it. No scope state
            // exists to push or pop, so these instructions are no-ops here.
            // The result_reg is allocated but never written; it is also
            // skipped by the return-value selector below.
            Instruction::PushScope | Instruction::PopScope => {
                // No-op.
            }

            // Copy is a register move used at scope-exit and control-flow
            // convergence points to plumb a value out of an inner block.
            // Maps directly to `mov` in execution_tape.
            Instruction::Copy { source } => {
                let src_reg = get_reg(&reg_map, source.0)?;
                asm.mov(result_reg, src_reg);
            }

            // --- Unsupported ---
            _ => {
                return Err(CrossCompileError::UnsupportedInstruction(format!(
                    "{:?}",
                    instr
                )));
            }
        }
    }

    // Select the function's return value + type. Scan backwards for the
    // last value-producing instruction (skipping PushScope/PopScope, which
    // are no-ops in this target and don't write a register). This is also
    // how `ret_types` is inferred — the cross-compiler is single-return and
    // its caller (SCENARIO-0109's `dual_vm_parity` step-def) reads result[0].
    let last_value_idx = (0..code.instructions.len()).rev().find(|&i| {
        !matches!(
            code.instructions[i],
            Instruction::PushScope | Instruction::PopScope
        )
    });

    let (ret_reg, ret_type) = match last_value_idx {
        Some(idx) => {
            let reg = reg_map.get(&idx).copied().unwrap_or(0);
            let ty = match types.get(idx).copied().unwrap_or(TapeType::Unknown) {
                TapeType::I64 => ValueType::I64,
                TapeType::F64 => ValueType::F64,
                TapeType::Bool => ValueType::Bool,
                TapeType::Str => ValueType::Str,
                // Unit / Unknown: fall back to I64 for now. Unknown means
                // type inference didn't track the instruction (e.g., LoadVar
                // without binding context). The dual_vm_parity gate will
                // catch a mismatch loudly if the assumption is wrong.
                TapeType::Unit | TapeType::Unknown => ValueType::I64,
            };
            (reg, ty)
        }
        None => (0, ValueType::I64),
    };
    asm.ret(0, &[ret_reg]);

    pb.push_function_checked(
        asm,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ret_type],
        },
    )
    .map_err(|e| CrossCompileError::VerificationFailed(e.to_string()))?;

    pb.build_verified()
        .map_err(|e| CrossCompileError::VerificationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Compiler;

    #[test]
    fn test_cross_compile_arithmetic() {
        // Compile: 7 + 9 * 5
        let source = "7 + 9 * 5";
        let tokens = crate::lexer::Lexer::new(source).tokenize().unwrap();
        let ast = crate::parser::Parser::with_source(&tokens, source)
            .parse()
            .unwrap();
        let fmpl_code = Compiler::new().compile(&ast).unwrap();

        let exec_program = cross_compile(&fmpl_code).unwrap();
        assert!(!exec_program.program().functions.is_empty());
    }

    #[test]
    fn test_cross_compile_simple() {
        // Compile: 1 + 2
        let source = "1 + 2";
        let tokens = crate::lexer::Lexer::new(source).tokenize().unwrap();
        let ast = crate::parser::Parser::with_source(&tokens, source)
            .parse()
            .unwrap();
        let fmpl_code = Compiler::new().compile(&ast).unwrap();

        let exec_program = cross_compile(&fmpl_code).unwrap();
        assert!(!exec_program.program().functions.is_empty());
    }
}
