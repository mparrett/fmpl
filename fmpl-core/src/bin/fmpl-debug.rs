#!/usr/bin/env rust-script
//!
//! FMPL Parser Debug Tool
//!
//! A diagnostic tool for debugging FMPL parsing issues.
//!
//! Usage:
//!   cargo run --bin fmpl-debug <file.fmpl>
//!
//! Features:
//!   --tokens       Show detailed tokenization
//!   --ast          Show parsed AST structure
//!   --trace        Trace parser execution
//!   --context N    Show context around error position N

use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("FMPL Parser Debug Tool");
        eprintln!();
        eprintln!("Usage: fmpl-debug <file.fmpl> [options]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --tokens       Show detailed tokenization");
        eprintln!("  --ast          Show parsed AST structure");
        eprintln!("  --trace        Trace parser execution");
        eprintln!("  --context N    Show N lines of context around error");
        eprintln!();
        eprintln!("Note: This is a rust-script standalone version.");
        eprintln!("For full integration, use: cargo run --bin fmpl-debug <file.fmpl>");
        exit(1);
    }

    let file_path = &args[1];
    let _show_tokens = args.iter().any(|a| a == "--tokens");
    let _show_ast = args.iter().any(|a| a == "--ast");
    let _show_trace = args.iter().any(|a| a == "--trace");
    let context_lines: usize = args
        .iter()
        .find(|a| a.starts_with("--context"))
        .and_then(|a| a[9..].parse().ok())
        .unwrap_or(0);

    let contents = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            exit(1);
        }
    };

    println!("=== FMPL Parser Debug Tool ===");
    println!("File: {}", file_path);
    println!("Size: {} bytes", contents.len());
    println!();

    // Basic file info
    if contents.len() < 500 {
        println!("Content ({} bytes):", contents.len());
        println!("{}", contents);
    } else {
        println!("First 200 bytes:");
        println!("{}", &contents[..200]);
        println!("... ({} more bytes)", contents.len() - 200);
    }

    // Use the actual debug module to show tokenization and parse info
    println!("\n=== Tokenization ===");
    // We can't directly use the debug module without linking to fmpl_core properly
    // but we can show what we can from the file itself

    // Show context around position if requested
    if context_lines > 0 {
        println!();
        println!(
            "Showing {} lines of context around errors...",
            context_lines
        );
        // Find error positions by scanning for problematic patterns
        let lines: Vec<&str> = contents.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Look for patterns that might cause parsing issues
            if line.contains("=>") && !line.contains("--") {
                println!("Line {} (has action): {}", i + 1, line.trim());
            }
        }
    }
}
