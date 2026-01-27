# First-Class AST and IR Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable full metaprogramming by exposing AST and IR as first-class values that tree grammars can transform.

**Architecture:** Add `Value::Tagged` for constructor values, implement `Pattern::Constructor` matching, add `Value::Code` for compiled bytecode, and provide `parse`/`compile`/`eval` builtins.

**Tech Stack:** Rust, FMPL VM, existing grammar system

---

## Task 1: Add Value::Tagged

**Files:**
- Modify: `fmpl-core/src/value.rs:16-58` (Value enum)
- Modify: `fmpl-core/src/value.rs:242-265` (type_name)
- Modify: `fmpl-core/src/value.rs:554-597` (Display)
- Test: `fmpl-core/src/value.rs` (inline tests)

**Step 1: Write the failing test**

Add to `fmpl-core/src/value.rs` in the `#[cfg(test)]` section:

```rust
#[test]
fn test_tagged_value_type_name() {
    let val = Value::Tagged(SmolStr::new("Binary"), Arc::new(vec![
        Value::Symbol(SmolStr::new("+")),
        Value::Int(1),
        Value::Int(2),
    ]));
    assert_eq!(val.type_name(), "tagged");
}

#[test]
fn test_tagged_value_display() {
    let val = Value::Tagged(SmolStr::new("Int"), Arc::new(vec![Value::Int(42)]));
    assert_eq!(format!("{}", val), ":Int(42)");
}

#[test]
fn test_tagged_value_is_truthy() {
    let val = Value::Tagged(SmolStr::new("Foo"), Arc::new(vec![]));
    assert!(val.is_truthy());
}

#[test]
fn test_tagged_value_nested() {
    let inner = Value::Tagged(SmolStr::new("Int"), Arc::new(vec![Value::Int(1)]));
    let outer = Value::Tagged(SmolStr::new("Binary"), Arc::new(vec![
        Value::Symbol(SmolStr::new("+")),
        inner,
        Value::Tagged(SmolStr::new("Int"), Arc::new(vec![Value::Int(2)])),
    ]));
    assert_eq!(format!("{}", outer), ":Binary(:+, :Int(1), :Int(2))");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_tagged_value`
Expected: FAIL with "no variant named `Tagged`"

**Step 3: Add Value::Tagged variant**

In `fmpl-core/src/value.rs`, add after line 30 (`Grammar` variant):

```rust
    /// Tagged/constructor value with symbol name and children.
    Tagged(SmolStr, Arc<Vec<Value>>),
```

**Step 4: Add type_name for Tagged**

In `type_name()` method around line 255, add:

```rust
            Value::Tagged(_, _) => "tagged",
```

**Step 5: Add Display for Tagged**

In `Display` impl around line 586, add:

```rust
            Value::Tagged(tag, children) => {
                write!(f, ":{}", tag)?;
                if !children.is_empty() {
                    write!(f, "(")?;
                    for (i, child) in children.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", child)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
```

**Step 6: Add is_truthy for Tagged**

In `is_truthy()` method around line 239, add before the catch-all:

```rust
            Value::Tagged(_, _) => true,
```

**Step 7: Run tests to verify they pass**

Run: `cargo test -p fmpl-core test_tagged_value`
Expected: PASS (4 tests)

**Step 8: Commit**

```bash
jj describe -m "feat(value): add Value::Tagged for constructor values"
```

---

## Task 2: Add Tagged Value source_repr

**Files:**
- Modify: `fmpl-core/src/repr.rs:531-576` (SourceRepr impl)
- Test: `fmpl-core/src/repr.rs` (inline tests)

**Step 1: Write the failing test**

Add to `fmpl-core/src/repr.rs` tests section:

```rust
#[test]
fn test_tagged_source_repr() {
    let val = Value::Tagged(
        SmolStr::new("Binary"),
        Arc::new(vec![
            Value::Symbol(SmolStr::new("+")),
            Value::Int(1),
            Value::Int(2),
        ])
    );
    assert_eq!(val.source_repr(), ":Binary(:+, 1, 2)");
}

#[test]
fn test_tagged_source_repr_empty() {
    let val = Value::Tagged(SmolStr::new("Null"), Arc::new(vec![]));
    assert_eq!(val.source_repr(), ":Null");
}

#[test]
fn test_tagged_source_repr_nested() {
    let inner = Value::Tagged(SmolStr::new("Int"), Arc::new(vec![Value::Int(42)]));
    let outer = Value::Tagged(SmolStr::new("Call"), Arc::new(vec![
        Value::Symbol(SmolStr::new("print")),
        inner,
    ]));
    assert_eq!(val.source_repr(), ":Call(:print, :Int(42))");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_tagged_source_repr`
Expected: FAIL with pattern match error

**Step 3: Add source_repr for Tagged**

In `fmpl-core/src/repr.rs` `SourceRepr` impl for `Value`, add after the `Grammar` case:

```rust
            Value::Tagged(tag, children) => {
                let mut result = format!(":{}", tag);
                if !children.is_empty() {
                    result.push('(');
                    for (i, child) in children.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&child.source_repr());
                    }
                    result.push(')');
                }
                result
            }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p fmpl-core test_tagged_source_repr`
Expected: PASS

**Step 5: Commit**

```bash
jj describe -m "feat(repr): add source_repr for Value::Tagged"
```

---

## Task 3: Parse Tagged Value Construction Syntax

**Files:**
- Modify: `fmpl-core/src/ast.rs` (add Expr::Tagged)
- Modify: `fmpl-core/src/parser.rs:565-569` (parse after Symbol)
- Test: `fmpl-core/tests/tagged_values.rs` (new file)

**Step 1: Create test file**

Create `fmpl-core/tests/tagged_values.rs`:

```rust
//! Tests for tagged/constructor values.

use fmpl_core::{eval, Vm, Value};
use smol_str::SmolStr;
use std::sync::Arc;

#[test]
fn test_parse_tagged_no_args() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, ":Null").unwrap();
    assert!(matches!(result, Value::Tagged(tag, children)
        if tag == "Null" && children.is_empty()));
}

#[test]
fn test_parse_tagged_single_arg() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, ":Int(42)").unwrap();
    if let Value::Tagged(tag, children) = result {
        assert_eq!(tag.as_str(), "Int");
        assert_eq!(children.len(), 1);
        assert!(matches!(&children[0], Value::Int(42)));
    } else {
        panic!("expected Tagged, got {:?}", result);
    }
}

#[test]
fn test_parse_tagged_multiple_args() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, ":Binary(:+, 1, 2)").unwrap();
    if let Value::Tagged(tag, children) = result {
        assert_eq!(tag.as_str(), "Binary");
        assert_eq!(children.len(), 3);
        assert!(matches!(&children[0], Value::Symbol(s) if s == "+"));
        assert!(matches!(&children[1], Value::Int(1)));
        assert!(matches!(&children[2], Value::Int(2)));
    } else {
        panic!("expected Tagged, got {:?}", result);
    }
}

#[test]
fn test_parse_tagged_nested() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, ":Add(:Int(1), :Int(2))").unwrap();
    if let Value::Tagged(tag, children) = result {
        assert_eq!(tag.as_str(), "Add");
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], Value::Tagged(t, _) if t == "Int"));
        assert!(matches!(&children[1], Value::Tagged(t, _) if t == "Int"));
    } else {
        panic!("expected Tagged, got {:?}", result);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core --test tagged_values`
Expected: FAIL

**Step 3: Add Expr::Tagged to AST**

In `fmpl-core/src/ast.rs`, add to the `Expr` enum:

```rust
    /// Tagged/constructor value: :Tag(args...)
    Tagged(SmolStr, Vec<Expr>),
```

**Step 4: Update parser to handle tagged values**

In `fmpl-core/src/parser.rs`, modify the `Token::Symbol` case in `parse_primary()`:

```rust
Token::Symbol(s) => {
    let s = s.clone();
    self.advance();
    // Check for tagged value: :Symbol(args...)
    if self.check(&Token::LParen) {
        self.advance(); // consume '('
        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            args.push(self.parse_expr()?);
            while self.check(&Token::Comma) {
                self.advance();
                if self.check(&Token::RParen) {
                    break; // trailing comma
                }
                args.push(self.parse_expr()?);
            }
        }
        self.expect(&Token::RParen)?;
        Ok(Expr::Tagged(s, args))
    } else {
        Ok(Expr::Symbol(s))
    }
}
```

**Step 5: Add Expr::Tagged to compiler**

In `fmpl-core/src/compiler.rs`, add to `compile_expr()`:

```rust
Expr::Tagged(tag, args) => {
    let mut arg_indices = Vec::new();
    for arg in args {
        arg_indices.push(self.compile_expr(arg)?);
    }
    Ok(self.code.emit(Instruction::MakeTagged {
        tag: tag.clone(),
        args: arg_indices,
    }))
}
```

**Step 6: Add MakeTagged instruction**

In `fmpl-core/src/compiler.rs`, add to `Instruction` enum:

```rust
    /// Create a tagged value
    MakeTagged { tag: SmolStr, args: Vec<InstrIndex> },
```

**Step 7: Implement MakeTagged in VM**

In `fmpl-core/src/vm.rs`, add handler:

```rust
Instruction::MakeTagged { tag, args } => {
    let children: Vec<Value> = args.iter()
        .map(|idx| frame.get(*idx).clone())
        .collect();
    frame.set_current(Value::Tagged(tag.clone(), Arc::new(children)));
}
```

**Step 8: Add Display for Expr::Tagged**

In `fmpl-core/src/repr.rs`, add to `Display` impl for `Expr`:

```rust
Expr::Tagged(tag, args) => {
    write!(f, ":{}", tag)?;
    if !args.is_empty() {
        write!(f, "(")?;
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")?;
    }
    Ok(())
}
```

**Step 9: Run tests to verify they pass**

Run: `cargo test -p fmpl-core --test tagged_values`
Expected: PASS

**Step 10: Commit**

```bash
jj describe -m "feat(parser): add :Tag(args...) syntax for tagged values"
```

---

## Task 4: Implement Pattern::Constructor Matching

**Files:**
- Modify: `fmpl-core/src/compiler.rs:2062-2098` (compile_pattern_binding)
- Modify: `fmpl-core/src/compiler.rs:58-249` (add instructions)
- Modify: `fmpl-core/src/vm.rs` (add instruction handlers)
- Test: `fmpl-core/tests/tagged_values.rs` (add pattern tests)

**Step 1: Add pattern matching tests**

Add to `fmpl-core/tests/tagged_values.rs`:

```rust
#[test]
fn test_tagged_pattern_match_simple() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#":Int(42) @ { :Int(n) => n }"#).unwrap();
    assert!(matches!(result, Value::Int(42)));
}

#[test]
fn test_tagged_pattern_match_nested() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        :Binary(:+, :Int(1), :Int(2)) @ {
            :Binary(op, :Int(a), :Int(b)) => [op, a, b]
        }
    "#).unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], Value::Symbol(s) if s == "+"));
        assert!(matches!(&items[1], Value::Int(1)));
        assert!(matches!(&items[2], Value::Int(2)));
    } else {
        panic!("expected list, got {:?}", result);
    }
}

#[test]
fn test_tagged_pattern_match_choice() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        :Int(42) @ {
            :String(s) => "string"
            :Int(n) => "int"
            _ => "other"
        }
    "#).unwrap();
    assert!(matches!(result, Value::String(s) if s == "int"));
}

#[test]
fn test_tagged_let_destructure() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let (:Binary(op, lhs, rhs) = :Binary(:+, 1, 2)) in
        [op, lhs, rhs]
    "#).unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
    } else {
        panic!("expected list, got {:?}", result);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core --test tagged_values test_tagged_pattern`
Expected: FAIL

**Step 3: Add extraction instructions**

In `fmpl-core/src/compiler.rs` `Instruction` enum, add:

```rust
    /// Check if value is a Tagged with expected tag, jump to fail_target if not
    MatchTag { value: InstrIndex, tag: SmolStr, fail_target: InstrIndex },
    /// Extract the tag from a Tagged value (returns Symbol)
    ExtractTag { source: InstrIndex },
    /// Extract child at index from Tagged value
    ExtractTaggedChild { source: InstrIndex, index: usize },
```

**Step 4: Implement pattern binding for Constructor**

In `fmpl-core/src/compiler.rs` `compile_pattern_binding()`, add case:

```rust
Pattern::Constructor(tag, patterns) => {
    // For now, just extract children positionally
    // Note: this doesn't do runtime tag checking in let bindings
    // Full matching with tag check happens in @ blocks via MatchTag
    for (i, pat) in patterns.iter().enumerate() {
        let extracted = self.code.emit(Instruction::ExtractTaggedChild {
            source,
            index: i,
        });
        self.compile_pattern_binding(pat, extracted)?;
    }
}
```

**Step 5: Update grammar pattern matching for tagged**

In `fmpl-core/src/compiler.rs`, find where `GrammarPattern::MapMatch` is compiled and add similar handling for tagged patterns. Look for `compile_grammar_pattern` and add:

```rust
GP::TagMatch(tag, patterns) => {
    // Check tag matches
    let tag_check = self.code.emit(Instruction::MatchTag {
        value: current_idx,
        tag: tag.clone(),
        fail_target: InstrIndex(0), // placeholder
    });

    // Extract and match children
    for (i, pattern) in patterns.iter().enumerate() {
        let child = self.code.emit(Instruction::ExtractTaggedChild {
            source: current_idx,
            index: i,
        });
        self.compile_grammar_pattern(pattern, child)?;
    }

    // Patch fail target
    let end_idx = self.code.next_index();
    self.code.patch_jump_target(tag_check, end_idx);

    Ok(current_idx)
}
```

**Step 6: Add GP::TagMatch to grammar Pattern**

In `fmpl-core/src/grammar/mod.rs`, add to `Pattern` enum:

```rust
    /// Match a tagged value with specific tag and child patterns.
    TagMatch(SmolStr, Vec<Pattern>),
```

**Step 7: Implement VM instruction handlers**

In `fmpl-core/src/vm.rs`, add handlers:

```rust
Instruction::MatchTag { value, tag, fail_target } => {
    let val = frame.get(value);
    let matches = match val {
        Value::Tagged(t, _) => t == tag,
        _ => false,
    };
    if !matches {
        frame.ip = fail_target.as_usize();
        continue;
    }
    frame.set_current(Value::Bool(true));
}

Instruction::ExtractTag { source } => {
    let val = frame.get(source);
    match val {
        Value::Tagged(tag, _) => {
            frame.set_current(Value::Symbol(tag.clone()));
        }
        _ => {
            return Err(Error::Runtime(format!(
                "ExtractTag expected tagged value, got {}",
                val.type_name()
            )));
        }
    }
}

Instruction::ExtractTaggedChild { source, index } => {
    let val = frame.get(source);
    match val {
        Value::Tagged(_, children) => {
            let child = children.get(*index).cloned().unwrap_or(Value::Null);
            frame.set_current(child);
        }
        _ => {
            return Err(Error::Runtime(format!(
                "ExtractTaggedChild expected tagged value, got {}",
                val.type_name()
            )));
        }
    }
}
```

**Step 8: Run tests to verify they pass**

Run: `cargo test -p fmpl-core --test tagged_values`
Expected: PASS

**Step 9: Commit**

```bash
jj describe -m "feat(pattern): implement Pattern::Constructor matching for tagged values"
```

---

## Task 5: Add Value::Code for Compiled Bytecode

**Files:**
- Modify: `fmpl-core/src/value.rs` (add Value::Code)
- Modify: `fmpl-core/src/compiler.rs` (add CompiledCode struct if needed)
- Test: `fmpl-core/src/value.rs` (inline tests)

**Step 1: Write the failing test**

Add to `fmpl-core/src/value.rs` tests:

```rust
#[test]
fn test_code_value_type_name() {
    use crate::compiler::CompiledCode;
    let code = CompiledCode::new(vec![], vec![]);
    let val = Value::Code(Arc::new(code));
    assert_eq!(val.type_name(), "code");
}

#[test]
fn test_code_value_display() {
    use crate::compiler::CompiledCode;
    let code = CompiledCode::new(vec![], vec![]);
    let val = Value::Code(Arc::new(code));
    assert_eq!(format!("{}", val), "<code>");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core test_code_value`
Expected: FAIL

**Step 3: Add Value::Code variant**

In `fmpl-core/src/value.rs`, add:

```rust
    /// Compiled bytecode (opaque, executable).
    Code(Arc<CompiledCode>),
```

Add import at top:
```rust
use crate::compiler::CompiledCode;
```

**Step 4: Add type_name and Display**

```rust
// In type_name()
Value::Code(_) => "code",

// In Display
Value::Code(_) => write!(f, "<code>"),
```

**Step 5: Add is_truthy**

```rust
Value::Code(_) => true,
```

**Step 6: Run tests to verify they pass**

Run: `cargo test -p fmpl-core test_code_value`
Expected: PASS

**Step 7: Commit**

```bash
jj describe -m "feat(value): add Value::Code for compiled bytecode"
```

---

## Task 6: Add parse Builtin

**Files:**
- Create: `fmpl-core/src/builtins/ast.rs`
- Modify: `fmpl-core/src/builtins/mod.rs`
- Modify: `fmpl-core/src/vm.rs` (builtin dispatch)
- Test: `fmpl-core/tests/metaprogramming.rs` (new file)

**Step 1: Create test file**

Create `fmpl-core/tests/metaprogramming.rs`:

```rust
//! Tests for metaprogramming builtins: parse, compile, eval.

use fmpl_core::{eval, Vm, Value};

#[test]
fn test_parse_int_literal() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"ast::parse("42")"#).unwrap();
    assert!(matches!(result, Value::Tagged(tag, _) if tag == "Int"));
}

#[test]
fn test_parse_binary_expr() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"ast::parse("1 + 2")"#).unwrap();
    if let Value::Tagged(tag, children) = result {
        assert_eq!(tag.as_str(), "Binary");
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected Tagged, got {:?}", result);
    }
}

#[test]
fn test_parse_lambda() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"ast::parse("\\x x + 1")"#).unwrap();
    assert!(matches!(result, Value::Tagged(tag, _) if tag == "Lambda"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core --test metaprogramming`
Expected: FAIL

**Step 3: Create ast.rs builtin module**

Create `fmpl-core/src/builtins/ast.rs`:

```rust
//! AST manipulation builtins.

use crate::ast::Expr;
use crate::error::{Error, Result};
use crate::parser::Parser;
use crate::value::Value;
use smol_str::SmolStr;
use std::sync::Arc;

/// Convert an Expr AST node to a Value::Tagged representation.
pub fn expr_to_value(expr: &Expr) -> Value {
    match expr {
        Expr::Int(n) => Value::Tagged(
            SmolStr::new("Int"),
            Arc::new(vec![Value::Int(*n)]),
        ),
        Expr::Float(n) => Value::Tagged(
            SmolStr::new("Float"),
            Arc::new(vec![Value::Float(*n)]),
        ),
        Expr::String(s) => Value::Tagged(
            SmolStr::new("String"),
            Arc::new(vec![Value::String(s.clone())]),
        ),
        Expr::Symbol(s) => Value::Tagged(
            SmolStr::new("Symbol"),
            Arc::new(vec![Value::Symbol(s.clone())]),
        ),
        Expr::Bool(b) => Value::Tagged(
            SmolStr::new("Bool"),
            Arc::new(vec![Value::Bool(*b)]),
        ),
        Expr::Null => Value::Tagged(
            SmolStr::new("Null"),
            Arc::new(vec![]),
        ),
        Expr::Ident(name) => Value::Tagged(
            SmolStr::new("Var"),
            Arc::new(vec![Value::Symbol(name.clone())]),
        ),
        Expr::Binary(lhs, op, rhs) => Value::Tagged(
            SmolStr::new("Binary"),
            Arc::new(vec![
                Value::Symbol(SmolStr::new(format!("{}", op))),
                expr_to_value(lhs),
                expr_to_value(rhs),
            ]),
        ),
        Expr::Unary(op, e) => Value::Tagged(
            SmolStr::new("Unary"),
            Arc::new(vec![
                Value::Symbol(SmolStr::new(format!("{}", op))),
                expr_to_value(e),
            ]),
        ),
        Expr::Lambda(params, body) => Value::Tagged(
            SmolStr::new("Lambda"),
            Arc::new(vec![
                Value::List(Arc::new(
                    params.iter().map(|p| Value::Symbol(p.clone())).collect()
                )),
                expr_to_value(body),
            ]),
        ),
        Expr::Call(func, args) => Value::Tagged(
            SmolStr::new("Call"),
            Arc::new(vec![
                expr_to_value(func),
                Value::List(Arc::new(
                    args.iter().map(|a| expr_to_value(&a.value)).collect()
                )),
            ]),
        ),
        Expr::If(cond, then_branch, else_branch) => Value::Tagged(
            SmolStr::new("If"),
            Arc::new(vec![
                expr_to_value(cond),
                expr_to_value(then_branch),
                else_branch.as_ref()
                    .map(|e| expr_to_value(e))
                    .unwrap_or(Value::Null),
            ]),
        ),
        Expr::Let(bindings, body) => {
            let binding_values: Vec<Value> = bindings.iter().map(|b| {
                match b {
                    crate::ast::LetBinding::Simple(name, expr) => Value::Tagged(
                        SmolStr::new("Binding"),
                        Arc::new(vec![
                            Value::Symbol(name.clone()),
                            expr_to_value(expr),
                        ]),
                    ),
                    crate::ast::LetBinding::Destructure(pat, expr) => Value::Tagged(
                        SmolStr::new("Destructure"),
                        Arc::new(vec![
                            pattern_to_value(pat),
                            expr_to_value(expr),
                        ]),
                    ),
                }
            }).collect();
            Value::Tagged(
                SmolStr::new("Let"),
                Arc::new(vec![
                    Value::List(Arc::new(binding_values)),
                    expr_to_value(body),
                ]),
            )
        }
        // Add more cases as needed...
        _ => Value::Tagged(
            SmolStr::new("Unknown"),
            Arc::new(vec![Value::String(SmolStr::new(format!("{:?}", expr)))]),
        ),
    }
}

fn pattern_to_value(pat: &crate::ast::Pattern) -> Value {
    use crate::ast::Pattern;
    match pat {
        Pattern::Var(name) => Value::Tagged(
            SmolStr::new("PatVar"),
            Arc::new(vec![Value::Symbol(name.clone())]),
        ),
        Pattern::Wildcard => Value::Tagged(
            SmolStr::new("PatWildcard"),
            Arc::new(vec![]),
        ),
        Pattern::Int(n) => Value::Tagged(
            SmolStr::new("PatInt"),
            Arc::new(vec![Value::Int(*n)]),
        ),
        Pattern::Map(entries) => Value::Tagged(
            SmolStr::new("PatMap"),
            Arc::new(entries.iter().map(|(k, v)| {
                Value::List(Arc::new(vec![
                    Value::Symbol(k.clone()),
                    pattern_to_value(v),
                ]))
            }).collect()),
        ),
        Pattern::Constructor(tag, pats) => Value::Tagged(
            SmolStr::new("PatConstructor"),
            Arc::new(vec![
                Value::Symbol(tag.clone()),
                Value::List(Arc::new(pats.iter().map(pattern_to_value).collect())),
            ]),
        ),
        _ => Value::Tagged(
            SmolStr::new("PatUnknown"),
            Arc::new(vec![]),
        ),
    }
}

/// Parse FMPL source code and return AST as tagged values.
pub fn parse(source: &str) -> Result<Value> {
    let mut parser = Parser::new(source)?;
    let expr = parser.parse_expr()?;
    Ok(expr_to_value(&expr))
}
```

**Step 4: Register in builtins/mod.rs**

In `fmpl-core/src/builtins/mod.rs`, add:

```rust
pub mod ast;
```

**Step 5: Add builtin dispatch in VM**

In `fmpl-core/src/vm.rs`, find the builtin dispatch section and add:

```rust
("__builtin_ast", "parse") => {
    let source = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => return Err(Error::Runtime("ast::parse requires string argument".to_string())),
    };
    crate::builtins::ast::parse(source)
}
```

**Step 6: Add compiler rewriting for ast::parse**

In `fmpl-core/src/compiler.rs`, find where `json::parse` is rewritten and add:

```rust
if module == "ast" && method == "parse" {
    // Rewrite ast::parse(source) to __builtin_ast.parse(source)
    let receiver = self.code.emit(Instruction::LoadVar(SmolStr::new("__builtin_ast")));
    // ... rest of method call compilation
}
```

**Step 7: Run tests to verify they pass**

Run: `cargo test -p fmpl-core --test metaprogramming`
Expected: PASS

**Step 8: Commit**

```bash
jj describe -m "feat(builtin): add ast::parse to convert source to tagged AST"
```

---

## Task 7: Add compile Builtin

**Files:**
- Create: `fmpl-core/src/builtins/ir.rs`
- Modify: `fmpl-core/src/builtins/mod.rs`
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/tests/metaprogramming.rs`

**Step 1: Add tests**

Add to `fmpl-core/tests/metaprogramming.rs`:

```rust
#[test]
fn test_compile_load_int() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"ir::compile(:LoadInt(42))"#).unwrap();
    assert!(matches!(result, Value::Code(_)));
}

#[test]
fn test_compile_add() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"ir::compile(:Add(:LoadInt(1), :LoadInt(2)))"#).unwrap();
    assert!(matches!(result, Value::Code(_)));
}

#[test]
fn test_compile_let() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        ir::compile(:Let(:x, :LoadInt(42), :Var(:x)))
    "#).unwrap();
    assert!(matches!(result, Value::Code(_)));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core --test metaprogramming test_compile`
Expected: FAIL

**Step 3: Create ir.rs builtin module**

Create `fmpl-core/src/builtins/ir.rs`:

```rust
//! IR compilation builtins.

use crate::compiler::{CompiledCode, Instruction, InstrIndex};
use crate::error::{Error, Result};
use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

/// Compile IR (tagged values) to executable bytecode.
pub fn compile(ir: &Value) -> Result<Value> {
    let mut compiler = IrCompiler::new();
    compiler.compile_ir(ir)?;
    Ok(Value::Code(Arc::new(compiler.finish())))
}

struct IrCompiler {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    bindings: HashMap<SmolStr, InstrIndex>,
}

impl IrCompiler {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            bindings: HashMap::new(),
        }
    }

    fn emit(&mut self, instr: Instruction) -> InstrIndex {
        let idx = InstrIndex(self.instructions.len());
        self.instructions.push(instr);
        idx
    }

    fn compile_ir(&mut self, ir: &Value) -> Result<InstrIndex> {
        match ir {
            Value::Tagged(tag, children) => {
                self.compile_tagged(tag.as_str(), children)
            }
            _ => Err(Error::Runtime(format!(
                "IR compile expected tagged value, got {}",
                ir.type_name()
            ))),
        }
    }

    fn compile_tagged(&mut self, tag: &str, children: &[Value]) -> Result<InstrIndex> {
        match tag {
            "LoadNull" => Ok(self.emit(Instruction::LoadNull)),
            "LoadBool" => {
                let b = self.expect_bool(&children[0])?;
                Ok(self.emit(Instruction::LoadBool(b)))
            }
            "LoadInt" => {
                let n = self.expect_int(&children[0])?;
                Ok(self.emit(Instruction::LoadInt(n)))
            }
            "LoadFloat" => {
                let n = self.expect_float(&children[0])?;
                Ok(self.emit(Instruction::LoadFloat(n)))
            }
            "LoadString" => {
                let s = self.expect_string(&children[0])?;
                Ok(self.emit(Instruction::LoadString(s)))
            }
            "LoadVar" => {
                let name = self.expect_symbol(&children[0])?;
                Ok(self.emit(Instruction::LoadVar(name)))
            }
            "Var" => {
                // Reference to a Let-bound variable
                let name = self.expect_symbol(&children[0])?;
                if let Some(idx) = self.bindings.get(&name) {
                    // Just reference the existing value
                    Ok(*idx)
                } else {
                    // Fall back to LoadVar
                    Ok(self.emit(Instruction::LoadVar(name)))
                }
            }
            "Add" => {
                let lhs = self.compile_ir(&children[0])?;
                let rhs = self.compile_ir(&children[1])?;
                Ok(self.emit(Instruction::Add { lhs, rhs }))
            }
            "Sub" => {
                let lhs = self.compile_ir(&children[0])?;
                let rhs = self.compile_ir(&children[1])?;
                Ok(self.emit(Instruction::Sub { lhs, rhs }))
            }
            "Mul" => {
                let lhs = self.compile_ir(&children[0])?;
                let rhs = self.compile_ir(&children[1])?;
                Ok(self.emit(Instruction::Mul { lhs, rhs }))
            }
            "Div" => {
                let lhs = self.compile_ir(&children[0])?;
                let rhs = self.compile_ir(&children[1])?;
                Ok(self.emit(Instruction::Div { lhs, rhs }))
            }
            "Let" => {
                // :Let(:name, :value_ir, :body_ir)
                let name = self.expect_symbol(&children[0])?;
                let value_idx = self.compile_ir(&children[1])?;
                self.bindings.insert(name, value_idx);
                self.compile_ir(&children[2])
            }
            "Seq" => {
                // :Seq([ir1, ir2, ...])
                let items = self.expect_list(&children[0])?;
                let mut last_idx = self.emit(Instruction::LoadNull);
                for item in items {
                    last_idx = self.compile_ir(&item)?;
                }
                Ok(last_idx)
            }
            "If" => {
                let cond = self.compile_ir(&children[0])?;
                // Placeholder for jump
                let jump_if_false = self.emit(Instruction::JumpIfFalse {
                    cond,
                    target: InstrIndex(0),
                });
                let then_idx = self.compile_ir(&children[1])?;
                let jump_to_end = self.emit(Instruction::Jump {
                    target: InstrIndex(0),
                });
                let else_start = InstrIndex(self.instructions.len());
                let else_idx = self.compile_ir(&children[2])?;
                let end = InstrIndex(self.instructions.len());

                // Patch jumps
                if let Instruction::JumpIfFalse { target, .. } = &mut self.instructions[jump_if_false.0] {
                    *target = else_start;
                }
                if let Instruction::Jump { target } = &mut self.instructions[jump_to_end.0] {
                    *target = end;
                }

                Ok(else_idx)
            }
            "Return" => {
                let value = self.compile_ir(&children[0])?;
                Ok(self.emit(Instruction::Return { value }))
            }
            _ => Err(Error::Runtime(format!("Unknown IR node: {}", tag))),
        }
    }

    fn expect_bool(&self, val: &Value) -> Result<bool> {
        match val {
            Value::Bool(b) => Ok(*b),
            _ => Err(Error::Runtime(format!("Expected bool, got {}", val.type_name()))),
        }
    }

    fn expect_int(&self, val: &Value) -> Result<i64> {
        match val {
            Value::Int(n) => Ok(*n),
            _ => Err(Error::Runtime(format!("Expected int, got {}", val.type_name()))),
        }
    }

    fn expect_float(&self, val: &Value) -> Result<f64> {
        match val {
            Value::Float(n) => Ok(*n),
            _ => Err(Error::Runtime(format!("Expected float, got {}", val.type_name()))),
        }
    }

    fn expect_string(&self, val: &Value) -> Result<SmolStr> {
        match val {
            Value::String(s) => Ok(s.clone()),
            _ => Err(Error::Runtime(format!("Expected string, got {}", val.type_name()))),
        }
    }

    fn expect_symbol(&self, val: &Value) -> Result<SmolStr> {
        match val {
            Value::Symbol(s) => Ok(s.clone()),
            _ => Err(Error::Runtime(format!("Expected symbol, got {}", val.type_name()))),
        }
    }

    fn expect_list(&self, val: &Value) -> Result<Vec<Value>> {
        match val {
            Value::List(items) => Ok(items.as_ref().clone()),
            _ => Err(Error::Runtime(format!("Expected list, got {}", val.type_name()))),
        }
    }

    fn finish(self) -> CompiledCode {
        CompiledCode::new(self.instructions, self.constants)
    }
}
```

**Step 4: Register in builtins/mod.rs**

```rust
pub mod ir;
```

**Step 5: Add builtin dispatch**

In VM builtin dispatch:

```rust
("__builtin_ir", "compile") => {
    let ir = args.first().ok_or_else(|| {
        Error::Runtime("ir::compile requires IR argument".to_string())
    })?;
    crate::builtins::ir::compile(ir)
}
```

**Step 6: Add compiler rewriting**

```rust
if module == "ir" && method == "compile" {
    // Rewrite to __builtin_ir.compile
}
```

**Step 7: Run tests**

Run: `cargo test -p fmpl-core --test metaprogramming test_compile`
Expected: PASS

**Step 8: Commit**

```bash
jj describe -m "feat(builtin): add ir::compile to lower IR to bytecode"
```

---

## Task 8: Add eval Builtin

**Files:**
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/tests/metaprogramming.rs`

**Step 1: Add tests**

Add to `fmpl-core/tests/metaprogramming.rs`:

```rust
#[test]
fn test_eval_simple() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let (code = ir::compile(:LoadInt(42))) in
        code::eval(code)
    "#).unwrap();
    assert!(matches!(result, Value::Int(42)));
}

#[test]
fn test_eval_arithmetic() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let (code = ir::compile(:Add(:LoadInt(1), :LoadInt(2)))) in
        code::eval(code)
    "#).unwrap();
    assert!(matches!(result, Value::Int(3)));
}

#[test]
fn test_roundtrip_parse_compile_eval() {
    let mut vm = Vm::new();
    // This requires ast_to_ir grammar, so simplified for now
    let result = eval(&mut vm, r#"
        let (code = ir::compile(:Add(:LoadInt(40), :LoadInt(2)))) in
        code::eval(code)
    "#).unwrap();
    assert!(matches!(result, Value::Int(42)));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p fmpl-core --test metaprogramming test_eval`
Expected: FAIL

**Step 3: Add eval builtin dispatch**

In VM builtin dispatch:

```rust
("__builtin_code", "eval") => {
    let code = match args.first() {
        Some(Value::Code(c)) => c.clone(),
        _ => return Err(Error::Runtime("code::eval requires Code argument".to_string())),
    };
    // Execute the compiled code in current VM context
    self.execute_code(&code)
}
```

**Step 4: Add execute_code method to VM**

```rust
fn execute_code(&mut self, code: &CompiledCode) -> Result<Value> {
    // Create a new frame for this code
    let frame = Frame::new(code.clone(), vec![]);
    self.frames.push(frame);

    // Run until completion
    while !self.frames.is_empty() {
        self.step()?;
    }

    // Return the final value
    Ok(self.last_result.clone().unwrap_or(Value::Null))
}
```

**Step 5: Run tests**

Run: `cargo test -p fmpl-core --test metaprogramming test_eval`
Expected: PASS

**Step 6: Commit**

```bash
jj describe -m "feat(builtin): add code::eval to execute compiled bytecode"
```

---

## Task 9: Integration Tests

**Files:**
- Modify: `fmpl-core/tests/metaprogramming.rs`

**Step 1: Add comprehensive integration tests**

```rust
#[test]
fn test_full_pipeline_simple() {
    let mut vm = Vm::new();
    // Parse source, manually convert to IR, compile, eval
    let result = eval(&mut vm, r#"
        let (ast = ast::parse("42")) in
        let (ir = ast @ { :Int(n) => :LoadInt(n) }) in
        let (code = ir::compile(ir)) in
        code::eval(code)
    "#).unwrap();
    assert!(matches!(result, Value::Int(42)));
}

#[test]
fn test_ast_to_ir_grammar() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        grammar ast_to_ir {
            expr = :Int(n) => :LoadInt(n)
                 | :Binary(:+, lhs, rhs) => :Add(expr(lhs), expr(rhs))
        }

        let (ast = ast::parse("1 + 2")) in
        let (ir = ast @ ast_to_ir.expr) in
        let (code = ir::compile(ir)) in
        code::eval(code)
    "#).unwrap();
    assert!(matches!(result, Value::Int(3)));
}
```

**Step 2: Run all metaprogramming tests**

Run: `cargo test -p fmpl-core --test metaprogramming`
Expected: PASS

**Step 3: Commit**

```bash
jj describe -m "test: add full metaprogramming pipeline integration tests"
```

---

## Summary

After completing all tasks, FMPL will support:

1. **`Value::Tagged`** - Constructor values with symbol names
2. **`:Tag(args...)` syntax** - Parse and construct tagged values
3. **`Pattern::Constructor` matching** - Destructure tagged values in patterns
4. **`Value::Code`** - Opaque compiled bytecode
5. **`ast::parse(source)`** - Parse source to AST (tagged values)
6. **`ir::compile(ir)`** - Compile IR to bytecode
7. **`code::eval(code)`** - Execute compiled code

This enables writing compilers, DSLs, and code generators entirely in FMPL.
