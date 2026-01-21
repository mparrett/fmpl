//! Built-in objects and functions for FMPL.

pub mod curl;
pub mod llm;

pub use curl::CurlBuiltin;
pub use llm::{init_llm, llm_chat};
