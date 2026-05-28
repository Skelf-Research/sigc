//! Cache speedup benchmarks for the paper's reproducibility experiment.
//!
//! Quantifies the content-addressed-cache benefit: a compile that hits the
//! cache should be orders of magnitude cheaper than the cold compile, because
//! all that runs is a blake3 source hash + a sled point-lookup + an rkyv
//! zero-copy deserialize.
//!
//! Run with: cargo bench --package sig_runtime --bench cache_repro

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sig_cache::Cache;
use sig_compiler::Compiler;

// A realistic, fully-typed strategy from the bundled corpus.
const SRC: &str = include_str!("../../../examples/leaky/safe_momentum.sig");

fn bench_compile_cache(c: &mut Criterion) {
    let mut g = c.benchmark_group("compile_cache");

    // Baseline: no cache at all. Isolates the cost of the cache machinery
    // when cold (should be ~indistinguishable from `cold`).
    g.bench_function("nocache", |b| {
        let compiler = Compiler::new();
        b.iter(|| {
            black_box(compiler.compile(black_box(SRC)).unwrap());
        });
    });

    // Cold: fresh in-memory cache per iteration; every compile misses.
    g.bench_function("cold", |b| {
        b.iter(|| {
            let cache = Cache::in_memory().unwrap();
            let compiler = Compiler::with_cache(cache);
            black_box(compiler.compile(black_box(SRC)).unwrap());
        });
    });

    // Warm: cache is primed once outside the loop; every iter is a hit.
    g.bench_function("warm", |b| {
        let cache = Cache::in_memory().unwrap();
        let compiler = Compiler::with_cache(cache);
        compiler.compile(SRC).expect("prime");
        b.iter(|| {
            black_box(compiler.compile(black_box(SRC)).unwrap());
        });
    });

    g.finish();
}

criterion_group!(benches, bench_compile_cache);
criterion_main!(benches);
