//! Runtime support for code-generated FMPL code.
//!
//! Provides traits and helpers that allow generated code to work with
//! different Value implementations (fmpl_core::Value vs standalone).

pub mod value_ops;

pub use value_ops::{FallibleOps, InfallibleOps};
