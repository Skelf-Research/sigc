//! Reproducibility: identical inputs must yield byte-identical outputs.
//!
//! This backs the paper's reproducibility claim — and guards against the
//! kind of hidden nondeterminism (e.g. an unseeded RNG in the synthetic data
//! path) that silently breaks "identical inputs = identical outputs".

use polars::prelude::*;
use sig_cache::Cache;
use sig_compiler::Compiler;
use sig_runtime::{DataLoader, Runtime};

const STRATEGY: &str = "data:
  prices: load parquet from \"prices.parquet\"

signal momentum:
  r   = ret(prices, periods=20)
  vol = rolling_std(r, window=60)
  emit zscore(r / vol)
";

/// blake3 digest of a returns series, treated as little-endian f64 bytes.
fn digest(returns: &[f64]) -> String {
    let mut bytes = Vec::with_capacity(returns.len() * 8);
    for v in returns {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    Cache::hash(&bytes)
}

#[test]
fn compilation_is_deterministic() {
    let a = Compiler::new().compile(STRATEGY).expect("compile a");
    let b = Compiler::new().compile(STRATEGY).expect("compile b");
    // Content-addressed: same source -> same hash, same graph.
    assert_eq!(a.metadata.source_hash, b.metadata.source_hash);
    assert_eq!(a.nodes.len(), b.nodes.len());
    assert_eq!(a.outputs, b.outputs);
}

#[test]
fn backtest_is_bit_exact_across_runs() {
    let ir = Compiler::new().compile(STRATEGY).expect("compile");

    let mut rt1 = Runtime::new();
    let mut rt2 = Runtime::new();
    let r1 = rt1.run_ir(&ir).expect("run 1");
    let r2 = rt2.run_ir(&ir).expect("run 2");

    assert_eq!(r1.returns_series, r2.returns_series, "returns differ across runs");
    assert_eq!(digest(&r1.returns_series), digest(&r2.returns_series), "digests differ");

    // Metrics must match to the bit, not merely approximately.
    assert_eq!(r1.metrics.sharpe_ratio.to_bits(), r2.metrics.sharpe_ratio.to_bits());
    assert_eq!(r1.metrics.total_return.to_bits(), r2.metrics.total_return.to_bits());
    assert_eq!(r1.metrics.max_drawdown.to_bits(), r2.metrics.max_drawdown.to_bits());
}

#[test]
fn seeded_sample_data_is_reproducible() {
    let a = DataLoader::sample_prices_seeded(64, 5, 123).expect("sample a");
    let b = DataLoader::sample_prices_seeded(64, 5, 123).expect("sample b");

    let col = |df: &DataFrame| -> Vec<f64> {
        df.column("asset_0")
            .unwrap()
            .f64()
            .unwrap()
            .into_iter()
            .map(|v| v.unwrap())
            .collect()
    };
    assert_eq!(col(&a), col(&b), "same seed must produce identical prices");
}

#[test]
fn different_seeds_differ() {
    let a = DataLoader::sample_prices_seeded(64, 5, 1).expect("sample a");
    let b = DataLoader::sample_prices_seeded(64, 5, 2).expect("sample b");
    let last = |df: &DataFrame| -> f64 {
        let s = df.column("asset_0").unwrap().f64().unwrap();
        s.get(s.len() - 1).unwrap()
    };
    assert_ne!(last(&a), last(&b), "different seeds should diverge");
}
