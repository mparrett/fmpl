# Indexed RPN Conversion Specification

Convert VM from stack-based bytecode to true Indexed RPN format.

**Status**: Enhanced (v2)
**Source**: [docs/designs/indexed-rpn.md](../docs/designs/indexed-rpn.md)

---

## Summary

Replace implicit stack-based operand passing with explicit index references. Each instruction that produces a value is assigned an index, and consuming instructions reference operands by their producing instruction's index.

---

## Current State (Stack-Based)

```rust
// (3 + 4) * 5
LoadInt(3)       // push 3
LoadInt(4)       // push 4
Add              // pop two, push result
LoadInt(5)       // push 5
Mul              // pop two, push result
```

Operands are implicit via stack position.

---

## Target State (Indexed RPN)

```rust
// (3 + 4) * 5
// Index 0: LoadInt(3)           → values[0] = 3
// Index 1: LoadInt(4)           → values[1] = 4
// Index 2: Add(lhs: 0, rhs: 1)  → values[2] = values[0] + values[1]
// Index 3: LoadInt(5)           → values[3] = 5
// Index 4: Mul(lhs: 2, rhs: 3)  → values[4] = values[2] * values[3]
```

Each instruction explicitly names its operands by index.

---

## Acceptance Criteria

### AC-1: Instruction Format

**Given** a binary operation (Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, Gt, LtEq, GtEq, And, Or)
**When** compiled
**Then** the instruction contains explicit `lhs` and `rhs` indices referencing prior instructions

**Example**:
```rust
// Before (stack-based)
Add

// After (indexed)
Add { lhs: InstrIndex, rhs: InstrIndex }
```

---

### AC-2: Unary Operations

**Given** a unary operation (Neg, Not)
**When** compiled
**Then** the instruction contains explicit `operand` index

**Example**:
```rust
// Before
Neg

// After
Neg { operand: InstrIndex }
```

---

### AC-3: Values Array Parallel to Instructions

**Given** compiled code with N instructions
**When** VM executes
**Then** a `values: Vec<Value>` array of size N is allocated, indexed by instruction position

**Example**:
```rust
fn run(&mut self, code: &CompiledCode) -> Result<Value> {
    let mut values: Vec<Value> = vec![Value::Null; code.instructions.len()];
    // Each instruction writes to values[ip]
}
```

---

### AC-4: No Operand Stack for Expressions

**Given** an arithmetic or logical expression
**When** executed
**Then** no push/pop operations occur; operands are read directly from `values[index]`

**Example**:
```rust
Instruction::Add { lhs, rhs } => {
    let left = values[lhs];
    let right = values[rhs];
    values[ip] = left + right;
}
```

---

### AC-5: Variable Binding Uses Bind Node

**Given** a let-binding `let x = expr`
**When** compiled
**Then** emits `Bind { name, value_index }` where `value_index` points to the expression's instruction

**Example**:
```fmpl
let a = 10
let b = a + 5
```

```rust
// Index 0: LoadInt(10)           → values[0] = 10
// Index 1: Bind("a", 0)          → scope["a"] = 0 (index, not value)
// Index 2: NameRef(1)            → values[2] = values[scope["a"]]
// Index 3: LoadInt(5)            → values[3] = 5
// Index 4: Add(2, 3)             → values[4] = values[2] + values[3]
// Index 5: Bind("b", 4)          → scope["b"] = 4
```

---

### AC-6: NameRef Resolves to Index

**Given** a variable reference `x`
**When** compiled
**Then** emits `NameRef { bind_index }` pointing to the `Bind` instruction that introduced the name

**When** executed
**Then** looks up the value index from the Bind and reads `values[value_index]`

---

### AC-7: Jumps Reference Instruction Indices

**Given** control flow (if/else, while)
**When** compiled
**Then** `Jump`, `JumpIfFalse`, `JumpIfTrue` contain target instruction indices (already implemented)

**Given** the condition for a branch
**When** compiled
**Then** `JumpIfFalse { cond_index, target }` explicitly references the condition's instruction index

**Example**:
```rust
// Before
JumpIfFalse(target)  // condition implicitly on stack top

// After
JumpIfFalse { cond: InstrIndex, target: InstrIndex }
```

---

### AC-8: Function Calls

**Given** a function call `f(a, b, c)`
**When** compiled
**Then** `Call { func_index, arg_indices: Vec<InstrIndex> }` explicitly lists the function and arguments

**Example**:
```rust
// Before
LoadVar("a")
LoadVar("b")
LoadVar("c")
LoadVar("f")
Call(3)

// After
// Index 0: LoadVar("a")
// Index 1: LoadVar("b")
// Index 2: LoadVar("c")
// Index 3: LoadVar("f")
// Index 4: Call { func: 3, args: [0, 1, 2] }
```

---

### AC-9: Method Calls

**Given** a method call `obj.method(a, b)`
**When** compiled
**Then** `MethodCall { receiver_index, method_name, arg_indices }` explicitly references receiver and args

---

### AC-10: List/Map Construction

**Given** a list literal `[a, b, c]`
**When** compiled
**Then** `MakeList { element_indices: Vec<InstrIndex> }` explicitly lists element indices

**Given** a map literal `%{k1: v1, k2: v2}`
**When** compiled
**Then** `MakeMap { key_value_pairs: Vec<(InstrIndex, InstrIndex)> }` pairs keys with values

---

### AC-11: Index/Slice Operations

**Given** an index operation `list[idx]`
**When** compiled
**Then** `Index { collection: InstrIndex, key: InstrIndex }`

**Given** a slice operation `list[start..end]`
**When** compiled
**Then** `Slice { collection: InstrIndex, start: Option<InstrIndex>, end: Option<InstrIndex> }`

**Note**: Slice bounds are optional to support `list[..]`, `list[start..]`, and `list[..end]` syntax:
- `list[..]` → `Slice { collection, start: None, end: None }`
- `list[1..]` → `Slice { collection, start: Some(idx), end: None }`
- `list[..5]` → `Slice { collection, start: None, end: Some(idx) }`
- `list[1..5]` → `Slice { collection, start: Some(idx1), end: Some(idx2) }`

---

### AC-12: Property Access/Assignment

**Given** property access `obj.prop`
**When** compiled
**Then** `GetProp { object: InstrIndex, name: SmolStr }`

**Given** property assignment `obj.prop = val`
**When** compiled
**Then** `SetProp { object: InstrIndex, name: SmolStr, value: InstrIndex }`

---

### AC-13: Lambda Capture

**Given** a lambda `\x x + captured_var`
**When** compiled
**Then** `MakeLambda` includes `captured_indices: Vec<InstrIndex>` for closure values

---

### AC-14: Stream Operations

**Given** a stream pipeline `stream |> map(\x x + 1)`
**When** compiled
**Then** stream ops explicitly reference their source: `StreamMap { source: InstrIndex, func: InstrIndex }`

---

### AC-15: Pattern Matching

**Given** pattern matching with extraction
**When** compiled
**Then** extraction ops reference their source: `ExtractMapKey { source: InstrIndex, key }`

---

### AC-16: Return Value

**Given** `Return`
**When** compiled
**Then** `Return { value: InstrIndex }` explicitly references the value to return

---

### AC-17: Pipe Operator

**Given** `x |> f`
**When** compiled
**Then** `Pipe { arg: InstrIndex, func: InstrIndex }` explicitly references both

---

### AC-18: Exception Handling

**Given** `throw expr`
**When** compiled
**Then** `Throw { value: InstrIndex }`

---

### AC-19: Spawn/Facet

**Given** `spawn Object(args)`
**When** compiled
**Then** `Spawn { object_index, arg_indices }`

**Given** `obj::facet`
**When** compiled
**Then** `GetFacet { object: InstrIndex, name }`

---

### AC-20: BlockStart/BlockEnd for Scope Delimiting

**Given** a block expression `{ ... }`
**When** compiled
**Then** emits `BlockStart` before the block body and `BlockEnd` after

**When** VM executes
**Then** these are no-ops for value computation; the `resolve_names` pass has already wired all `NameRef` nodes to their `Bind` instructions

**Example**:
```fmpl
let x = 1
{
  let x = 2
  x
}
x
```

```rust
// Index 0: LoadInt(1)
// Index 1: Bind("x", 0)
// Index 2: BlockStart           ← scope boundary
// Index 3: LoadInt(2)
// Index 4: Bind("x", 3)         ← shadows outer x within this block
// Index 5: NameRef(4)           ← resolved to inner Bind
// Index 6: BlockEnd             ← scope boundary
// Index 7: NameRef(1)           ← resolved to outer Bind
```

**Note**: `PushScope`/`PopScope` are replaced by `BlockStart`/`BlockEnd`. The latter are metadata instructions that guide name resolution but have no runtime effect.

---

### AC-21: NameRef Resolution is Static

**Given** a compiled program
**When** `NameRef` instructions execute
**Then** they do NOT perform runtime scope lookup; they directly reference the `Bind` instruction's value index (resolved at compile time by `resolve_names`)

**Example**:
```rust
// After resolve_names, NameRef holds bind_index, not a string
Instruction::NameRef { bind: InstrIndex } => {
    // Look up which value index the Bind points to
    if let Instruction::Bind { value, .. } = &code.instructions[bind.0] {
        values[ip] = values[value.0].clone();
    }
}
```

---

## InstrIndex Type

```rust
/// Index into the instructions array, used for operand references
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstrIndex(pub usize);
```

---

## Revised Instruction Enum

```rust
pub enum Instruction {
    // Literals (produce values, no operand references)
    LoadNull,
    LoadBool(bool),
    LoadInt(i64),
    LoadFloat(f64),
    LoadString(SmolStr),
    LoadSymbol(SmolStr),

    // Variable access
    LoadVar(SmolStr),
    StoreVar { name: SmolStr, value: InstrIndex },

    // Special references (produce values)
    LoadSelf,
    LoadParent,
    LoadCaller,
    LoadUser,
    LoadArgs,

    // Binary arithmetic
    Add { lhs: InstrIndex, rhs: InstrIndex },
    Sub { lhs: InstrIndex, rhs: InstrIndex },
    Mul { lhs: InstrIndex, rhs: InstrIndex },
    Div { lhs: InstrIndex, rhs: InstrIndex },
    Mod { lhs: InstrIndex, rhs: InstrIndex },

    // Unary
    Neg { operand: InstrIndex },
    Not { operand: InstrIndex },

    // Comparison
    Eq { lhs: InstrIndex, rhs: InstrIndex },
    NotEq { lhs: InstrIndex, rhs: InstrIndex },
    Lt { lhs: InstrIndex, rhs: InstrIndex },
    Gt { lhs: InstrIndex, rhs: InstrIndex },
    LtEq { lhs: InstrIndex, rhs: InstrIndex },
    GtEq { lhs: InstrIndex, rhs: InstrIndex },

    // Logical
    And { lhs: InstrIndex, rhs: InstrIndex },
    Or { lhs: InstrIndex, rhs: InstrIndex },

    // Control flow
    Jump { target: InstrIndex },
    JumpIfFalse { cond: InstrIndex, target: InstrIndex },
    JumpIfTrue { cond: InstrIndex, target: InstrIndex },

    // Functions
    Call { func: InstrIndex, args: Vec<InstrIndex> },
    TailCall { func: InstrIndex, args: Vec<InstrIndex> },
    MethodCall { receiver: InstrIndex, method: SmolStr, args: Vec<InstrIndex> },
    Return { value: InstrIndex },

    // Objects
    GetProp { object: InstrIndex, name: SmolStr },
    SetProp { object: InstrIndex, name: SmolStr, value: InstrIndex },
    Spawn { object: InstrIndex, args: Vec<InstrIndex> },
    GetFacet { object: InstrIndex, name: SmolStr },

    // Sync/Async
    SyncCall { target: InstrIndex },
    AsyncCall { target: InstrIndex },

    // Data structures
    MakeList { elements: Vec<InstrIndex> },
    MakeMap { pairs: Vec<(InstrIndex, InstrIndex)> },
    Index { collection: InstrIndex, key: InstrIndex },
    Slice { collection: InstrIndex, start: Option<InstrIndex>, end: Option<InstrIndex> },

    // Binding & Scope (BlockStart/BlockEnd replace PushScope/PopScope)
    BlockStart,                                  // Scope boundary: begin
    BlockEnd,                                    // Scope boundary: end
    Bind { name: SmolStr, value: InstrIndex },   // Introducer: name → value index
    NameRef { bind: InstrIndex },                // Reference to Bind instruction

    // Lambda
    MakeLambda { params: Vec<SmolStr>, body: usize, captures: Vec<InstrIndex> },

    // Pipe
    Pipe { arg: InstrIndex, func: InstrIndex },

    // Streams
    MakeStream { source: InstrIndex },
    StreamMap { source: InstrIndex, func: InstrIndex },
    StreamFilter { source: InstrIndex, pred: InstrIndex },
    StreamFlatMap { source: InstrIndex, func: InstrIndex },
    StreamReduce { source: InstrIndex, init: InstrIndex, func: InstrIndex },
    StreamParse { source: InstrIndex, grammar: SmolStr },

    // Pattern matching
    MatchPattern { value: InstrIndex, pattern_count: usize },
    ExtractMapKey { source: InstrIndex, key: SmolStr },
    ExtractListIndex { source: InstrIndex, index: usize },

    // Object definition
    DefineObject { name: SmolStr },
    DefineMethod { name: SmolStr, body: usize },
    DefineProp { name: SmolStr, value: InstrIndex },
    DefineFacet { name: SmolStr, body: usize, public: bool },

    // Grammar
    GrammarApply { input: InstrIndex, grammar: InstrIndex, rule: SmolStr },
    LoadGrammar(Arc<Grammar>),
    ExtendGrammar { base: InstrIndex, extension: Grammar },

    // Exception handling
    PushHandler { catch_target: InstrIndex },
    PopHandler,
    Throw { value: InstrIndex },

    // No-op (for Dup that becomes unnecessary, or placeholder)
    Nop,
}
```

---

## resolve_names Algorithm

The `resolve_names` pass runs after initial compilation to wire all `NameRef` instructions to their `Bind` introducers. This eliminates runtime scope lookup.

### Algorithm

```rust
fn resolve_names(instructions: &mut [Instruction]) {
    // Stack of scopes: each scope maps name → Bind instruction index
    let mut scope_stack: Vec<HashMap<SmolStr, InstrIndex>> = vec![HashMap::new()];

    for ip in 0..instructions.len() {
        match &instructions[ip] {
            Instruction::BlockStart => {
                // Push new scope
                scope_stack.push(HashMap::new());
            }

            Instruction::BlockEnd => {
                // Pop scope - inner bindings are forgotten
                scope_stack.pop();
            }

            Instruction::Bind { name, .. } => {
                // Register in current (top) scope
                let current = scope_stack.last_mut().unwrap();
                current.insert(name.clone(), InstrIndex(ip));
            }

            Instruction::NameRef { bind } if bind.0 == usize::MAX => {
                // Unresolved NameRef - need to look up by name
                // (Initial compilation emits NameRef with placeholder bind)
                let name = get_unresolved_name(&instructions[ip]); // implementation detail

                // Search from innermost to outermost scope
                let target = scope_stack.iter().rev()
                    .find_map(|scope| scope.get(&name).copied())
                    .expect("undefined variable");

                // Patch the instruction
                instructions[ip] = Instruction::NameRef { bind: target };
            }

            _ => {}
        }
    }
}
```

### Key Properties

1. **Single pass**: O(n) traversal of instruction array
2. **Lexical scoping**: Inner scopes shadow outer bindings
3. **No runtime lookup**: After resolution, `NameRef` directly references `Bind` index
4. **Scope stack discarded**: Only needed during resolution pass

### Unresolved NameRef Representation

During initial compilation, `NameRef` is emitted with a placeholder:
```rust
// Compiler emits this before resolve_names
Instruction::UnresolvedNameRef { name: SmolStr }

// Or alternatively, use a sentinel value
Instruction::NameRef { bind: InstrIndex(usize::MAX) }  // placeholder
```

After `resolve_names`, all `NameRef` instructions have valid `bind` indices.

---

## Backpatching for Control Flow

### Problem

When compiling `if cond { then } else { els }`, we emit `JumpIfFalse` before knowing where the `else` block starts. Similarly, we emit `Jump` at the end of `then` before knowing where to merge.

### Solution: Backpatching

Reserve instruction slots with placeholder targets, then update them once the target is known.

### Algorithm for If-Else

```rust
fn compile_if_else(&mut self, cond: &Expr, then_body: &Block, else_body: &Block) -> InstrIndex {
    // 1. Compile condition
    let cond_idx = self.compile_expr(cond)?;

    // 2. Emit JumpIfFalse with placeholder target
    let jump_to_else_idx = self.emit(Instruction::JumpIfFalse {
        cond: cond_idx,
        target: InstrIndex(usize::MAX),  // placeholder
    });

    // 3. Compile "then" block
    let then_result = self.compile_block(then_body)?;

    // 4. Emit Jump (skip else) with placeholder
    let jump_to_end_idx = self.emit(Instruction::Jump {
        target: InstrIndex(usize::MAX),  // placeholder
    });

    // 5. Mark else start - BACKPATCH jump_to_else
    let else_start = self.current_index();
    self.patch_jump(jump_to_else_idx, else_start);

    // 6. Compile "else" block
    let else_result = self.compile_block(else_body)?;

    // 7. Mark end - BACKPATCH jump_to_end
    let end_idx = self.current_index();
    self.patch_jump(jump_to_end_idx, end_idx);

    // 8. Return index of final result (phi node or last instruction)
    end_idx
}

fn patch_jump(&mut self, jump_idx: InstrIndex, target: InstrIndex) {
    match &mut self.instructions[jump_idx.0] {
        Instruction::Jump { target: t } => *t = target,
        Instruction::JumpIfFalse { target: t, .. } => *t = target,
        Instruction::JumpIfTrue { target: t, .. } => *t = target,
        _ => panic!("not a jump instruction"),
    }
}
```

### Backpatching for While Loop

```rust
fn compile_while(&mut self, cond: &Expr, body: &Block) -> InstrIndex {
    // 1. Mark loop start
    let loop_start = self.current_index();

    // 2. Compile condition
    let cond_idx = self.compile_expr(cond)?;

    // 3. Emit JumpIfFalse to exit (placeholder)
    let jump_to_exit_idx = self.emit(Instruction::JumpIfFalse {
        cond: cond_idx,
        target: InstrIndex(usize::MAX),
    });

    // 4. Compile body
    self.compile_block(body)?;

    // 5. Jump back to loop start
    self.emit(Instruction::Jump { target: loop_start });

    // 6. Mark exit - BACKPATCH
    let exit_idx = self.current_index();
    self.patch_jump(jump_to_exit_idx, exit_idx);

    // While loops evaluate to null
    self.emit(Instruction::LoadNull)
}
```

### Key Properties

1. **Forward references**: Jumps can target instructions not yet emitted
2. **Placeholder sentinel**: `InstrIndex(usize::MAX)` marks unresolved targets
3. **Mutable patching**: Jump targets updated in-place after target is known
4. **No separate linking phase**: Backpatching happens during compilation

---

## Compiler Changes

### Track Current Instruction Index

```rust
struct Compiler {
    instructions: Vec<Instruction>,
    // ... other fields
}

impl Compiler {
    fn current_index(&self) -> InstrIndex {
        InstrIndex(self.instructions.len())
    }

    fn emit(&mut self, instr: Instruction) -> InstrIndex {
        let idx = self.current_index();
        self.instructions.push(instr);
        idx
    }
}
```

### Expression Compilation Returns Index

```rust
fn compile_expr(&mut self, expr: &Expr) -> Result<InstrIndex> {
    match expr {
        Expr::Int(n) => Ok(self.emit(Instruction::LoadInt(*n))),
        Expr::BinOp(op, lhs, rhs) => {
            let lhs_idx = self.compile_expr(lhs)?;
            let rhs_idx = self.compile_expr(rhs)?;
            Ok(self.emit(match op {
                Op::Add => Instruction::Add { lhs: lhs_idx, rhs: rhs_idx },
                Op::Sub => Instruction::Sub { lhs: lhs_idx, rhs: rhs_idx },
                // ...
            }))
        }
        // ...
    }
}
```

---

## VM Changes

### Values Array Replaces Operand Stack

```rust
impl Vm {
    pub fn run(&mut self, code: &CompiledCode) -> Result<Value> {
        let mut values: Vec<Value> = vec![Value::Null; code.instructions.len()];
        let mut ip = 0;

        while ip < code.instructions.len() {
            match &code.instructions[ip] {
                Instruction::LoadInt(n) => {
                    values[ip] = Value::Int(*n);
                    ip += 1;
                }
                Instruction::Add { lhs, rhs } => {
                    let left = &values[lhs.0];
                    let right = &values[rhs.0];
                    values[ip] = add_values(left, right)?;
                    ip += 1;
                }
                Instruction::JumpIfFalse { cond, target } => {
                    if values[cond.0].is_falsy() {
                        ip = target.0;
                    } else {
                        ip += 1;
                    }
                }
                // ...
            }
        }

        // Return value of last instruction
        Ok(values.last().cloned().unwrap_or(Value::Null))
    }
}
```

---

## Edge Cases

### E-1: Short-Circuit Evaluation

**Given** `a && b` or `a || b`
**When** short-circuit occurs
**Then** `b` is not evaluated; `And`/`Or` instructions must handle unevaluated operands

**Solution**: Compile `&&` and `||` using `JumpIfFalse`/`JumpIfTrue` control flow rather than as single instructions:

```rust
// a && b
// Index 0: <compile a>
// Index 1: JumpIfFalse { cond: 0, target: 4 }  // skip to false result
// Index 2: <compile b>
// Index 3: Jump { target: 5 }
// Index 4: LoadBool(false)
// Index 5: (continuation)
```

---

### E-2: Function Bodies as Nested Code

**Given** lambdas and method bodies
**When** compiled
**Then** remain in `CompiledCode::nested` with separate instruction arrays

The body gets its own `values` array during execution.

---

### E-3: Recursive Calls

**Given** recursive function call
**When** executed
**Then** each call frame has its own `values` array; no conflict

---

### E-4: Pop Instruction Removal

**Given** `Pop` was used to discard unused values
**When** converted to Indexed RPN
**Then** `Pop` becomes `Nop` or is eliminated; values simply go unused

---

### E-5: Dup Instruction Removal

**Given** `Dup` was used to duplicate stack top
**When** converted to Indexed RPN
**Then** `Dup` is eliminated; consuming instructions reference the same index multiple times

---

## Out of Scope

- JIT compilation optimizations
- Register allocation
- Instruction reordering/scheduling
- Dead code elimination (may reference unused indices)
- Type inference from indexed flow

---

## Performance Considerations

- Values array pre-allocated to instruction count (no dynamic growth)
- Direct index access (O(1)) vs stack operations
- Cache-friendly linear memory layout
- No pointer chasing for operand access

---

## Migration Strategy

1. **Add `InstrIndex` type** — Define the newtype wrapper
2. **Add `BlockStart`/`BlockEnd` instructions** — Replace PushScope/PopScope
3. **Add indexed variants to `Instruction`** — Keep old variants temporarily
4. **Implement `resolve_names` pass** — Wire NameRef to Bind indices
5. **Update `Compiler`** — Emit indexed instructions with backpatching
6. **Update `Vm`** — Handle indexed instructions with values array
7. **Remove old stack-based variants** — Once all tests pass
8. **Remove operand stack** — Final cleanup

---

## Test Cases

### T-1: Simple Arithmetic
```fmpl
(3 + 4) * 5
```
Expected: 35

### T-2: Variable Binding
```fmpl
let x = 10
let y = x + 5
y
```
Expected: 15

### T-3: Nested Scopes
```fmpl
let x = 1
{
  let x = 2
  x  // 2
}
x  // 1
```
Expected outer: 1, inner: 2

### T-4: Conditional
```fmpl
if true { 1 } else { 2 }
```
Expected: 1

### T-5: Function Call
```fmpl
let add = \a \b a + b
add(3, 4)
```
Expected: 7

### T-6: Short-Circuit
```fmpl
false && (1 / 0)  // should not error
```
Expected: false (no division by zero)

### T-7: List Construction
```fmpl
[1, 2, 3]
```
Expected: list of 3 elements

### T-8: Method Call
```fmpl
[1, 2, 3].len()
```
Expected: 3

### T-9: Deep Nesting
```fmpl
let x = 1
{
  let y = 2
  {
    let z = 3
    x + y + z
  }
}
```
Expected: 6

### T-10: Shadowing Across Blocks
```fmpl
let x = 10
let result = {
  let x = 20
  x
}
result + x
```
Expected: 30 (result=20, x=10)

### T-11: Backpatching Integrity
```fmpl
let x = if true { 1 } else { 2 }
let y = if false { 3 } else { 4 }
x + y
```
Expected: 5

### T-12: While Loop
```fmpl
let sum = 0
let i = 0
while i < 3 {
  sum = sum + i
  i = i + 1
}
sum
```
Expected: 3 (0 + 1 + 2)

### T-13: resolve_names Wiring
```fmpl
// After resolve_names, NameRef for inner x points to Bind at index 4
let x = 1
{
  let x = 2
  x  // NameRef(4), not string lookup
}
```
Verify: NameRef instruction contains bind index, not string name

---

## References

- [Indexed RPN Design Doc](../docs/designs/indexed-rpn.md)
- [Carbon Compiler](https://github.com/carbon-language/carbon-lang)
- [Current VM Spec](./vm.md)
