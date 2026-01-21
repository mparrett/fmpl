//! Built-in objects and functions for FMPL.

pub mod curl;
pub mod io;

pub use curl::CurlBuiltin;
pub use io::{EnvBuiltin, IoBuiltin};
