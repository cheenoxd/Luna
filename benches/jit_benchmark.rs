use criterion::{black_box, criterion_group, criterion_main, Criterion};
use luna::runtime::LuaJitRuntime;

fn fibonacci_benchmark(c: &mut Criterion) {
    c.bench_function("fibonacci_interpreted", |b| {
        b.iter(|| {
            let mut runtime = LuaJitRuntime::new();
            let source = r#"
                function fib(n)
                    if n <= 1 then
                        return n
                    else
                        return fib(n - 1) + fib(n - 2)
                    end
                end
                return fib(10)
            "#;
            runtime.execute(black_box(source)).unwrap()
        })
    });
}

fn arithmetic_benchmark(c: &mut Criterion) {
    c.bench_function("arithmetic_hot_loop", |b| {
        b.iter(|| {
            let mut runtime = LuaJitRuntime::new();
            let source = r#"
                local sum = 0
                for i = 1, 1000 do
                    sum = sum + i * 2
                end
                return sum
            "#;
            runtime.execute(black_box(source)).unwrap()
        })
    });
}

fn variable_assignment_benchmark(c: &mut Criterion) {
    c.bench_function("variable_assignment", |b| {
        b.iter(|| {
            let mut runtime = LuaJitRuntime::new();
            let source = r#"
                local x = 10
                local y = 20
                local z = x + y * 3
                return z
            "#;
            runtime.execute(black_box(source)).unwrap()
        })
    });
}

criterion_group!(benches, fibonacci_benchmark, arithmetic_benchmark, variable_assignment_benchmark);
criterion_main!(benches);
