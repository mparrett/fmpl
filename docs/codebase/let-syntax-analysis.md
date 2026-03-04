# Let Syntax: Grammar vs Implementation Analysis

## EBNF Grammar (fmpl.ebnf)

Single form only:

```
<exp> ::= <LET> <letsymlist> <exp>
<letsymlist> ::= '(' <letsymseq> ')' | '(' ')'
<letsymseq> ::= <letsym> | <letsymseq> ',' <letsym> | <letsymseq> <letsym>
<letsym> ::= <TAG> | <TAG> <EQ> <exp> | <pattern> <EQ> <exp>
```

Semantics: `let (bindings) body` — scoped expression with explicit binding list.

## Implementation (3 components)

### AST (`ast.rs:250-253`)

Two distinct nodes:

- `Let(Vec<LetBinding>, Box<Expr>)` — expression-style, creates new scope
- `LetStmt(SmolStr, Box<Expr>)` — statement-style, binds to current scope

### Parser (`parser.rs:1599-1652`)

Dispatches on `(` after `let`:

- `let (...)` → `Expr::Let` (matches EBNF)
- `let name = expr` → `Expr::LetStmt` (NOT in EBNF)

### Compiler (`compiler.rs:2355-2398`)

- `compile_let`: emits `PushScope` → bindings → body → `PopScope` → `Copy`
- `compile_let_stmt`: emits `Bind` → `Copy` (no scope push/pop)

### Bootstrap (`ast_to_ir.fmpl:38-39`)

Only handles single-binding `Let`:

```
| :Let([:Binding(name, expr:value)], expr:body) => :Let(name, value, body)
```

Does NOT handle:
- Multi-binding `Let` (list with >1 binding)
- `LetStmt` node
- Destructuring bindings

## Analysis

### Intentional expansion: `LetStmt`

`LetStmt` is required for REPL and sequence blocks — `let x = 1; let y = 2; x + y` needs bindings to persist in the current scope, not create nested scopes. This is a deliberate semantic split, not accidental drift. The `value_to_ast.rs:1211` `transform_do_to_nested_lets` function converts sequences of `LetStmt` into nested `Let` expressions when needed.

### Actual drift: bootstrap pipeline gaps

The `ast_to_ir.fmpl` grammar only handles single-binding Let. Multi-binding lets and destructuring lets are not transformed, causing silent failures in the bootstrap pipeline. This is tracked by existing parity tests.

### Grammar needs update

The EBNF should document both forms:

```
<exp> ::= <LET> <letsymlist> <exp>     -- scoped let-expression
        | <LET> <TAG> <EQ> <exp>        -- statement-style let (current scope)
```
