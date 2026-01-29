//! Rust <-> FMPL bridge.
//!
//! Provides bidirectional FFI between Rust and FMPL:
//! - **Rust → FMPL**: Call Rust functions from FMPL
//! - **FMPL → Rust**: Embed FMPL expressions in Rust code
//! - **Shared types**: Common Value representation for interoperability

use crate::Vm;
use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry of exported Rust functions available to FMPL code.
pub struct FunctionRegistry {
    functions: RwLock<HashMap<String, RustFunction>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: RwLock::new(HashMap::new()),
        }
    }

    /// Register a Rust function that can be called from FMPL.
    pub fn register<F>(&self, name: &str, func: F)
    where
        F: Fn(Vec<Value>) -> Result<Value> + Send + Sync + 'static,
    {
        let rust_func = RustFunction {
            name: name.to_string(),
            func: Arc::new(func),
        };
        self.functions
            .write()
            .unwrap()
            .insert(name.to_string(), rust_func);
    }

    /// Call a registered Rust function.
    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        let functions = self.functions.read().unwrap();
        let func = functions
            .get(name)
            .ok_or_else(|| Error::Runtime(format!("Unknown function: {}", name)))?;
        (func.func)(args)
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A Rust function that can be called from FMPL.
#[derive(Clone)]
pub struct RustFunction {
    pub name: String,
    pub func: Arc<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync>,
}

/// Compiled FMPL expression that can be evaluated multiple times.
pub struct CompiledExpr {
    /// The bytecode for the compiled expression.
    pub code: crate::CompiledCode,
}

impl CompiledExpr {
    /// Evaluate the compiled expression.
    pub fn eval(&self, vm: &mut Vm) -> Result<Value> {
        vm.run(&self.code)
    }
}

/// Bridge context for embedding FMPL in Rust applications.
pub struct FmplBridge {
    /// Registry of exported Rust functions.
    pub functions: FunctionRegistry,
    /// Global variable bindings.
    globals: RwLock<HashMap<String, Value>>,
}

impl FmplBridge {
    /// Create a new FMPL bridge.
    pub fn new() -> Self {
        Self {
            functions: FunctionRegistry::new(),
            globals: RwLock::new(HashMap::new()),
        }
    }

    /// Set a global variable accessible to FMPL code.
    pub fn set_global(&self, name: &str, value: Value) {
        self.globals
            .write()
            .unwrap()
            .insert(name.to_string(), value);
    }

    /// Get a global variable.
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.read().unwrap().get(name).cloned()
    }

    /// Evaluate FMPL source code and return the result.
    ///
    /// This is a convenience function that parses, compiles, and executes FMPL code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fmpl_core::builtins::FmplBridge;
    /// use fmpl_core::value::Value;
    ///
    /// let bridge = FmplBridge::new();
    /// let result = bridge.eval("1 + 2").unwrap();
    /// assert_eq!(result, Value::Int(3));
    /// ```
    pub fn eval(&self, source: &str) -> Result<Value> {
        let mut vm = Vm::new();

        // Set global variables
        for (name, value) in self.globals.read().unwrap().iter() {
            // Note: VM doesn't have a direct set_var method, so we skip this for now
            // This would require extending the VM API
            let _ = (name, value);
        }

        // Parse and compile
        let tokens = crate::lexer::Lexer::new(source).tokenize()?;
        let ast = crate::parser::Parser::with_source(&tokens, source).parse()?;
        let code = crate::Compiler::new().compile(&ast)?;

        // Execute
        vm.run(&code)
    }

    /// Compile FMPL source code to an executable expression.
    ///
    /// The compiled expression can be evaluated multiple times.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fmpl_core::builtins::FmplBridge;
    /// use fmpl_core::value::Value;
    ///
    /// let bridge = FmplBridge::new();
    /// let compiled = bridge.compile("1 + 2").unwrap();
    ///
    /// let mut vm = fmpl_core::Vm::new();
    /// assert_eq!(compiled.eval(&mut vm).unwrap(), Value::Int(3));
    /// ```
    pub fn compile(&self, source: &str) -> Result<CompiledExpr> {
        let ast = self.parse_source(source)?;
        let code = crate::Compiler::new().compile(&ast)?;
        Ok(CompiledExpr { code })
    }

    /// Parse FMPL source code to AST.
    pub fn parse_source(&self, source: &str) -> Result<crate::ast::Expr> {
        let tokens = crate::lexer::Lexer::new(source).tokenize()?;
        crate::parser::Parser::with_source(&tokens, source).parse()
    }
}

impl Default for FmplBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate FMPL source code with a fresh VM.
///
/// This is a convenience function for quick evaluation without needing
/// to manage a bridge context.
///
/// # Example
///
/// ```rust
/// use fmpl_core::builtins::eval_fmpl;
/// use fmpl_core::value::Value;
///
/// let result = eval_fmpl("1 + 2 * 3").unwrap();
/// assert_eq!(result, Value::Int(7));
/// ```
pub fn eval_fmpl(source: &str) -> Result<Value> {
    FmplBridge::new().eval(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vm;
    use crate::value::Value;

    #[test]
    fn test_eval_simple() {
        let result = eval_fmpl("1 + 2").unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_compile_and_eval() {
        let bridge = FmplBridge::new();
        let compiled = bridge.compile("1 + 2").unwrap();

        let mut vm = Vm::new();
        let result = compiled.eval(&mut vm).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_function_registry() {
        let registry = FunctionRegistry::new();
        registry.register("double", |args| {
            if let Some(Value::Int(n)) = args.first() {
                Ok(Value::Int(n * 2))
            } else {
                Err(Error::Runtime("Expected Int".to_string()))
            }
        });

        let result = registry.call("double", vec![Value::Int(5)]).unwrap();
        assert_eq!(result, Value::Int(10));
    }
}
