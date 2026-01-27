# First-Class AST and IR Design

## Overview

Expose Grammars, AST, and IR as first-class values in the FMPL VM, enabling full metaprogramming: write compilers, DSLs, and code generators in FMPL itself.

**Pipeline:**
```
Source (String) → AST (Tagged Values) → IR (Tagged Values) → CompiledCode (opaque)
         ↑               ↑                      ↑                      ↑
    parse()        tree grammars          tree grammars          compile()
```

Each stage except final `CompiledCode` is plain FMPL data that grammars can transform.

---

## Value::Tagged

New value type for tagged/constructor values:

```rust
Value::Tagged(SmolStr, Arc<Vec<Value>>)  // (symbol_name, children)
```

**Construction syntax:**
```fmpl
:Binary(:+, lhs, rhs)
:Int(42)
:Call(:print, [:LoadString("hello")])
```

**Pattern matching:**
```fmpl
expr @ {
    :Binary(op, lhs, rhs) => transform(op, lhs, rhs)
    :Int(n) => :LoadInt(n)
    _ => error("unknown node")
}
```

Uses symbols (`:Symbol`) for type/node names - interned, fast to compare, semantically clear.

---

## AST Representation

The `parse` builtin returns tagged values. Node types mirror `ast::Expr`:

**Literals:**
```fmpl
:Int(42)
:Float(3.14)
:String("hello")
:Symbol(:foo)
:Bool(true)
:Null
```

**Variables and access:**
```fmpl
:Var(:x)
:GetProp(:Var(:obj), :field)
:Index(:Var(:arr), :Int(0))
```

**Operations:**
```fmpl
:Binary(:+, :Var(:x), :Int(1))
:Unary(:-, :Var(:x))
```

**Control flow:**
```fmpl
:If(:cond, :then, :else)
:While(:cond, :body)
:Let([:Binding(:x, :Int(42))], :body)
:Lambda([:x, :y], :body)
:Call(:func, [:arg1, :arg2])
```

**Grammar-related:**
```fmpl
:GrammarApply(:input, :grammar, :rule)
:GrammarLiteral(:grammar_data)
```

---

## IR Representation

Tree-based with optional named temporaries. The `compile` builtin linearizes to Indexed RPN.

**Expressions (tree-based):**
```fmpl
:Add(:LoadInt(1), :LoadInt(2))
:Call(:LoadVar(:print), [:LoadString("hello")])
:GetProp(:LoadVar(:obj), :name)
```

**Sharing via Let:**
```fmpl
:Let(:x, :Call(:expensive),
  :Mul(:Var(:x), :Var(:x)))
```

**Sequencing:**
```fmpl
:Seq([
  :Let(:a, :LoadInt(1)),
  :Let(:b, :Add(:Var(:a), :LoadInt(2))),
  :Var(:b)
])
```

**Control flow:**
```fmpl
:If(:cond, :then_expr, :else_expr)
:While(:cond, :body)
:Block(:label, :body)
:Break(:label, :value)
:Continue(:label)
```

The IR mirrors the Instruction enum but in tree form. `compile` handles:
- Walking the tree
- Allocating instruction indices
- Tracking Let binding name → index mappings
- Emitting Indexed RPN instructions

---

## Builtins

**`parse(source)`**
- Input: String of FMPL source code
- Output: Tagged value representing AST
- Implementation: Uses FMPL grammar (metacircular)
- Example: `parse("1 + 2")` → `:Binary(:+, :Int(1), :Int(2))`

**`compile(ir)`**
- Input: Tagged value representing IR
- Output: `Value::Code(Arc<CompiledCode>)` - opaque, executable
- Example: `compile(:Add(:LoadInt(1), :LoadInt(2)))` → `<code>`

**`eval(code)`**
- Input: CompiledCode value
- Output: Result of executing the code
- Example: `eval(compile(:LoadInt(42)))` → `42`

---

## Value::Code

New opaque value type for compiled bytecode:

```rust
Value::Code(Arc<CompiledCode>)
```

Cannot be inspected or decompiled (for now). Only executable via `eval`.

---

## Implementation Plan

### Phase 1: Value::Tagged
- Add `Value::Tagged(SmolStr, Arc<Vec<Value>>)` to value.rs
- Add construction syntax `:Symbol(args...)` to parser
- Implement `Pattern::Constructor` matching in compiler/VM
- Add `source_repr` for tagged values
- Add tests

### Phase 2: AST Reification
- Add `ast_to_value(expr: &Expr) -> Value` function
- Returns tagged values representing AST nodes
- Wire up `parse` builtin using FMPL grammar
- Add tests for round-tripping

### Phase 3: IR and Compile
- Add `Value::Code(Arc<CompiledCode>)` variant
- Implement `compile(ir: Value) -> Result<Value::Code>` builtin
- Walk IR tree, track Let bindings, emit Indexed RPN
- Error on malformed IR
- Add tests

### Phase 4: Eval
- Add `eval(code: Value::Code) -> Value` builtin
- Execute compiled code in current VM context
- Add tests

### Phase 5: Integration Testing
- Round-trip: `eval(compile(ast_to_ir(parse(source)))) == eval(source)`
- Write simple AST→IR grammar in FMPL
- Test error cases (malformed IR, type errors, etc.)

---

## Example: Simple AST→IR Grammar

```fmpl
grammar ast_to_ir {
  expr = :Int(n) => :LoadInt(n)
       | :String(s) => :LoadString(s)
       | :Bool(b) => :LoadBool(b)
       | :Null => :LoadNull
       | :Var(name) => :LoadVar(name)
       | :Binary(op, lhs, rhs) => binary_op(op, expr(lhs), expr(rhs))
       | :Unary(op, e) => unary_op(op, expr(e))
       | :If(c, t, e) => :If(expr(c), expr(t), expr(e))
       | :Call(f, args) => :Call(expr(f), args.map(expr))
       | :Lambda(params, body) => :Lambda(params, expr(body))
       | :Let(bindings, body) => compile_let(bindings, body)

  binary_op(op, l, r) = op @ {
    :+ => :Add(l, r)
    :- => :Sub(l, r)
    :* => :Mul(l, r)
    :/ => :Div(l, r)
    :== => :Eq(l, r)
    :< => :Lt(l, r)
    _ => error("unknown op: " + op)
  }

  compile_let([], body) = expr(body)
  compile_let([:Binding(name, val) | rest], body) =
    :Let(name, expr(val), compile_let(rest, body))
}

-- Usage
let (ast = parse("1 + 2 * 3")) in
let (ir = ast @ ast_to_ir.expr) in
let (code = compile(ir)) in
eval(code)  -- => 7
```

---

## Open Questions

1. **Hygiene for macros**: If we add `quote`/`unquote`, how do we handle variable capture?

2. **Debugging info**: Should IR/Code preserve source locations for error messages?

3. **Incremental compilation**: Can we compile individual functions without recompiling everything?

4. **Grammar values**: Grammars are already first-class. Should `parse` return the grammar used, for inspection?

---

## References

- [OMeta](http://www.tinlizzie.org/~awarth/papers/dls07.pdf) - Tree transformation grammars
- [Indexed RPN](https://burakemir.ch/post/indexed-rpn/) - FMPL's bytecode format
- `fmpl-core/tests/fmpl/fmpl_grammar.fmpl` - Metacircular FMPL parser
