//! Cross-compiler from FMPL Indexed RPN IR to execution_tape bytecode.
//!
//! This module provides a direct compilation path from FMPL's IR to
//! execution_tape's verified bytecode, enabling performance comparison.

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
fn infer_types(instructions: &[Instruction]) -> Vec<TapeType> {
    let mut types = vec![TapeType::Unknown; instructions.len()];

    for (idx, instr) in instructions.iter().enumerate() {
        types[idx] = match instr {
            Instruction::LoadInt(_) => TapeType::I64,
            Instruction::LoadFloat(_) => TapeType::F64,
            Instruction::LoadBool(_) => TapeType::Bool,
            Instruction::LoadNull => TapeType::Unit,
            Instruction::LoadString(_) | Instruction::LoadSymbol(_) => TapeType::Str,
            Instruction::LoadVar(_) => TapeType::Unknown,

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

            // --- Variables (simplified) ---
            Instruction::LoadVar(_name) => {
                asm.const_i64(result_reg, 0); // Placeholder
            }

            // --- Bind / NameRef ---
            Instruction::Bind { value, .. } => {
                let val_reg = get_reg(&reg_map, value.0)?;
                asm.mov(result_reg, val_reg);
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

            // --- Unsupported ---
            _ => {
                return Err(CrossCompileError::UnsupportedInstruction(format!(
                    "{:?}",
                    instr
                )));
            }
        }
    }

    // Emit return with the last instruction's result
    if let Some(&last_reg) = reg_map.get(&(code.instructions.len() - 1)) {
        asm.ret(0, &[last_reg]);
    } else {
        asm.ret(0, &[0]);
    }

    pb.push_function_checked(
        asm,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ValueType::I64],
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
