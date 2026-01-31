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
//!
//! NOTE: fmpl-bootstrap always uses the legacy parser since it's used
//! to generate the parser, avoiding circular dependencies.

use fmpl_core::compiler::Compiler;
use fmpl_core::error::Result;
use fmpl_core::lexer::Lexer;
use fmpl_core::parser::Parser;
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
        let result = eval_legacy(&mut vm, code)?;
        output_result(&result);
        return Ok(());
    }

    // Load and execute file
    let file_path = &args[1];
    let code = fs::read_to_string(file_path).map_err(|e| {
        fmpl_core::error::Error::Runtime(format!("Failed to read {}: {}", file_path, e))
    })?;

    let result = eval_legacy(&mut vm, &code)?;
    output_result(&result);

    Ok(())
}

/// Evaluate using the legacy parser (always used by fmpl-bootstrap)
fn eval_legacy(vm: &mut Vm, source: &str) -> Result<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast = Parser::with_source(&tokens, source).parse()?;
    let code = Compiler::new().compile(&ast)?;
    vm.run(&code)
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
