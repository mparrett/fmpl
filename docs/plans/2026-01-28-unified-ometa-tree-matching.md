# Unified OMeta-Style Tree Matching Plan

**Status:** ✅ COMPLETED (2026-01-28)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement full OMeta-style parsing that works uniformly on text AND tree structures (lists, maps, AST nodes).

**Architecture:** Lower grammar combinators to base IR (loops + jumps) and add an input stack for tree descent. The same primitives work on any stream type.

**Tech Stack:** Rust, FMPL VM, Compiler

**Supersedes:** `2026-01-27-ometa-lowering-to-ir.md` (expanded scope)

## Implementation Summary

**Completed:**
- ✅ Phase 1: Input Stack Infrastructure - `ParseState` with input stack for tree descent
- ✅ Phase 2: Polymorphic MatchAny - Returns Null on failure instead of error
- ✅ Phase 3-5: Star/Plus/Choice lowered to base IR (loops + jumps)
- ✅ Phase 6: Semantic Predicates - `GP::Predicate` compiles to JumpIfFalse
- ✅ Phase 7-8: List/Map Pattern Syntax - Working with MatchList/MatchMap instructions
- ✅ Phase 9: Removed MatchStar, MatchPlus, MatchChoice generic instructions
- ✅ Phase 10: Documentation updated in `specs/grammar-system.md`

**Code changes:**
- `vm.rs`: ~330 lines removed (net), added new instructions
- `compiler.rs`: ~170 lines added for lowering logic
- `parse_state.rs`: Refactored for input stack model

**New instructions added:**
- `ParsePush`, `ParsePop`, `ParsePosition` - Tree descent
- `ListAppend` - Collecting Star/Plus results
- `IsList`, `IsMap`, `IsString` - Type checking for tree matching

**Instructions removed:**
- `MatchStar { pattern }` - Now lowered to IR loop
- `MatchPlus { pattern }` - Now lowered to IR loop
- `MatchChoice { cases }` - Now lowered to IR with checkpoint/restore

---

## Background

### What OMeta Does

OMeta grammars operate on **any stream** - characters, tokens, or AST nodes. The same combinators (Star, Plus, Choice, Seq) work uniformly:

```fmpl
grammar simplify {
  // Tree transformation - operates on nested lists
  expr = ['add' expr:x expr:y] ?(x == 0) -> y      // 0 + y = y
       | ['add' expr:x expr:y] ?(y == 0) -> x      // x + 0 = x
       | ['mul' expr:x expr:y] ?(x == 1) -> y      // 1 * y = y
       | ['mul' expr:x expr:y] ?(y == 1) -> x      // x * 1 = x
       | ['add' expr:x expr:y] -> ['add', x, y]    // recurse children
       | ['mul' expr:x expr:y] -> ['mul', x, y]    // recurse children
       | any:x -> x                                 // base case (leaf)
}

// Use it
['add', 0, ['mul', 1, 42]] @ simplify.expr  // => 42
```

The `['add' expr:x expr:y]` pattern:
1. Match current stream element (must be a list)
2. **Descend** into the list as a new stream
3. Match literal 'add', then recursively apply `expr` twice
4. **Ascend** back to outer stream, advance position

### Current State

**Text parsing** has specialized instructions:
- `MatchStar`, `MatchPlus`, `MatchChoice` (~400 lines in vm.rs)
- Only work on characters, limited pattern support
- `MatchChoice` is broken (placeholder implementation)

**Tree matching** has separate instructions:
- `MatchList`, `MatchMap`, `MatchMapNested`
- Structural destructuring only, no recursion/grammar rules
- Can't express `Node*` (repetition on tree elements)
- Can't express `Leaf | Branch` (backtracking choice on trees)

**Problem:** Two separate codepaths, neither complete.

### The Solution

**Unified model:**
1. Lower combinators (Star/Plus/Choice) to loops using existing primitives
2. Add input stack for descending into nested structures
3. Same lowered code works on text AND trees

---

## Architecture

### Input Stack Model

```rust
struct ParseState {
    // Stack of (input_value, position) for tree descent
    input_stack: Vec<(Value, usize)>,
    // Memoization keyed by (stack_depth, position, rule_name)
    memo: HashMap<MemoKey, MemoEntry>,
}
```

- **Text parsing:** `input_stack = [("hello world", 5)]`
- **Tree parsing:** `input_stack = [(outer_list, 2), (inner_list, 0)]`

Descending into a list pushes a new frame. Ascending pops back.

### Primitive Instructions (Minimal Set)

**Stream Management:**
| Instruction | Purpose |
|-------------|---------|
| `ParseCheckpoint` | Save (stack_depth, position) for backtracking |
| `ParseRestore { checkpoint }` | Restore to checkpoint |
| `ParsePush { value }` | Push value as new input stream (position 0) |
| `ParsePop` | Pop to previous input stream |
| `ParsePosition` | Get current position as Int (for zero-length guard) |

**Primitive Matchers (polymorphic - work on text or values):**
| Instruction | Purpose |
|-------------|---------|
| `MatchAny` | Consume one item, return it (char or value) |
| `MatchLiteral { value }` | Match exact value, advance |
| `MatchCharClass { ranges }` | Match char in ranges (text only) |
| `MatchEnd` | Match end of current stream |
| `MatchPredicate { expr }` | Match if expr evaluates truthy |

**Type Tests:**
| Instruction | Purpose |
|-------------|---------|
| `IsNull { value }` | Check if value is null (pattern failed) |
| `IsList { value }` | Check if value is a list |
| `IsMap { value }` | Check if value is a map |
| `IsString { value }` | Check if value is a string |

### Lowered Combinators

These are NOT instructions - the compiler generates them as control flow:

**Star(pattern):**
```
results = MakeList []
loop_start:
  checkpoint = ParseCheckpoint
  start_pos = ParsePosition
  result = <compile pattern>
  JumpIfNull result, loop_end
  results = ListAppend results, result
  end_pos = ParsePosition
  JumpIfEqual start_pos, end_pos, loop_end  // zero-length guard
  Jump loop_start
loop_end:
  ParseRestore checkpoint  // restore position to after last success
  Return results
```

**Plus(pattern):**
```
first = <compile pattern>
JumpIfNull first, fail
results = MakeList [first]
// ... same loop as Star ...
```

**Choice([p1, p2, p3]):**
```
  checkpoint = ParseCheckpoint
  r1 = <compile p1>
  JumpIfNotNull r1, done
  ParseRestore checkpoint

  r2 = <compile p2>
  JumpIfNotNull r2, done
  ParseRestore checkpoint

  r3 = <compile p3>
  JumpIfNotNull r3, done
  ParseRestore checkpoint

  Return Null  // all failed
done:
  Return <result>
```

**Seq([p1, p2, p3]):**
```
  r1 = <compile p1>
  JumpIfNull r1, fail
  r2 = <compile p2>
  JumpIfNull r2, fail
  r3 = <compile p3>
  JumpIfNull r3, fail
  Return r3  // or collect results
fail:
  Return Null
```

### List Pattern Compilation

`['add' expr:x expr:y]` compiles to:

```
  current = MatchAny                    // get element from stream
  is_list = IsList current
  JumpIfFalse is_list, fail
  ParsePush current                     // descend into list

  tag = MatchAny                        // first element
  cmp = Equal tag, "add"
  JumpIfFalse cmp, fail_pop

  x = ApplyRule "expr"                  // recursive descent
  JumpIfNull x, fail_pop

  y = ApplyRule "expr"
  JumpIfNull y, fail_pop

  at_end = MatchEnd                     // must consume entire list
  JumpIfNull at_end, fail_pop

  ParsePop                              // ascend
  // ... continue with semantic predicate/action ...

fail_pop:
  ParsePop
fail:
  Return Null
```

---

## Implementation Plan

### Phase 1: Input Stack Infrastructure (2-3 hours)

**Modify `ParseState`:**
```rust
pub struct ParseState {
    input_stack: Vec<InputFrame>,
    memo: HashMap<MemoKey, MemoEntry>,
}

struct InputFrame {
    value: Value,
    position: usize,
}

struct MemoKey {
    depth: usize,      // input_stack.len()
    position: usize,
    rule: SmolStr,
}
```

**Add instructions:**
- `ParsePush { value: InstrIndex }` - push value as new input
- `ParsePop` - pop to previous input
- `ParsePosition` - return current position as Int

**Update existing methods:**
- `position()` → reads from `input_stack.last()`
- `advance(n)` → modifies `input_stack.last_mut()`
- `head_char()` / `head_value()` → read from current frame
- `checkpoint()` → captures `(stack.len(), position)`
- `restore()` → truncates stack and sets position

**Acceptance:**
- Can push/pop input frames
- Position tracking works per-frame
- Existing text parsing still works

---

### Phase 2: Polymorphic MatchAny (1-2 hours)

**Update `MatchAny` instruction:**
```rust
Instruction::MatchAny => {
    let frame = self.frames.last_mut().unwrap();

    // Try as text first
    if let Some(ch) = frame.parse_state.head_char() {
        frame.parse_state.advance(ch.len_utf8());
        frame.set_current(Value::String(SmolStr::new(ch.to_string())));
    }
    // Try as value stream
    else if let Some(val) = frame.parse_state.head_value() {
        frame.parse_state.advance(1);
        frame.set_current(val);
    }
    // At end
    else {
        frame.set_current(Value::Null);
    }
}
```

**Acceptance:**
- `MatchAny` returns char from string input
- `MatchAny` returns element from list input
- Returns Null at end of stream

---

### Phase 3: Lower Star to IR (2-3 hours)

**Update `compile_grammar_pattern` for `GP::Star(p)`:**

Instead of emitting `MatchStar { pattern }`, emit:
1. Empty list creation
2. Loop with checkpoint
3. Pattern compilation
4. Append to list
5. Zero-length guard
6. Jump back / break

**Helper instructions needed:**
- `ListAppend { list: InstrIndex, item: InstrIndex }` → returns new list
- Or use mutable list building

**Acceptance:**
- `digit*` compiles to loop bytecode
- Correctly matches zero or more
- Zero-length patterns don't infinite loop
- Works on text input

---

### Phase 4: Lower Plus to IR (1-2 hours)

Same as Star but:
- Require at least one match before entering loop
- Return Null if first match fails

**Acceptance:**
- `digit+` requires at least one digit
- Returns Null on zero matches

---

### Phase 5: Lower Choice to IR (2-3 hours)

**Update `compile_grammar_pattern` for `GP::Choice(patterns)`:**

Emit sequential try/restore chain:
1. For each pattern:
   - Checkpoint
   - Compile pattern
   - If success, jump to done
   - Restore checkpoint
2. Return Null (all failed)

**Acceptance:**
- `'a' | 'b' | 'c'` tries each in order
- Backtracking works correctly
- First match wins

---

### Phase 6: Semantic Predicates (2-3 hours)

**Add grammar syntax:** `?(expr)` or `?(expr)` after pattern

**Add instruction:**
```rust
MatchPredicate { expr: InstrIndex }  // fail if expr is falsy
```

**Or lower to:**
```
  result = <compile expr>
  JumpIfFalse result, fail
```

**Acceptance:**
- `digit:d ?(d > 5)` only matches digits > 5
- Predicate failure triggers backtracking

---

### Phase 7: List Pattern Syntax (3-4 hours)

**Add grammar syntax:** `[p1 p2 p3]` or `[p1 p2 ...rest]`

**Parser changes:**
- Recognize `[` in grammar pattern context
- Parse element patterns
- Handle `...rest` spread syntax

**Compiler changes:**
- Emit `MatchAny` + `IsList` check
- Emit `ParsePush`
- Compile element patterns as sequence
- Emit `MatchEnd` (unless `...rest`)
- Emit `ParsePop`

**Acceptance:**
- `['add' any:x any:y]` matches list starting with 'add'
- Bindings (`x`, `y`) are captured
- `...rest` captures remaining elements

---

### Phase 8: Map Pattern Syntax (2-3 hours)

**Add grammar syntax:** `%{key: pattern, ...}` in grammars

**Implementation:**
- Check value is map
- For each key, extract value and match against pattern
- Bind captured variables

**Acceptance:**
- `%{type: 'node', children: cs}` matches maps with those keys
- Values matched against nested patterns

---

### Phase 9: Remove Old Instructions (2-3 hours)

**Delete from Instruction enum:**
- `MatchStar`, `MatchPlus`, `MatchChoice`
- `MatchStarChar`, `MatchPlusChar`, `MatchStarCharClass`, `MatchPlusCharClass`
- `MatchStarLiteral`, `MatchPlusLiteral`
- `MatchStarRule`, `MatchPlusRule`
- `MatchList` (old form), `MatchListWithBindings`
- `MatchMap` (old form), `MatchMapNested`

**Delete from VM:**
- All handlers for above (~400+ lines)

**Acceptance:**
- Code compiles
- All tests pass
- VM significantly smaller

---

### Phase 10: Documentation and Cleanup (2-3 hours)

1. Update `specs/grammar-system.md` with unified model
2. Add tree transformation examples
3. Document list/map pattern syntax
4. Remove dead code
5. Update CLAUDE.md if needed

**Acceptance:**
- Docs reflect new architecture
- Examples work

---

## Success Criteria

1. ✅ All existing tests pass
2. ✅ Tree transformation grammar works:
   ```fmpl
   ['add', 0, x] @ simplify.expr  // => x
   ```
3. ✅ Repetition on tree elements works:
   ```fmpl
   [node*] @ grammar.rule  // match zero or more nodes
   ```
4. ✅ Choice backtracks on tree patterns:
   ```fmpl
   (Leaf(x) | Branch(l, r)) @ grammar.rule
   ```
5. ✅ Semantic predicates work:
   ```fmpl
   any:x ?(x > 0) @ grammar.rule
   ```
6. ✅ VM reduced by ~400 lines (pattern matching section)
7. ✅ No `MatchStar`, `MatchPlus`, `MatchChoice` instructions
8. ✅ Unified input model (text and trees use same primitives)

---

## Open Questions

1. **List result collection:** Use immutable append or mutable builder?
   - Option A: `ListAppend` returns new list (functional)
   - Option B: `ListPush` mutates in place (imperative)
   - Recommendation: Start with A, optimize later if needed

2. **Memoization with input stack:** How to key the memo table?
   - Need `(stack_depth, position, rule)` as key
   - Or `(input_identity, position, rule)` for structural sharing

3. **Error messages:** How to report "expected X at position Y in nested structure"?
   - Track path through input stack for error context

4. **Semantic action scope:** What variables are in scope for `-> expr`?
   - All bindings from the pattern
   - Outer scope from grammar definition?

---

## Estimated Total Effort

| Phase | Hours |
|-------|-------|
| 1. Input stack | 2-3 |
| 2. Polymorphic MatchAny | 1-2 |
| 3. Lower Star | 2-3 |
| 4. Lower Plus | 1-2 |
| 5. Lower Choice | 2-3 |
| 6. Semantic predicates | 2-3 |
| 7. List pattern syntax | 3-4 |
| 8. Map pattern syntax | 2-3 |
| 9. Remove old instructions | 2-3 |
| 10. Documentation | 2-3 |
| **Total** | **20-29 hours** |

---

## Key Insight

**This is the OMeta way:** Grammars compile to the target language. The VM provides:
- Primitive matching (MatchAny, MatchLiteral, MatchCharClass)
- Control flow (Jump, JumpIfNull, JumpIfFalse)
- State management (ParseCheckpoint, ParsePush/Pop)

The compiler combines these to implement Star/Plus/Choice/ListPattern.

This aligns with FMPL's philosophy: **simple VM + smart compiler**.
