// Benchmark: FMPL VM vs execution_tape VM (cross-compiled)
//
// This benchmark compares the performance of FMPL's Indexed RPN VM
// against execution_tape's register-based VM when running the same
// FMPL bytecode (cross-compiled to execution_tape format).

#[cfg(feature = "cross_compile")]
use std::time::Instant;

#[cfg(feature = "cross_compile")]
use execution_tape::host::{Host, HostError, SigHash, ValueRef};
#[cfg(feature = "cross_compile")]
use execution_tape::trace::TraceMask;
#[cfg(feature = "cross_compile)]
use execution_tape::value::FuncId;
#[cfg(feature = "cross_compile)]
use execution_tape::vm::{Limits, Vm as ExecVm};
#[cfg(feature = "cross_compile")]
use fmpl_core::cross_compile::cross_compile;

struct TestHost;
#[cfg(feature = "cross_compile")]
impl Host for TestHost {
    fn call(
        &mut self,
        _symbol: &str,
        _sig_hash: SigHash,
        _args: &[ValueRef<'_>],
    ) -> Result<(Vec<execution_tape::value::Value>, u64), HostError> {
        Err(HostError::UnknownSymbol)
    }
}

#[cfg(feature = "cross_compile")]
fn benchmark_fmpl(code: &fmpl_core::CompiledCode, name: &str, iterations: usize) {
    use fmpl_core::Vm;

    let mut vm = Vm::new();

    // Warmup
    for _ in 0..100 {
        let _ = vm.run(code);
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = vm.run(code);
    }
    let elapsed = start.elapsed();

    let ns_per_op = elapsed.as_nanos() / iterations as u128;
    let ops_per_sec = (iterations as f64 / elapsed.as_secs_f64()) as u64;

    println!("  FMPL VM ({}):", name);
    println!("    {} ops in {:?}", iterations, elapsed);
    println!("    {:.2} ns/op", ns_per_op);
    println!("    {:.2} M ops/sec", ops_per_sec as f64 / 1_000_000.0);
}

#[cfg(feature = "cross_compile")]
fn benchmark_execution_tape(
    program: &execution_tape::verifier::VerifiedProgram,
    func_id: FuncId,
    name: &str,
    iterations: usize,
) {
    let mut vm = ExecVm::new(TestHost, Limits::default());

    // Warmup
    for _ in 0..100 {
        let _ = vm.run(program, func_id, &[], TraceMask::NONE, None);
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = vm.run(program, func_id, &[], TraceMask::NONE, None);
    }
    let elapsed = start.elapsed();

    let ns_per_op = elapsed.as_nanos() / iterations as u128;
    let ops_per_sec = (iterations as f64 / elapsed.as_secs_f64()) as u64;

    println!("  execution_tape VM ({}):", name);
    println!("    {} ops in {:?}", iterations, elapsed);
    println!("    {:.2} ns/op", ns_per_op);
    println!("    {:.2} M ops/sec", ops_per_sec as f64 / 1_000_000.0);
}

#[cfg(feature = "cross_compile")]
fn run_benchmark(source: &str, name: &str, iterations: usize) {
    use fmpl_core::compiler::Compiler;
    use fmpl_core::lexer::Lexer;
    use fmpl_core::parser::Parser;

    println!("\n=== Benchmark: {} ===", name);
    println!("Source: {}", source);

    // Compile to FMPL bytecode
    let tokens = Lexer::new(source).tokenize().expect("lexer error");
    let ast = Parser::with_source(&tokens, source)
        .parse()
        .expect("parse error");
    let fmpl_code = Compiler::new().compile(&ast).expect("compile error");

    // Cross-compile to execution_tape
    let exec_program = cross_compile(&fmpl_code).expect("cross-compile error");

    // Benchmark FMPL VM
    benchmark_fmpl(&fmpl_code, name, iterations);

    // Benchmark execution_tape VM
    benchmark_execution_tape(&exec_program, FuncId(0), name, iterations);

    // Calculate speedup
    let mut vm_fmpl = fmpl_core::Vm::new();
    let mut vm_exec = ExecVm::new(TestHost, Limits::default());

    // Single run to verify correctness
    let fmpl_result = vm_fmpl.run(&fmpl_code).expect("FMPL VM error");
    let exec_result = vm_exec
        .run(&exec_program, FuncId(0), &[], TraceMask::NONE, None)
        .expect("execution_tape VM error");

    println!("  Results:");
    println!("    FMPL VM: {:?}", fmpl_result);
    println!("    execution_tape VM: {:?}", exec_result[0]);

    // Timing comparison
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = vm_fmpl.run(&fmpl_code);
    }
    let fmpl_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = vm_exec.run(&exec_program, FuncId(0), &[], TraceMask::NONE, None);
    }
    let exec_time = start.elapsed();

    let speedup = fmpl_time.as_secs_f64() / exec_time.as_secs_f64();
    println!("  Speedup: {:.2}x", speedup);
}

fn main() {
    #[cfg(feature = "cross_compile")]
    {
        println!("=== FMPL VM vs execution_tape VM (Cross-Compiled) ===");
        println!("Comparing FMPL bytecode execution on both VMs");

        let iterations = 10000;

        // Test 1: Simple arithmetic
        run_benchmark("7 + 9 * 5", "Arithmetic", iterations);

        // Test 2: Comparison
        run_benchmark("10 > 5", "Comparison", iterations);

        // Test 3: Complex expression
        run_benchmark("(1 + 2) * (3 + 4)", "Complex", iterations);

        println!("\n=== Summary ===");
        println!("Cross-compilation enables direct comparison of VM performance");
        println!("on identical FMPL bytecode programs.");
    }

    #[cfg(not(feature = "cross_compile"))]
    {
        println!("Benchmark requires 'cross_compile' feature.");
        println!("Run with: cargo bench --features cross_compile");
    }
}
