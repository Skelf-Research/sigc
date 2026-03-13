//! Benchmarks for kernel performance
//!
//! Run with: cargo bench --package sig_runtime

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use polars::prelude::*;
use sig_runtime::kernels;

fn generate_series(n: usize) -> Series {
    let values: Vec<f64> = (0..n).map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0).collect();
    Series::new("data".into(), values)
}

fn bench_ret(c: &mut Criterion) {
    let mut group = c.benchmark_group("ret");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::ret(black_box(&series), black_box(10)))
        });
    }
    group.finish();
}

fn bench_lag(c: &mut Criterion) {
    let mut group = c.benchmark_group("lag");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::lag(black_box(&series), black_box(5)))
        });
    }
    group.finish();
}

fn bench_zscore(c: &mut Criterion) {
    let mut group = c.benchmark_group("zscore");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::zscore(black_box(&series)))
        });
    }
    group.finish();
}

fn bench_rank(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::rank(black_box(&series)))
        });
    }
    group.finish();
}

fn bench_rolling_mean(c: &mut Criterion) {
    let mut group = c.benchmark_group("rolling_mean");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::rolling_mean(black_box(&series), black_box(20)))
        });
    }
    group.finish();
}

fn bench_rolling_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("rolling_std");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::rolling_std(black_box(&series), black_box(20)))
        });
    }
    group.finish();
}

fn bench_winsor(c: &mut Criterion) {
    let mut group = c.benchmark_group("winsor");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::winsor(black_box(&series), black_box(0.01), black_box(0.99)))
        });
    }
    group.finish();
}

fn bench_cumsum(c: &mut Criterion) {
    let mut group = c.benchmark_group("cumsum");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::cumsum(black_box(&series)))
        });
    }
    group.finish();
}

fn bench_rsi(c: &mut Criterion) {
    let mut group = c.benchmark_group("rsi");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::rsi(black_box(&series), black_box(14)))
        });
    }
    group.finish();
}

fn bench_long_short(c: &mut Criterion) {
    let mut group = c.benchmark_group("long_short");

    for size in [1000, 10000, 100000].iter() {
        let series = generate_series(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| kernels::long_short(black_box(&series), black_box(0.2), black_box(0.2), black_box(1.0)))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_ret,
    bench_lag,
    bench_zscore,
    bench_rank,
    bench_rolling_mean,
    bench_rolling_std,
    bench_winsor,
    bench_cumsum,
    bench_rsi,
    bench_long_short,
);

criterion_main!(benches);
