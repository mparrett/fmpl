//! Cross-compiler from FMPL Indexed RPN IR to execution_tape bytecode.
//!
//! This module provides a direct compilation path from FMPL's IR to
//! execution_tape's verified bytecode, enabling performance comparison.

use crate::compiler::{CompiledCode, InstrIndex, Instruction};
use execution_tape::asm::{Asm, FunctionSig};
use execution_tape::program::{ProgramBuilder, ValueType};
use execution_tape::verifier::VerifiedProgram;
use smol_str::SmolStr;
use std::collections::HashMap;

/// Errors during cross-compilation.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CrossCompileError {
    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// Cross-compile FMPL Indexed RPN IR to execution_tape bytecode.
///
/// This performs a direct translation from FMPL's instruction format
/// to execution_tape's register-based format.
pub fn cross_compile(code: &CompiledCode) -> Result<VerifiedProgram, CrossCompileError> {
    let mut asm = Asm::new();
    let mut reg_count = 0;

    // Map from FMPL InstrIndex to execution_tape register number
    let mut reg_map: HashMap<usize, usize> = HashMap::new();

    // First pass: assign register numbers and emit instructions
    for (idx, instr) in code.instructions.iter().enumerate() {
        let instr_idx = idx;
        cross_compile_instruction(&mut asm, instr, instr_idx, &mut reg_map, &mut reg_count)?;
    }

    // Emit return with the last instruction's result
    if let Some(&last_reg) = reg_map.get(&(code.instructions.len() - 1)) {
        asm.ret(0, &[last_reg]);
    } else {
        asm.ret(0, &[0]); // Return r0 if no instructions
    }

    // Build and verify the program
    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        asm,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ValueType::I64],
            reg_count: reg_count,
        },
    )
    .map_err(|e| CrossCompileError::VerificationFailed(e.to_string()))?;

    pb.build_verified()
        .map_err(|e| CrossCompileError::VerificationFailed(e.to_string()))
}

/// Cross-compile a single instruction.
fn cross_compile_instruction(
    asm: &mut Asm,
    instr: &Instruction,
    idx: usize,
    reg_map: &mut HashMap<usize, usize>,
    reg_count: &mut usize,
) -> Result<(), CrossCompileError> {
    // Assign/register for this instruction's result
    let result_reg = reg_map.entry(idx).or_insert_with(|| {
        let r = *reg_count;
        *reg_count += 1;
        r
    });

    match instr {
        // Literals
        Instruction::LoadInt(n) => {
            asm.const_i64(*result_reg, *n);
        }
        Instruction::LoadFloat(f) => {
            asm.const_f64(*result_reg, *f);
        }
        Instruction::LoadBool(b) => {
            asm.const_bool(*result_reg, *b);
        }
        Instruction::LoadNull => {
            asm.const_unit(*result_reg);
        }
        Instruction::LoadString(s) => {
            asm.const_string(*result_reg, s.as_str());
        }
        Instruction::LoadSymbol(s) => {
            asm.const_string(*result_reg, s.as_str());
        }

        // Variables (simplified - assume predefined)
        Instruction::LoadVar(name) => {
            // For now, load from a predefined environment slot
            // In a real implementation, we'd need to handle variable binding
            asm.const_i64(*result_reg, 0); // Placeholder
        }

        // Binary arithmetic
        Instruction::Add { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_add(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Sub { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_sub(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Mul { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_mul(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Div { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_div(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Mod { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_rem(*result_reg, lhs_reg, rhs_reg);
        }

        // Comparisons
        Instruction::Eq { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.eq(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Lt { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_lt(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::Gt { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_gt(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::LtEq { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_le(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::GtEq { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.i64_ge(*result_reg, lhs_reg, rhs_reg);
        }
        Instruction::NotEq { lhs, rhs } => {
            let lhs_reg = get_reg(reg_map, lhs.0)?;
            let rhs_reg = get_reg(reg_map, rhs.0)?;
            asm.ne(*result_reg, lhs_reg, rhs_reg);
        }

        // Unary
        Instruction::Neg { operand } => {
            let op_reg = get_reg(reg_map, operand.0)?;
            asm.i64_neg(*result_reg, op_reg);
        }
        Instruction::Not { operand } => {
            let op_reg = get_reg(reg_map, operand.0)?;
            asm.not(*result_reg, op_reg);
        }

        // Data structures (simplified)
        Instruction::MakeList { elements } => {
            asm.list_new(*result_reg);
            // TODO: Append elements
        }

        // Unsupported instructions
        _ => {
            return Err(CrossCompileError::UnsupportedInstruction(format!(
                "{:?}",
                instr
            )));
        }
    }

    Ok(())
}

/// Get register number for an instruction index.
fn get_reg(reg_map: &HashMap<usize, usize>, idx: usize) -> Result<usize, CrossCompileError> {
    reg_map.get(&idx).copied().ok_or_else(|| {
        CrossCompileError::UnsupportedInstruction(format!(
            "Reference to undefined instruction index: {}",
            idx
        ))
    })
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

        // Cross-compile to execution_tape
        let exec_program = cross_compile(&fmpl_code).unwrap();

        // Verify it compiles
        assert!(exec_program.function_count() > 0);
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

        // Cross-compile to execution_tape
        let exec_program = cross_compile(&fmpl_code).unwrap();

        // Verify it compiles
        assert!(exec_program.function_count() > 0);
    }
}
