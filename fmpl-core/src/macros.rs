//! Macros for working with list-based AST/IR nodes.
//!
//! FMPL uses a uniform list representation for all program data:
//! `[:NodeType, child1, child2, ...]`
//!
//! These macros make construction and destructuring ergonomic in Rust.

/// Construct a list-based AST/IR node: `[:Tag, child1, child2, ...]`
///
/// # Examples
/// ```ignore
/// ast_node!(Int, Value::Int(42))           // [:Int, 42]
/// ast_node!(Binary, sym("+"), lhs, rhs)    // [:Binary, :+, lhs, rhs]
/// ast_node!(Null)                          // [:Null]
/// ```
#[macro_export]
macro_rules! ast_node {
    ($tag:ident) => {
        $crate::value::Value::List(std::sync::Arc::new(vec![
            $crate::value::Value::Symbol(smol_str::SmolStr::new(stringify!($tag))),
        ]))
    };
    ($tag:ident, $($child:expr),+ $(,)?) => {
        $crate::value::Value::List(std::sync::Arc::new(vec![
            $crate::value::Value::Symbol(smol_str::SmolStr::new(stringify!($tag))),
            $($child),+
        ]))
    };
}

/// Match a list-based AST/IR node and extract children by type.
///
/// Dispatches on the symbol head of a `Value::List`, then binds children
/// by position with type coercion. Available child types:
///
/// - `int`  — extract as `i64` from `Value::Int`
/// - `float` — extract as `f64` from `Value::Float`
/// - `str`  — extract as `SmolStr` from `Value::String`
/// - `sym`  — extract as `SmolStr` from `Value::Symbol`
/// - `bool` — extract as `bool` from `Value::Bool`
/// - `list` — extract as `Vec<Value>` from `Value::List`
/// - `val`  — extract as `Value` (no coercion)
/// - `node` — extract as `Value` and recursively compile (for ir::compile)
///
/// # Examples
/// ```ignore
/// ast_match!(value, self, {
///     [Int, n: int] => Ok(self.emit(Instruction::LoadInt(n))),
///     [Add, lhs: node, rhs: node] => Ok(self.emit(Instruction::Add { lhs, rhs })),
///     [Lambda, params: list, body: node] => { ... },
/// })
/// ```
#[macro_export]
macro_rules! ast_match {
    // Entry point: match a value against node patterns
    ($val:expr, $self_:expr, { $( [$tag:ident $(, $b:ident : $k:ident)*] => $body:expr ),+ $(,)? }) => {{
        let __val = $val;
        match __val {
            $crate::value::Value::List(ref __items) if !__items.is_empty() => {
                if let $crate::value::Value::Symbol(ref __tag) = __items[0] {
                    match __tag.as_str() {
                        $(
                            stringify!($tag) => {
                                let __children = &__items[1..];
                                ast_match!(@bind __children, $self_, 0, $($b : $k,)* => $body)
                            }
                        ),+
                        __other => Err($crate::error::Error::Runtime(
                            format!("Unknown node: {}", __other)
                        ))
                    }
                } else {
                    Err($crate::error::Error::Runtime(
                        "Expected symbol as list head".into()
                    ))
                }
            }
            _ => Err($crate::error::Error::Runtime(
                format!("Expected list node, got {:?}", __val)
            ))
        }
    }};

    // Terminal: no more bindings, evaluate body
    (@bind $children:ident, $self_:expr, $idx:expr, => $body:expr) => {
        $body
    };

    // Extract i64 from Value::Int
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : int, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::Int(n) => *n,
            other => return Err($crate::error::Error::Runtime(
                format!("Expected int at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract f64 from Value::Float
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : float, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::Float(n) => *n,
            other => return Err($crate::error::Error::Runtime(
                format!("Expected float at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract SmolStr from Value::String
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : str, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::String(s) => s.clone(),
            other => return Err($crate::error::Error::Runtime(
                format!("Expected string at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract SmolStr from Value::Symbol (or Value::String for flexibility)
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : sym, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::Symbol(s) => s.clone(),
            $crate::value::Value::String(s) => s.clone(),
            other => return Err($crate::error::Error::Runtime(
                format!("Expected symbol at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract bool from Value::Bool
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : bool, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::Bool(b) => *b,
            other => return Err($crate::error::Error::Runtime(
                format!("Expected bool at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract Vec<Value> from Value::List
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : list, $($rest:tt)*) => {{
        let $name = match &$children[$idx] {
            $crate::value::Value::List(items) => items.as_ref().clone(),
            other => return Err($crate::error::Error::Runtime(
                format!("Expected list at position {}, got {:?}", $idx, other)
            )),
        };
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Extract raw Value (no coercion)
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : val, $($rest:tt)*) => {{
        let $name = $children[$idx].clone();
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};

    // Recursively compile child node (for ir::compile)
    (@bind $children:ident, $self_:expr, $idx:expr, $name:ident : node, $($rest:tt)*) => {{
        let $name = $self_.compile_ir(&$children[$idx])?;
        ast_match!(@bind $children, $self_, $idx + 1, $($rest)*)
    }};
}
