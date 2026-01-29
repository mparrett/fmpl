//! FMPL Bootstrap CLI
//!
//! Runs FMPL code and outputs results to stdout.
//! Used by build.rs to generate parser code at build time.
//!
//! Usage:
//!   fmpl-bootstrap <file.fmpl>
//!   fmpl-bootstrap -e '<expression>'
//!
//! The output is the result of evaluating the FMPL code,
//! typically Rust source code for the generated parser.

use fmpl_core::error::Result;
use fmpl_core::eval;
use fmpl_core::value::Value;
use fmpl_core::vm::Vm;
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    if let Err(e) = run() {
        eprintln!("fmpl-bootstrap error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fmpl-bootstrap <file.fmpl>");
        eprintln!("       fmpl-bootstrap -e '<expression>'");
        std::process::exit(1);
    }

    let mut vm = Vm::new();

    // Handle -e flag for direct expression evaluation
    if args[1] == "-e" {
        if args.len() < 3 {
            eprintln!("Error: -e requires an expression argument");
            std::process::exit(1);
        }
        let code = &args[2];
        let result = eval(&mut vm, code)?;
        output_result(&result);
        return Ok(());
    }

    // Load and execute file
    let file_path = &args[1];
    let code = fs::read_to_string(file_path).map_err(|e| {
        fmpl_core::error::Error::Runtime(format!("Failed to read {}: {}", file_path, e))
    })?;

    let result = eval(&mut vm, &code)?;
    output_result(&result);

    Ok(())
}

/// Output result to stdout. Strings are printed directly, other values use debug format.
fn output_result(result: &Value) {
    match result {
        Value::String(s) => {
            print!("{}", s);
        }
        _ => {
            print!("{:?}", result);
        }
    }
    io::stdout().flush().unwrap();
}
