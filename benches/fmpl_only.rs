// Direct comparison: FMPL VM vs execution_tape VM (simplified)
// Tests FMPL VM performance

use std::time::Instant;

fn main() {
    println!("=== FMPL VM Performance ===\n");

    // Test cases
    let tests = vec![
        ("7 + 9 * 5", "Arithmetic (7 + 9 * 5 = 52)"),
        ("1 + 2 + 3", "Simple addition"),
        ("10 * 5", "Multiplication"),
        ("100 - 50", "Subtraction"),
    ];

    let iterations = 10000;

    for (source, name) in tests {
        println!("=== Test: {} ===", name);
        println!("Source: {}\n", source);

        // Compile to FMPL IR
        let tokens = fmpl_core::lexer::Lexer::new(source).tokenize().unwrap();
        let ast = fmpl_core::parser::Parser::with_source(&tokens, source)
            .parse()
            .unwrap();
        let fmpl_code = fmpl_core::compiler::Compiler::new().compile(&ast).unwrap();

        // Benchmark FMPL VM
        let mut vm = fmpl_core::vm::Vm::new();

        // Warmup
        for _ in 0..100 {
            let _ = vm.run(&fmpl_code);
        }

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = vm.run(&fmpl_code);
        }
        let fmpl_time = start.elapsed();
        let fmpl_ns = fmpl_time.as_nanos() / iterations as u128;
        let fmpl_ops = (iterations as f64 / fmpl_time.as_secs_f64()) as u64;

        let fmpl_result = vm.run(&fmpl_code).unwrap();

        println!("FMPL VM:");
        println!("  Result: {:?}", fmpl_result);
        println!("  Time: {:.2}s", fmpl_time.as_secs_f64());
        println!("  {:.2} ns/op", fmpl_ns);
        println!("  {:.2} M ops/sec", fmpl_ops as f64 / 1_000_000.0);
        println!();
    }
}
