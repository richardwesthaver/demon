//! does nothing currently
#![feature(custom_test_frameworks)]
#![test_runner(criterion::runner)]
use criterion::*;
use criterion_macro::criterion;

fn fibonacci(n: u64) -> u64 {
  let mut a = 0;
  let mut b = 1;

  match n {
    0 => b,
    _ => {
      for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
      }
      b
    }
  }
}

fn test_env() -> Criterion {
  Criterion::default().sample_size(200)
}

#[criterion(test_env())]
fn fib_benchmark(c: &mut Criterion) {
  c.bench_function("calc_fibonacci_10", |b| b.iter(|| fibonacci(black_box(10))));
}

#[criterion(test_env())]
fn bench_throughput(c: &mut Criterion) {
  let mut group = c.benchmark_group("Throughput");
  for size in [1024, 2048, 4096].iter() {
    let input = vec![1u64, *size];
    group.throughput(Throughput::Elements(*size as u64));
    group.bench_with_input(BenchmarkId::new("sum", *size), &input, |b, i| {
      b.iter(|| i.iter().sum::<u64>())
    });
    group.bench_with_input(BenchmarkId::new("fold", *size), &input, |b, i| {
      b.iter(|| i.iter().fold(0u64, |a, b| a + b))
    });
  }

  group.finish();
}

fn main() {
  test_env();
}
