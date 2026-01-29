# IR to Rust Refactoring Summary

## Problem

The original `ir_to_rust.rs` had a `is_grammar_mode` flag to handle two different Value types:
- **fmpl_core::Value**: Methods return `Result<Value>` (can fail)
- **RUNTIME_PRELUDE Value**: Methods return `Value` directly (panic on error)

This mixed concerns and made the code harder to maintain.

## Solution

Created a clean separation with new modules:

### 1. `codegen/context.rs` - Code Generation Context

```rust
pub enum CodegenTarget {
    Parser,      // fmpl-core integration (Result<Value>)
    Standalone,  // Standalone programs (Value)
}

pub struct CodegenContext {
    pub target: CodegenTarget,
    pub debug: bool,
    pub indent: usize,
}

impl CodegenContext {
    pub fn unwrap_call(&self) -> &'static str {
        if self.is_parser() { ".unwrap()" } else { "" }
    }
}
```

### 2. `runtime/value_ops.rs` - Trait-Based Operations

```rust
pub trait FallibleOps {
    fn add(&self, other: &Self) -> Result<Self> where Self: Sized;
    // ... other operations
}

pub trait InfallibleOps {
    fn add(&self, other: &Self) -> Self where Self: Sized;
    // ... other operations
}

impl FallibleOps for Value { /* use existing methods */ }
```

### 3. `bridge.rs` - Rust <-> FMPL Bridge

```rust
pub struct FmplBridge {
    pub functions: FunctionRegistry,
    globals: RwLock<HashMap<String, Value>>,
}

impl FmplBridge {
    pub fn eval(&self, source: &str) -> Result<Value>;
    pub fn compile(&self, source: &str) -> Result<CompiledExpr>;
}

pub fn eval_fmpl(source: &str) -> Result<Value>;
```

## Usage Example

```rust
// Old way (with flag)
let mut transpiler = IrToRust::new_grammar_mode();
let code = transpiler.transpile_ir(ir)?;

// New way (with context)
let ctx = CodegenContext::parser();
let code = transpile_with_context(ir, &ctx)?;
```

## Benefits

1. **Separation of concerns**: IR transpilation is context-free
2. **Extensibility**: Easy to add new targets (WebAssembly, etc.)
3. **Type safety**: Traits encode operation contracts
4. **FFI support**: Bridge enables Rust <-> FMPL interop
5. **Testability**: Each module can be tested independently

## Next Steps

1. Migrate `ir_to_rust.rs` to use `CodegenContext`
2. Remove `is_grammar_mode` flag from `IrToRust`
3. Extend bridge with variable binding support
4. Add more target languages as needed
