# Plan: Remove :Tagged() from Grammars

## Goal
Migrate from `:Tagged(name, args...)` syntax to the new list syntax `[:Tag, ...args]` in FMPL grammars.

## Current State

### Files with :Tagged() references:
1. **lib/core/fmpl_parser.fmpl** (lines 78-80)
   - `tagged_with_args = ":" tag_name:tag "(" sp tagged_args:items ")" sp => :Tagged(tag, items)`
   - `tagged_empty = ":" tag_name:tag "(" sp ")" sp => :Tagged(tag, [])`
   - `tagged = tagged_with_args | tagged_empty`
   - Used in `primary` rule

### New List Syntax Reference (from grammar_optimizer.fmpl):
- `[:Seq, ...]` - Sequence
- `[:Choice, ...]` - Choice
- `[:Char, ...]` - Character
- `[:Literal, ...]` - Literal
- `[:Star, ...]` - Star repetition
- `[:Bind, ...]` - Bind
- `[:Action, ...]` - Action
- `:Empty` - Empty pattern
- `_:x` - Wildcard (passthrough)

## Migration Strategy

### Step 1: Update fmpl_parser.fmpl Tagged Values
**File**: `lib/core/fmpl_parser.fmpl`

Replace:
```fmpl
tagged_with_args = ":" tag_name:tag "(" sp tagged_args:items ")" sp => :Tagged(tag, items)
tagged_empty = ":" tag_name:tag "(" sp ")" sp => :Tagged(tag, [])
tagged = tagged_with_args | tagged_empty
```

With:
```fmpl
tagged_with_args = ":" tag_name:tag "(" sp tagged_args:items ")" sp => [tag, items]
tagged_empty = ":" tag_name:tag "(" sp ")" sp => [tag, []]
tagged = tagged_with_args | tagged_empty
```

### Step 2: Update AST Compiler (if needed)
**File**: `fmpl-core/src/ast.rs`

Check if `Expr::Tagged` needs updating or if it can handle both formats. The compiler may need to handle the new list syntax.

### Step 3: Update Builtins (if needed)
**Files**:
- `fmpl-core/src/builtins/grammar_to_ir.rs`
- `fmpl-core/src/builtins/ir_to_rust.rs`

These may reference `:Tagged` values and need updating to work with list syntax.

### Step 4: Update Optimizer Grammars
**File**: `lib/core/grammar_optimizer.fmpl`

Verify all optimizer grammars use the new list syntax (they should already).

### Step 5: Update Tests
**Files**: All test files using `:Tagged(...)` syntax

### Step 6: Update Documentation
**Files**: Any docs referencing the old `:Tagged()` syntax

## Verification

### Phase 1: Grammar Parsing
```bash
# Ensure grammar parses correctly
cargo build -p fmpl-bootstrap
```

### Phase 2: Parser Tests
```bash
# Run all parser tests
cargo test -p fmpl-core --test generated_parser_correctness
```

### Phase 3: Full Test Suite
```bash
# Ensure all tests pass
cargo test -p fmpl-core
```

## Success Criteria

1. All `:Tagged(...)` references replaced with list syntax `[:Tag, ...]`
2. Parser generation succeeds without errors
3. All generated parser tests pass
4. Full test suite passes
5. No references to old `Expr::Tagged` pattern (or it handles both formats transparently)

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| AST compiler expects `:Tagged` format | May need to add handling for list format in compiler |
| Semantic actions break | Update all `=> :Tagged(...)` actions to use list syntax |
| Tests fail | Update tests to use new syntax |
| Inconsistent migration | Do comprehensive search for all `:Tagged` usage |

## Notes

- The new list syntax is more consistent with the rest of the language
- Pattern matching already uses `[:Pattern, ...]` syntax in many places
- This change aligns tagged values with how lists and other data structures work
