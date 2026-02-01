// Quick performance comparison between FMPL VM and execution_tape VM

use std::time::Instant;

fn main() {
    println!("=== FMPL VM vs execution_tape VM Performance Comparison ===\n");

    // Test 1: Simple arithmetic
    println!("Test 1: Simple arithmetic (7 + 9 * 5)");
    run_fmpl_test(
        "let a = 7; let b = 9; let c = 5; a + b * c",
        "fmpl_arithmetic",
    );
    run_exec_arithmetic();

    println!("\nTest 2: Conditional (if x > 5 then x * 2 else x where x = 10)");
    run_fmpl_test("let x = 10; if x > 5 then x * 2 else x", "fmpl_conditional");
    run_exec_conditional();

    println!("\nTest 3: Function call (lambda composition)");
    run_fmpl_test(
        "let add = lambda(x) lambda(y) x + y; let inc = add(1); inc(42)",
        "fmpl_function_call",
    );
    run_exec_function_call();
}

fn run_fmpl_test(source: &str, name: &str) {
    use fmpl_core::{compiler::Compiler, vm::Vm};

    // Compile
    let tokens = fmpl_core::lexer::Lexer::new(source).tokenize().unwrap();
    let ast = fmpl_core::parser::Parser::with_source(&tokens, source)
        .parse()
        .unwrap();
    let compiler = Compiler::new();
    let code = compiler.compile(&ast).unwrap();

    // Warmup
    let mut vm = Vm::new();
    for _ in 0..100 {
        vm.run(&code).unwrap();
    }

    // Benchmark
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        vm.run(&code).unwrap();
    }
    let elapsed = start.elapsed();

    let result = vm.run(&code).unwrap();
    let ns_per_op = elapsed.as_nanos() / iterations as u128;
    let ops_per_sec = (iterations as f64 / elapsed.as_secs_f64()) as u64;

    println!("  {}: {:?}", name, result);
    println!("    {} ops in {:?}", iterations, elapsed);
    println!("    {:.2} ns/op", ns_per_op);
    println!("    {:.2} M ops/sec", ops_per_sec as f64 / 1_000_000.0);
}

use execution_tape::asm::{Asm, FunctionSig, ProgramBuilder};
use execution_tape::host::{Host, HostError, SigHash, ValueRef};
use execution_tape::program::ValueType;
use execution_tape::trace::TraceMask;
use execution_tape::value::FuncId;
use execution_tape::vm::{Limits, Vm as ExecVm};

struct TestHost;
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

fn run_exec_arithmetic() {
    // r1=7, r2=9, r3=5, r4=r2*r3, r5=r1+r4
    let mut a = Asm::new();
    a.const_i64(1, 7);
    a.const_i64(2, 9);
    a.const_i64(3, 5);
    a.i64_mul(4, 2, 3);
    a.i64_add(5, 1, 4);
    a.ret(0, &[5]);

    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        a,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ValueType::I64],
            reg_count: 6,
        },
    )
    .unwrap();
    let program = pb.build_verified().unwrap();

    benchmark_exec(&program, FuncId(0), "exec_tape_arithmetic");
}

fn run_exec_conditional() {
    // if (10 > 5) { 10 * 2 } else { 10 }
    let mut a = Asm::new();
    a.const_i64(1, 10);
    a.const_i64(2, 5);
    a.i64_gt(3, 1, 2);
    a.i64_mul(4, 1, 1);
    a.mov(5, 1);
    a.select(6, 3, 4, 5);
    a.ret(0, &[6]);

    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        a,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ValueType::I64],
            reg_count: 7,
        },
    )
    .unwrap();
    let program = pb.build_verified().unwrap();

    benchmark_exec(&program, FuncId(0), "exec_tape_conditional");
}

fn run_exec_function_call() {
    // func0(y): return y + 1
    // func1(): call func0(42)
    let mut func0 = Asm::new();
    func0.const_i64(1, 1);
    func0.i64_add(2, 1, 1);
    func0.ret(0, &[2]);

    let mut func1 = Asm::new();
    func1.const_i64(1, 42);
    func1.call(0, FuncId(0), 0, &[1], &[2]);
    func1.ret(0, &[2]);

    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        func0,
        FunctionSig {
            arg_types: vec![ValueType::I64],
            ret_types: vec![ValueType::I64],
            reg_count: 3,
        },
    )
    .unwrap();

    pb.push_function_checked(
        func1,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![ValueType::I64],
            reg_count: 3,
        },
    )
    .unwrap();

    let program = pb.build_verified().unwrap();

    benchmark_exec(&program, FuncId(1), "exec_tape_function_call");
}

fn benchmark_exec(
    program: &execution_tape::verifier::VerifiedProgram,
    func_id: FuncId,
    name: &str,
) {
    let mut vm = ExecVm::new(TestHost, Limits::default());

    // Warmup
    for _ in 0..100 {
        vm.run(program, func_id, &[], TraceMask::NONE, None)
            .unwrap();
    }

    // Benchmark
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        vm.run(program, func_id, &[], TraceMask::NONE, None)
            .unwrap();
    }
    let elapsed = start.elapsed();

    let result = vm
        .run(program, func_id, &[], TraceMask::NONE, None)
        .unwrap();
    let ns_per_op = elapsed.as_nanos() / iterations as u128;
    let ops_per_sec = (iterations as f64 / elapsed.as_secs_f64()) as u64;

    println!("  {}: {:?}", name, result[0]);
    println!("    {} ops in {:?}", iterations, elapsed);
    println!("    {:.2} ns/op", ns_per_op);
    println!("    {:.2} M ops/sec", ops_per_sec as f64 / 1_000_000.0);
}
