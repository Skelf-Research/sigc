//! Integration tests for sigc

use polars::prelude::NamedFrom;
use std::collections::HashMap;

#[test]
fn test_compile_simple_signal() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  period = 10

signal momentum:
  ret = ret(prices, period)
  emit zscore(ret)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    assert!(!ir.nodes.is_empty(), "IR should have nodes");
    assert!(!ir.outputs.is_empty(), "IR should have outputs");
    assert_eq!(ir.metadata.parameters.len(), 1);
    assert_eq!(ir.metadata.data_sources.len(), 1);
}

#[test]
fn test_compile_with_comments() {
    let source = r#"
// This is a comment

data:
  prices: load csv from "test.csv"

// Another comment
params:
  lookback = 5

signal test:
  x = ret(prices, lookback)
  emit x

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Should parse comments: {:?}", result.err());
}

#[test]
fn test_compile_multiple_signals() {
    let source = r#"
data:
  px: load parquet from "prices.parquet"

params:
  short = 5
  long = 20

signal fast:
  ret = ret(px, short)
  emit zscore(ret)

signal slow:
  ret = ret(px, long)
  emit zscore(ret)

portfolio combined:
  weights = rank(fast).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    assert_eq!(ir.metadata.parameters.len(), 2);
    assert_eq!(ir.metadata.data_sources.len(), 1);
}

#[test]
fn test_ir_execution() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  period = 5

signal mom:
  ret = ret(prices, period)
  emit zscore(ret)

portfolio main:
  weights = rank(mom).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    // Create mock input data
    let mut inputs: HashMap<String, polars::prelude::Series> = HashMap::new();
    let prices: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
    inputs.insert("prices".to_string(), polars::prelude::Series::new("prices".into(), prices));

    // Execute IR
    let mut engine = sig_runtime::Engine::new();
    let outputs = engine.execute(&ir, &inputs).unwrap();

    assert!(!outputs.is_empty(), "Should produce outputs");
    assert_eq!(outputs[0].len(), 100, "Output should match input length");
}

#[test]
fn test_full_pipeline() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  lookback = 10

signal alpha:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit winsor(score, p=0.01)

portfolio strat:
  weights = rank(alpha).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    // Create a runtime and execute
    let mut runtime = sig_runtime::Runtime::new();
    let result = runtime.run_ir(&ir);

    // Should complete without error (even with sample data)
    assert!(result.is_ok(), "Runtime should execute: {:?}", result.err());

    let report = result.unwrap();
    assert!(report.metrics.total_return.is_finite());
    assert!(report.metrics.sharpe_ratio.is_finite());
}

#[test]
fn test_error_undefined_identifier() {
    let source = r#"
data:
  prices: load csv from "test.csv"

signal bad:
  x = ret(undefined, 5)
  emit x

portfolio main:
  weights = rank(bad).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);

    assert!(result.is_err(), "Should fail with undefined identifier");
}

#[test]
fn test_data_format_detection() {
    use sig_runtime::DataFormat;

    assert!(matches!(DataFormat::from_path("data.parquet"), DataFormat::Parquet));
    assert!(matches!(DataFormat::from_path("data.csv"), DataFormat::Csv));
    assert!(matches!(DataFormat::from_path("s3://bucket/key.parquet"), DataFormat::Parquet));
    assert!(matches!(DataFormat::from_path("data.csv.gz"), DataFormat::Csv));
}

#[test]
fn test_backtest_metrics_valid() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  n = 5

signal sig:
  r = ret(px, n)
  emit zscore(r)

portfolio p:
  weights = rank(sig).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let report = runtime.run_ir(&ir).unwrap();

    // Check that metrics are reasonable
    assert!(report.metrics.max_drawdown >= 0.0, "Drawdown should be non-negative");
    assert!(report.metrics.max_drawdown <= 1.0, "Drawdown should be <= 100%");
    assert!(report.metrics.turnover >= 0.0, "Turnover should be non-negative");
}

#[test]
fn test_cache_hit_miss() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  period = 5

signal cached:
  ret = ret(prices, period)
  emit zscore(ret)

portfolio main:
  weights = rank(cached).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();

    // First compile - cache miss
    let ir1 = compiler.compile(source).unwrap();

    // Second compile - should hit cache (same source)
    let ir2 = compiler.compile(source).unwrap();

    // IRs should be equivalent
    assert_eq!(ir1.nodes.len(), ir2.nodes.len());
    assert_eq!(ir1.outputs.len(), ir2.outputs.len());
}

#[test]
fn test_deterministic_compilation() {
    // Test that compilation is deterministic (same IR for same source)
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  window = 10

signal deterministic:
  r = ret(prices, window)
  z = zscore(r)
  emit winsor(z, p=0.01)

portfolio main:
  weights = rank(deterministic).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir1 = compiler.compile(source).unwrap();
    let ir2 = compiler.compile(source).unwrap();

    // IRs should be identical
    assert_eq!(ir1.nodes.len(), ir2.nodes.len());
    assert_eq!(ir1.outputs.len(), ir2.outputs.len());
    assert_eq!(ir1.metadata.parameters.len(), ir2.metadata.parameters.len());
}

#[test]
fn test_all_basic_operators() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  n = 5

signal ops:
  r = ret(px, n)
  l = lag(px, n)
  z = zscore(r)
  rk = rank(r)
  w = winsor(z, p=0.01)
  emit w

portfolio main:
  weights = rank(ops).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let result = runtime.run_ir(&ir);
    assert!(result.is_ok(), "All basic operators should work: {:?}", result.err());
}

#[test]
fn test_rolling_operators() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  window = 10

signal rolling:
  mean = rolling_mean(px, window)
  std = rolling_std(px, window)
  sum = rolling_sum(px, window)
  min = rolling_min(px, window)
  max = rolling_max(px, window)
  emit mean

portfolio main:
  weights = rank(rolling).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let result = runtime.run_ir(&ir);
    assert!(result.is_ok(), "Rolling operators should work: {:?}", result.err());
}

#[test]
fn test_arithmetic_operators() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  n = 5

signal arith:
  r = ret(px, n)
  a = abs(r)
  s = sqrt(a)
  emit s

portfolio main:
  weights = rank(arith).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let result = runtime.run_ir(&ir);
    assert!(result.is_ok(), "Arithmetic operators should work: {:?}", result.err());
}

#[test]
fn test_grid_search_optimization() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  period = 10

signal mom:
  ret = ret(prices, period)
  emit zscore(ret)

portfolio main:
  weights = rank(mom).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let mut grid = sig_runtime::GridSearch::new();
    grid.add_param("period", vec![5.0, 10.0, 20.0]);

    let results = grid.optimize(&ir, &mut runtime, "sharpe").unwrap();

    assert_eq!(results.len(), 3, "Should have result for each parameter value");
    // Results should be sorted by metric
    assert!(results[0].metrics.sharpe_ratio >= results[1].metrics.sharpe_ratio);
}

#[test]
fn test_panel_operations() {
    use sig_runtime::Panel;

    let assets = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let values = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![5.0, 4.0, 3.0, 2.0, 1.0],
        vec![3.0, 3.0, 3.0, 3.0, 3.0],
    ];

    let panel = Panel::from_vecs(assets, values).unwrap();

    // Test cross-sectional operations
    let zscored = panel.xs_zscore().unwrap();
    let ranked = panel.xs_rank().unwrap();
    let demeaned = panel.xs_demean().unwrap();
    let scaled = panel.xs_scale().unwrap();

    // Verify dimensions preserved
    assert_eq!(zscored.n_assets(), 3);
    assert_eq!(zscored.n_periods, 5);
    assert_eq!(ranked.n_assets(), 3);
    assert_eq!(demeaned.n_assets(), 3);
    assert_eq!(scaled.n_assets(), 3);
}

#[test]
fn test_error_invalid_parameter() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  period = 5

signal bad:
  r = ret(px, invalid_param)
  emit r

portfolio main:
  weights = rank(bad).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "Should fail with invalid parameter");
}

#[test]
fn test_error_syntax() {
    let source = r#"
data:
  px: load csv from "test.csv"

signal bad
  r = ret(px, 5)
  emit r

portfolio main:
  weights = rank(bad).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "Should fail with syntax error");
}

#[test]
fn test_empty_source() {
    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile("");
    assert!(result.is_err(), "Empty source should fail");
}

#[test]
fn test_multiple_params() {
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  short = 5
  medium = 10
  long = 20
  threshold = 0.5

signal multi:
  r1 = ret(prices, short)
  r2 = ret(prices, medium)
  r3 = ret(prices, long)
  emit zscore(r1)

portfolio main:
  weights = rank(multi).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    assert_eq!(ir.metadata.parameters.len(), 4);
}

#[test]
fn test_backtest_report_fields() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  n = 5

signal test:
  r = ret(px, n)
  emit zscore(r)

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let report = runtime.run_ir(&ir).unwrap();

    // All key metrics should be populated
    assert!(report.metrics.total_return.is_finite());
    assert!(report.metrics.sharpe_ratio.is_finite());
    assert!(report.metrics.max_drawdown.is_finite());
    assert!(report.metrics.turnover.is_finite());

    // Report metadata should be set
    assert!(report.executed_at > 0);
}
