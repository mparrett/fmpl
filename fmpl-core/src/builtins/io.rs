//! File I/O and environment built-ins for FMPL.

use crate::error::{Error, Result};
use crate::value::Value;
use smol_str::SmolStr;

/// The io built-in object for file and environment operations.
pub struct IoBuiltin;

impl IoBuiltin {
    /// Load and evaluate an FMPL file.
    ///
    /// Arguments:
    /// - path: File path to load (string)
    /// - eval_fn: Function to evaluate the loaded FMPL code
    ///
    /// Returns the result of evaluating the file, or an error.
    pub fn load<F>(path: &str, eval_fn: F) -> Result<Value>
    where
        F: FnOnce(&str) -> Result<Value>,
    {
        // Resolve path relative to current directory
        let resolved_path = if std::path::Path::new(path).is_absolute() {
            path.to_string()
        } else {
            // Relative to current working directory
            std::env::current_dir()
                .map_err(|e| Error::Runtime(format!("Failed to get cwd: {}", e)))?
                .join(path)
                .to_str()
                .ok_or_else(|| Error::Runtime("Invalid path".to_string()))?
                .to_string()
        };

        // Read file contents
        let contents = std::fs::read_to_string(&resolved_path).map_err(|e| {
            Error::Runtime(format!("Failed to read file '{}': {}", resolved_path, e))
        })?;

        // Evaluate the FMPL code using the provided evaluator
        eval_fn(&contents)
    }

    /// Get an environment variable.
    ///
    /// Arguments:
    /// - name: Environment variable name (string)
    ///
    /// Returns the variable value as a string, or null if not set.
    pub fn get_env(name: &str) -> Result<Value> {
        match std::env::var(name) {
            Ok(value) => Ok(Value::String(SmolStr::new(value))),
            Err(std::env::VarError::NotPresent) => Ok(Value::Null),
            Err(e) => Err(Error::Runtime(format!(
                "Failed to read env var '{}': {}",
                name, e
            ))),
        }
    }
}

/// The env built-in object for environment variable access.
pub struct EnvBuiltin;

impl EnvBuiltin {
    /// Get an environment variable value.
    pub fn get(name: &str) -> Result<Value> {
        IoBuiltin::get_env(name)
    }
}
