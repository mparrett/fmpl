# AST to IR Parity Tests

**Status:** Complete (Phase 2 - Bootstrap)
**Date:** 2026-03-03
**Issue:** #6400ea8 - Bootstrap Phase 2: ast_to_ir.fmpl parity tests

## Overview

Parity tests verify that the FMPL compiler pipeline produces identical results to the Rust compiler for all test cases.

## Test Structure

The test file `fmpl-core/tests/ast_to_ir_parity.rs` contains two test categories:

1. **IR Compilation Tests** - Test `ir::compile()` builtin in isolation
2. **Full Pipeline Tests** - Test complete FMPL pipeline (parse → transform → compile → execute)

## Test Categories

### IR Compilation Tests

Tests that verify the `ir::compile()` Rust builtin correctly compiles IR tagged values to bytecode.

| Module | Tests | Status |
|--------|-------|--------|
| `literals` | integer, bool_true, bool_false, null_value, string_literal | ✅ Pass |
| `arithmetic` | addition, subtraction, multiplication, division, modulo, negation | ✅ Pass |
| `comparisons` | equality, inequality, less_than, greater_than, less_than_equal, greater_than_equal | ✅ Pass |
| `logical` | and_operator, or_operator, not_operator | ✅ Pass |
| `control_flow` | if_true, if_false | ✅ Pass |
| `let_bindings` | simple_let, let_with_arithmetic | ✅ Pass |
| `data_structures` | empty_list, list_of_ints, empty_map, map_literal | ✅ Pass |
| `functions` | lambda_call | ✅ Pass |

### Full Pipeline Tests

Tests that verify the complete FMPL compilation pipeline:
1. `ast::parse(source)` → AST tagged values
2. `ast @ ast_to_ir.expr` → IR tagged values
3. `ir::compile(ir)` → CompiledCode
4. `code::eval(code)` → result

| Test | Source | Result |
|------|--------|--------|
| `parity_integer` | `42` | ✅ Pass |
| `parity_arithmetic` | `1 + 2 * 3` | ✅ Pass |
| `parity_string` | `"hello"` | ✅ Pass |
| `parity_let_binding` | `let (x = 42) x + 1` | ✅ Pass |
| `parity_if_expr` | `if true then 1 else 2` | ✅ Pass |
| `parity_lambda` | `let (f = \x x + 1) f(41)` | ✅ Pass |
| `parity_list` | `[1, 2, 3]` | ✅ Pass |
| `parity_map` | `%{a: 1, b: 2}` | ✅ Pass |

## Summary

- **Total Tests:** 37
- **Passing:** 37
- **Failing:** 0
- **Coverage:** Basic language features (literals, arithmetic, control flow, functions, data structures)

## Next Steps

Per issue #7d37a85, expand coverage to include:
- Loops (while, for, do/while)
- Try/catch exception handling
- Pattern matching (match expressions)
- Objects (spawn, facets, bcom)
- Grammars (grammar definitions, rule application)
- Async operations (async calls, streams)
- Method calls and property access
- Pipe operator

## Files

- `fmpl-core/tests/ast_to_ir_parity.rs` - Test implementation
- `lib/core/ast_to_ir.fmpl` - FMPL tree grammar for AST → IR
- `lib/core/prelude.fmpl` - Helper functions
- `fmpl-core/src/builtins/ir.rs` - `ir::compile()` builtin
- `fmpl-core/src/builtins/ast.rs` - `ast::parse()` builtin
