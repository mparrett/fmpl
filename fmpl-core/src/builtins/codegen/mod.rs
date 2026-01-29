//! Code generation framework for FMPL.
//!
//! Provides a clean separation between:
//! - IR transpilation (IR → Rust expressions)
//! - Code generation for different targets (parser, standalone, etc.)
//!
//! ## Architecture
//!
//! ```text
//! IR (Value)
//!   ↓
//! Transpiler (context-free)
//!   ↓
//! Rust expression (String)
//!   ↓
//! CodeGenerator (target-specific)
//!   ↓
//! Complete Rust code
//! ```

pub mod context;

pub use context::{CodegenContext, CodegenTarget, RustBuilder};
