# Spec: Remove :Tagged() from Grammars

## Status
**Active** - Ready to implement

## Context
We're migrating from `:Tagged(name, args...)` syntax to the new list syntax `[:Tag, ...args]` in FMPL grammars. This aligns tagged values with how lists and other data structures work, and provides a more consistent syntax.

## Problem Statement
The current `:Tagged(name, args...)` syntax is inconsistent with the new list-based approach used throughout the language. Pattern matching and data structures use `[:Tag, ...]` format, and tagged values should follow suit.

## Solution

### Old Syntax
```fmpl
:Tagged("MyTag", [:Value(1), :Value(2)])
:Tagged("Seq", [:Value(1), :Value(2)])
:Tagged("MyTag", [])
```

### New Syntax
```fmpl
[:MyTag, :Value(1), :Value(2)]
[:Seq, :Value(1), :Value(2)]
[:MyTag, []]
```

## Implementation Steps

### Step 1: Update fmpl_parser.fmpl Tagged Values
**File**: `lib/core/fmpl_parser.fmpl`

Current implementation (lines 76-80):
```fmpl
tagged_with_args = ":" tag_name:tag "(" sp tagged_args:items ")" sp => :Tagged(tag, items)
tagged_empty = ":" tag_name:tag "(" sp ")" sp => :Tagged(tag, [])
tagged = tagged_with_args | tagged_empty
```

Replace with:
```fmpl
tagged_with_args = ":" tag_name:tag "(" sp tagged_args:items ")" sp => [tag, items]
tagged_empty = ":" tag_name:tag "(" sp ")" sp => [tag, []]
tagged = tagged_with_args | tagged_empty
```

**Note**: This is the first change. The generated parser will now parse `:Tag(args)` into list syntax `[tag, args]`, which means the AST will receive `Value::List` with a symbol as first element, not `Expr::Tagged`.

### Step 2: Update Rust Parser for New Syntax
**File**: `fmpl-core/src/parser.rs`

Update the Rust parser to produce the correct AST from list syntax tagged values.

Current implementation (around line 608):
```rust
Ok(Expr::Tagged(s, args))
```

Need to add handling for:
- Parse `:tag(args)` as a list `[tag, args...]`
- Distinguish from regular lists
- Produce appropriate AST representation

**Options**:
1. Keep `Expr::Tagged` for the AST
2. Create new `Expr::TaggedList` variant
3. Use `Expr::List` with special handling

### Step 3: Update Compiler for List-Based Tagged Values
**File**: `fmpl-core/src/compiler.rs`

Update compiler to handle the new AST representation from step 2.

Current implementation (around line 536):
```rust
Expr::Tagged(tag, args) => {
    let mut arg_indices = Vec::with_capacity(args.len());
    for arg in args {
        arg_indices.push(self.compile_expr(arg)?);
    }
    Ok(self.code.emit(Instruction::MakeTagged { tag, arg_indices }))
}
```

### Step 4: Update AST Compiler for List Syntax
**File**: `fmpl-core/src/ast.rs`

Consider if we need new AST node type or can use existing `Expr::List`.

### Step 5: Check Builtins for :Tagged References
**Files**:
- `fmpl-core/src/builtins/grammar_to_ir.rs`
- `fmpl-core/src/builtins/ir_to_rust.rs`

Search for any `:Tagged` value references and update to work with list syntax.

### Step 4: Verify Optimizer Grammars
**File**: `lib/core/grammar_optimizer.fmpl`

Verify all optimizer grammars use the new list syntax (they should already).

### Step 5: Search and Update All :Tagged Usage
Search entire codebase for `:Tagged(` pattern and update any remaining references.

### Step 6: Update Tests
Update any tests using the old `:Tagged(...)` syntax.

### Step 7: Update Documentation
Update any docs referencing the old syntax.

## Verification

### 1. Grammar Parsing
```bash
cargo build -p fmpl-bootstrap
```
Expected: Parser generation succeeds

### 2. Parser Tests
```bash
cargo test -p fmpl-core --test generated_parser_correctness
```
Expected: All 57 tests pass

### 3. Full Test Suite
```bash
cargo test -p fmpl-core
```
Expected: All tests pass

## Success Criteria

1. ✅ All `:Tagged(...)` references in grammars replaced with list syntax `[:Tag, ...]`
2. ✅ Parser generation succeeds without errors
3. ✅ All generated parser tests pass (57 tests)
4. ✅ Full test suite passes
5. ✅ No unintended `:Tagged` references remain

## Notes

- The new list syntax is already used in `grammar_optimizer.fmpl`
- Pattern matching uses `[:Pattern, ...]` syntax consistently
- This change aligns tagged values with how lists and maps work
- May need to handle both formats temporarily during transition

## Related

- Plan: `/Users/ndn/development/fmpl/docs/plans/2025-01-31-remove-tagged-from-grammars.md`
- Grammar optimizer: `lib/core/grammar_optimizer.fmpl`
