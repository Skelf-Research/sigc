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
