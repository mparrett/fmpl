# OMeta Lowering to Base IR Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate complex pattern matching instructions by lowering grammar patterns to base Indexed RPN control flow.

**Architecture:** Instead of specialized VM instructions for Star/Plus/Choice, the compiler lowers patterns to loops and conditional jumps.

**Tech Stack:** Rust, FMPL VM, Compiler

---

## Background

### Current State

The VM has specialized pattern matching instructions:

**Complex Combinators (to be removed):**
- `MatchStar { pattern }` - Zero or more (127 lines of inline pattern logic)
- `MatchPlus { pattern }` - One or more (135 lines)
- `MatchChoice { cases }` - Ordered choice (69 lines, BROKEN)

**Specialized Repeats (to be removed):**
- `MatchStarChar`, `MatchStarCharClass`, `MatchStarLiteral`
- `MatchPlusChar`, `MatchPlusCharClass`, `MatchPlusLiteral`

**Problem:**
- ~400 lines of duplicated pattern matching logic in vm.rs
- Generic `MatchStar`/`MatchPlus` only handle 3 pattern types (Char, CharClass, Literal)
- `MatchChoice` has placeholder implementation
- Maintenance burden: adding new patterns requires new VM code

### The OMeta Approach

OMeta **compiles** PEG grammars to the target language. It doesn't add new VM instructions.

For example, in JavaScript OMeta:
```javascript
// OMeta grammar
digit = [0-9]
integer = digit+

// Compiles to JavaScript like:
function integer() {
  var results = [];
  while (true) {
    var checkpoint = this.savePosition();
    try {
      var r = this.apply('digit');
      results.push(r);
    } catch (e) {
      this.restorePosition(checkpoint);
      break;
    }
  }
  if (results.length === 0) throw new Error();
  return results;
}
```

### The FMPL Opportunity

FMPL's Indexed RPN VM already has:
- **Loops**: Via `Jump` back to a label
- **Conditionals**: `JumpIfNull`, `JumpIfFalse`, `JumpIfTrue`
- **Checkpoint/Restore**: `ParseCheckpoint` in ParseState

We can lower grammar patterns to these primitives!

---

## Proposed Architecture

### Compiler Lowering

Instead of emitting `MatchStar { pattern }`, the compiler emits:

```
# Star(digit) compiles to:
results = []                 -- MakeList (empty)
loop_start:                  -- LABEL:
  checkpoint = save()        -- ParseCheckpoint
  try:
    match digit              -- MatchCharClass [0-9]
    results = results ++ [r] -- (append logic)
    if pos == checkpoint_pos: jump loop_end  -- zero-length guard
    jump loop_start          -- Jump
  catch:
    restore(checkpoint)      -- ParseRestore
    jump loop_end            -- Jump
loop_end:                    -- LABEL:
  return results             -- Return
```

### Phase 1: Lower Star/Plus/Choice to IR

**Before:**
```rust
Instruction::MatchStar { pattern: InstrIndex }
```

**After (compiler emits):**
- Loop with checkpoint
- Pattern instruction
- Append to results
- Jump back / break on failure

**Key insight:** The compiler generates the loop structure, not the VM!

### Phase 2: Remove Specialized Instructions

Remove from `Instruction` enum:
- `MatchStar`, `MatchPlus`, `MatchChoice`
- `MatchStarChar`, `MatchPlusChar`, etc.
- `MatchStarRule`, `MatchPlusRule` (use ApplyRule in loop instead)

Keep primitives:
- `MatchChar`, `MatchLiteral`, `MatchCharClass`, `MatchNegCharClass`
- `MatchByte`, `MatchAny`
- `MatchEnd`
- `ApplyRule` (for rule application)

---

## Implementation Plan

### Phase 1: Add Checkpoint/Restore Instructions (1 hour)

**Add to Instruction enum:**
```rust
ParseCheckpoint,           // Create checkpoint for backtracking, store in values[ip]
ParseRestore { checkpoint: InstrIndex },  // Restore from checkpoint
```

**VM implementation:**
- `ParseCheckpoint`: Creates `ParseCheckpoint` with current `input_pos`, stores in `values[ip]`
- `ParseRestore`: Restores `input_pos` from checkpoint value

**Acceptance:**
- Instructions compile and execute
- Tests pass

---

### Phase 2: Compiler Lowering for Star (2-3 hours)

**Update `compile_grammar_pattern` for `Star(p)`:**

```rust
GP::Star(p) => {
    // Compile to loop:
    // 1. Create empty results list
    // 2. Loop start label
    // 3. ParseCheckpoint
    // 4. Compile pattern p
    // 5. Append result to list
    // 6. Zero-length guard: jump to end if position unchanged
    // 7. Jump back to loop start
    // 8. (Pattern failure falls through to restore)
    // 9. ParseRestore
    // 10. Jump to end
    // 11. Loop end label
    // 12. Return results

    let loop_start = self.code.emit_placeholder();  // Will fix up later
    let checkpoint_idx = self.code.emit(Instruction::ParseCheckpoint);
    let pattern_idx = self.compile_grammar_pattern(p)?;

    // Append pattern result to list (TBD - need list append instruction)
    // For now, collect as we go - may need helper instruction

    let zero_length_guard = self.code.emit(Instruction::JumpIfNull {
        cond: /* position comparison */,
        target: /* end label */
    });

    let jump_back = self.code.emit(Instruction::Jump { target: loop_start });

    let restore_idx = self.code.emit(Instruction::ParseRestore { checkpoint: checkpoint_idx });
    let jump_to_end = self.code.emit(Instruction::Jump { target: /* end label */ });

    let loop_end = self.code.next_index();

    // Fix up jump targets
    self.code.fixup_target(loop_start, loop_end);  // Jump back to loop start
    // ... fix up other jumps

    // Return the result list
    loop_end
}
```

**Challenge:** Need a way to:
1. Collect results in a list
2. Compare positions (for zero-length guard)
3. Handle pattern failure (jump to restore)

**Possible solutions:**
- Add `ListAppend { list: InstrIndex, item: InstrIndex }` instruction
- Add `PositionEq { pos1: InstrIndex, pos2: InstrIndex }` instruction
- Use exception handling for pattern failure

**Acceptance:**
- `Star(digit)` compiles to loop bytecode
- Runtime executes correctly
- Tests pass

---

### Phase 3: Compiler Lowering for Plus (1-2 hours)

Similar to Star, but:
- Require at least one match (check if results is empty after loop)
- Return error if no matches

**Acceptance:**
- `Plus(digit)` compiles and executes correctly
- Returns error if no match

---

### Phase 4: Compiler Lowering for Choice (2-3 hours)

**Update `compile_grammar_pattern` for `Choice([p1, p2, p3])`:**

```rust
GP::Choice(patterns) => {
    // Compile to if/else chain with backtracking:
    // 1. Checkpoint
    // 2. Try p1
    // 3. Jump to end on success
    // 4. (Failure falls through)
    // 5. Restore checkpoint
    // 6. Try p2
    // 7. Jump to end on success
    // 8. (Failure falls through)
    // 9. Restore checkpoint
    // 10. Try p3
    // 11. Return result

    let mut end_jumps = Vec::new();

    for (i, pattern) in patterns.iter().enumerate() {
        let checkpoint_idx = self.code.emit(Instruction::ParseCheckpoint);
        let pattern_idx = self.compile_grammar_pattern(pattern)?;

        // Jump to end if pattern succeeded
        let jump_to_end = self.code.emit(Instruction::JumpIfNull {
            cond: pattern_idx,  // If Null, pattern failed
            target: /* next case */,
        });

        end_jumps.push(jump_to_end);

        // Restore and try next pattern on failure
        let restore_idx = self.code.emit(Instruction::ParseRestore { checkpoint: checkpoint_idx });
    }

    // All cases failed - return error
    self.code.emit(Instruction::Return { value: /* error */ });

    // Fix up jump targets
    // ...
}
```

**Acceptance:**
- `Choice([Char('a'), Char('b')])` compiles and executes
- Backtracking works correctly

---

### Phase 5: Remove Old Instructions (1-2 hours)

**Remove from Instruction enum:**
- `MatchStar`, `MatchPlus`, `MatchChoice`
- `MatchStarChar`, `MatchPlusChar`, `MatchStarCharClass`, `MatchPlusCharClass`
- `MatchStarLiteral`, `MatchPlusLiteral`
- `MatchStarRule`, `MatchPlusRule`

**Remove from VM:**
- Delete all `MatchStar*` and `MatchPlus*` instruction handlers (~400 lines)

**Acceptance:**
- Code compiles
- All tests pass
- VM is significantly smaller

---

### Phase 6: Add Helper Instructions (if needed) (1-2 hours)

Based on Phase 2-4 experience, add:
- `ListAppend { list: InstrIndex, item: InstrIndex }`
- `GetPosition { }` - Get current parse position
- `PositionEq { pos1: InstrIndex, pos2: InstrIndex }` - Compare positions
- Or use exception handling for control flow

**Acceptance:**
- Helper instructions work correctly
- Tests pass

---

### Phase 7: Cleanup and Documentation (1-2 hours)

1. Update `docs/grammar-system.md` with new lowering approach
2. Add examples of compiled bytecode
3. Remove unused code
4. Update comments

**Acceptance:**
- Documentation updated
- Code is clean

---

## Success Criteria

1. ✅ All 470+ tests pass
2. ✅ VM code reduced by ~400 lines (pattern matching section)
3. ✅ No `MatchStar`, `MatchPlus`, `MatchChoice` instructions
4. ✅ Grammar patterns compile to base IR (loops + jumps)
5. ✅ Documentation updated

---

## Open Questions

1. **How to collect results in Star/Plus loops?**
   - Option A: Add `ListAppend` instruction
   - Option B: Use `MakeList` with variable elements
   - Option C: Exception-based control flow

2. **How to handle pattern failure in the middle of a loop?**
   - Option A: Exception handling
   - Option B: Pattern instructions return `Result<Value>` and we check after each
   - Option C: Jump on null pattern

3. **How to compare positions for zero-length guard?**
   - Option A: Add position comparison instruction
   - Option B: Check position in ParseCheckpoint directly
   - Option C: Use different approach

---

## Next Steps

1. Review this approach
2. Implement Phase 1 (checkpoint/restore instructions)
3. Test Phase 2 (Star lowering) with a simple example
4. Iterate based on what works
5. Complete all phases

---

## Key Insight

**This is the OMeta way!** Grammar patterns are not VM features - they're compiler library features. The VM provides:
- Basic pattern matching (MatchChar, MatchLiteral, etc.)
- Control flow (Jump, JumpIfNull)
- State management (ParseCheckpoint)

The compiler combines these to implement Star/Plus/Choice.

This aligns with FMPL's philosophy: simple VM + smart compiler.
