// Criterion benchmark comparing FMPL VM vs execution_tape VM
// on equivalent operations

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use execution_tape::asm::{Asm, FunctionSig, ProgramBuilder};
use execution_tape::host::{Host, HostError, SigHash, ValueRef};
use execution_tape::trace::TraceMask;
use execution_tape::value::FuncId;
use execution_tape::verifier::verify_program_owned;
use execution_tape::vm::{Limits, Vm as ExecVm};
use fmpl_core::{compiler::Compiler, vm::Vm};

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

fn compile_fmpl(source: &str) -> fmpl_core::compiler::CompiledCode {
    let tokens = fmpl_core::lexer::Lexer::new(source).tokenize().unwrap();
    let ast = fmpl_core::parser::Parser::with_source(&tokens, source)
        .parse()
        .unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

fn run_fmpl(code: &fmpl_core::compiler::CompiledCode) -> fmpl_core::value::Value {
    let mut vm = Vm::new();
    vm.run(code).unwrap()
}

fn build_exec_tape(
    asm: Asm,
    reg_count: u32,
    ret_count: usize,
) -> execution_tape::verifier::VerifiedProgram {
    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        asm,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![execution_tape::program::ValueType::I64; ret_count],
            reg_count,
        },
    )
    .unwrap();
    let p = pb.build_verified().unwrap();
    verify_program_owned(p.program(), p.type_table())
}

fn run_exec_tape(
    program: &execution_tape::verifier::VerifiedProgram,
    func_id: FuncId,
) -> execution_tape::value::Value {
    let mut vm = ExecVm::new(TestHost, Limits::default());
    let out = vm
        .run(program, func_id, &[], TraceMask::NONE, None)
        .unwrap();
    out[0].clone()
}

// Benchmark: Simple arithmetic (a + b * c)
fn bench_arithmetic_fmpl(c: &mut Criterion) {
    let code = compile_fmpl("let a = 7; let b = 9; let c = 5; a + b * c");

    c.bench_function("fmpl_arithmetic", |b| {
        b.iter(|| run_fmpl(black_box(&code)));
    });
}

fn bench_arithmetic_exec(c: &mut Criterion) {
    // r1=7, r2=9, r3=5, r4=r2*r3, r5=r1+r4
    let mut a = Asm::new();
    a.const_i64(1, 7);
    a.const_i64(2, 9);
    a.const_i64(3, 5);
    a.i64_mul(4, 2, 3);
    a.i64_add(5, 1, 4);
    a.ret(0, &[5]);

    let program = build_exec_tape(a, 6, 1);

    c.bench_function("exec_tape_arithmetic", |b| {
        b.iter(|| run_exec_tape(black_box(&program), FuncId(0)));
    });
}

// Benchmark: Conditional logic
fn bench_conditional_fmpl(c: &mut Criterion) {
    let code = compile_fmpl("let x = 10; if x > 5 then x * 2 else x");

    c.bench_function("fmpl_conditional", |b| {
        b.iter(|| run_fmpl(black_box(&code)));
    });
}

fn bench_conditional_exec(c: &mut Criterion) {
    // if (10 > 5) { 10 * 2 } else { 10 }
    let mut a = Asm::new();
    a.const_i64(1, 10);
    a.const_i64(2, 5);
    a.i64_gt(3, 1, 2); // true
    a.i64_mul(4, 1, 1); // 10 * 2 = 20 (true branch)
    a.mov(5, 1); // 10 (false branch)
    a.select(6, 3, 4, 5);
    a.ret(0, &[6]);

    let program = build_exec_tape(a, 7, 1);

    c.bench_function("exec_tape_conditional", |b| {
        b.iter(|| run_exec_tape(black_box(&program), FuncId(0)));
    });
}

// Benchmark: Function call
fn bench_function_call_fmpl(c: &mut Criterion) {
    let code = compile_fmpl("let add = lambda(x) lambda(y) x + y; let inc = add(1); inc(42)");

    c.bench_function("fmpl_function_call", |b| {
        b.iter(|| run_fmpl(black_box(&code)));
    });
}

fn bench_function_call_exec(c: &mut Criterion) {
    // func0(y): return y + 1
    // func1(): call func0(42)
    let mut func0 = Asm::new();
    func0.const_i64(1, 1);
    func0.i64_add(2, 1, 1); // r1 + 1 (r1 is arg)
    func0.ret(0, &[2]);

    let mut func1 = Asm::new();
    func1.const_i64(1, 42);
    func1.call(0, FuncId(0), 0, &[1], 1, &[2]); // call func0(r1) -> r2
    func1.ret(0, &[2]);

    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        func0,
        FunctionSig {
            arg_types: vec![execution_tape::program::ValueType::I64],
            ret_types: vec![execution_tape::program::ValueType::I64],
            reg_count: 3,
        },
    )
    .unwrap();

    pb.push_function_checked(
        func1,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![execution_tape::program::ValueType::I64],
            reg_count: 3,
        },
    )
    .unwrap();

    let p = pb.build_verified().unwrap();
    let program = verify_program_owned(p.program(), p.type_table());

    c.bench_function("exec_tape_function_call", |b| {
        b.iter(|| run_exec_tape(black_box(&program), FuncId(1)));
    });
}

// Benchmark: Complex computation (fibonacci)
fn bench_fibonacci_fmpl(c: &mut Criterion) {
    let code =
        compile_fmpl("let fib = lambda(n) if n < 2 then n else fib(n-1) + fib(n-2); fib(10)");

    c.bench_function("fmpl_fibonacci", |b| {
        b.iter(|| run_fmpl(black_box(&code)));
    });
}

fn bench_fibonacci_exec(c: &mut Criterion) {
    // Iterative fibonacci to avoid deep recursion
    let mut a = Asm::new();
    // n = 10
    a.const_i64(1, 10);
    // if n < 2, return n
    a.const_i64(2, 2);
    a.i64_lt(3, 1, 2);
    a.br(3, 8, 4); // if true -> ret, else -> continue

    // Iterative: a=0, b=1, loop n times: (a, b) = (b, a+b)
    a.label(4);
    a.const_i64(5, 0); // a
    a.const_i64(6, 1); // b
    a.const_i64(7, 1); // i

    a.label(5); // loop start
    a.i64_add(8, 5, 6); // a + b
    a.mov(5, 6); // a = b
    a.mov(6, 8); // b = a+b
    a.const_i64(9, 1);
    a.i64_add(10, 7, 9); // i++
    a.mov(7, 10);
    a.i64_lt(11, 7, 1); // i < n?
    a.br(11, 5, 6); // if true -> loop, else -> exit

    a.label(6);
    a.ret(0, &[5]);

    a.label(8);
    a.ret(0, &[1]);

    let mut pb = ProgramBuilder::new();
    pb.push_function_checked(
        a,
        FunctionSig {
            arg_types: vec![],
            ret_types: vec![execution_tape::program::ValueType::I64],
            reg_count: 12,
        },
    )
    .unwrap();

    let p = pb.build_verified().unwrap();
    let program = verify_program_owned(p.program(), p.type_table());

    c.bench_function("exec_tape_fibonacci", |b| {
        b.iter(|| run_exec_tape(black_box(&program), FuncId(0)));
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets =
        bench_arithmetic_fmpl,
        bench_arithmetic_exec,
        bench_conditional_fmpl,
        bench_conditional_exec,
        bench_function_call_fmpl,
        bench_function_call_exec,
        bench_fibonacci_fmpl,
        bench_fibonacci_exec,
}

criterion_main!(benches);
