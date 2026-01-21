//! Built-in objects and functions for FMPL.

pub mod curl;
pub mod io;
pub mod sse;

pub use curl::CurlBuiltin;
pub use io::{EnvBuiltin, IoBuiltin};
pub use sse::SseBuiltin;
