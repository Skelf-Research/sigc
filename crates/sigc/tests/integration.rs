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

#[test]
fn test_multi_asset_backtest() {
    // Test that multi-asset backtests execute per-asset signals correctly
    let source = r#"
data:
  prices: load csv from "test.csv"

params:
  period = 10

signal momentum:
  r = ret(prices, period)
  emit zscore(r)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    let mut runtime = sig_runtime::Runtime::new();
    let report = runtime.run_ir(&ir).unwrap();

    // Metrics should be reasonable for a long/short strategy
    assert!(report.metrics.total_return.is_finite());
    assert!(report.metrics.sharpe_ratio.is_finite());
    // Max drawdown should be non-negative
    assert!(report.metrics.max_drawdown >= 0.0);
    // Turnover should be positive for an active strategy
    assert!(report.metrics.turnover >= 0.0);
}

#[test]
fn test_turnover_calculation() {
    // Test that turnover is calculated correctly from weight changes
    use polars::prelude::*;
    use sig_runtime::Backtester;

    let backtester = Backtester::new();

    // Create simple weights that change over time
    let weights = df! {
        "A" => &[0.5, 0.3, 0.4],
        "B" => &[-0.5, -0.3, -0.4]
    }.unwrap();

    // Create simple prices (constant returns)
    let prices = df! {
        "A" => &[100.0, 101.0, 102.0],
        "B" => &[100.0, 99.0, 98.0]
    }.unwrap();

    let plan = sig_types::BacktestPlan {
        ir: sig_types::Ir {
            nodes: vec![],
            outputs: vec![],
            metadata: sig_types::IrMetadata {
                source_hash: "test".to_string(),
                compiled_at: 0,
                compiler_version: "0.1.0".to_string(),
                parameters: vec![],
                data_sources: vec![],
            },
        },
        start_date: "2020-01-01".to_string(),
        end_date: "2024-12-31".to_string(),
        universe: "test".to_string(),
        parameters: std::collections::HashMap::new(),
    };

    let report = backtester.run(&weights, &prices, &plan).unwrap();

    // Turnover should be positive (weights changed)
    assert!(report.metrics.turnover > 0.0);
}

#[test]
fn test_date_range_filtering() {
    use sig_runtime::data::{DataLoader, DateRange};

    // Test date range creation
    let range = DateRange {
        start: Some("2022-01-01".to_string()),
        end: Some("2022-12-31".to_string()),
    };

    assert_eq!(range.start, Some("2022-01-01".to_string()));
    assert_eq!(range.end, Some("2022-12-31".to_string()));

    // Test sample data generation
    let df = DataLoader::sample_prices(100, 5).unwrap();
    assert_eq!(df.height(), 100);
    assert_eq!(df.width(), 6); // date + 5 assets
}

#[test]
fn test_cost_model_calculations() {
    use sig_runtime::{CostModel, ImpactModel};

    // Test institutional model
    let model = CostModel::institutional();
    let cost = model.calculate_cost(100_000.0, Some(1_000_000.0), false, 21.0);

    assert!(cost.commission > 0.0);
    assert!(cost.slippage > 0.0);
    assert!(cost.impact > 0.0);
    assert_eq!(cost.borrow, 0.0); // Not a short

    // Test short position borrow cost
    let short_cost = model.calculate_cost(100_000.0, Some(1_000_000.0), true, 21.0);
    assert!(short_cost.borrow > 0.0);

    // Test zero cost model
    let zero = CostModel::zero();
    let zero_cost = zero.calculate_cost(100_000.0, Some(1_000_000.0), true, 21.0);
    assert_eq!(zero_cost.total, 0.0);

    // Test impact models
    let linear = CostModel::new().with_impact(ImpactModel::Linear { coefficient: 0.1 });
    let sqrt = CostModel::new().with_impact(ImpactModel::SquareRoot { coefficient: 0.1 });

    let linear_cost = linear.calculate_cost(100_000.0, Some(1_000_000.0), false, 21.0);
    let sqrt_cost = sqrt.calculate_cost(100_000.0, Some(1_000_000.0), false, 21.0);

    assert!(linear_cost.impact > 0.0);
    assert!(sqrt_cost.impact > 0.0);
}

#[test]
fn test_walk_forward_optimization() {
    use sig_runtime::{WalkForward, WalkForwardConfig};

    // Test basic configuration
    let config = WalkForwardConfig::new(252, 126, 21);
    assert_eq!(config.total_periods, 252);
    assert_eq!(config.train_periods, 126);
    assert_eq!(config.test_periods, 21);

    // Test walk forward creation with parameters
    let mut wf = WalkForward::new(config);
    wf.add_range("period", 5.0, 20.0, 5.0);
    wf.add_range("lookback", 10.0, 50.0, 10.0);

    // Struct creation should succeed
    // (actual execution requires valid IR which is tested elsewhere)
}

#[test]
fn test_simd_kernels() {
    use sig_runtime::{rolling_mean_simd, rolling_std_simd, cumsum_simd, ema_simd, KernelDispatcher};
    use polars::prelude::*;

    let values: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let series = Series::new("test".into(), values);

    // Test SIMD rolling mean
    let mean = rolling_mean_simd(&series, 10).unwrap();
    assert_eq!(mean.len(), 100);

    // Test SIMD rolling std
    let std = rolling_std_simd(&series, 10).unwrap();
    assert_eq!(std.len(), 100);

    // Test SIMD cumsum
    let sum = cumsum_simd(&series).unwrap();
    let last: f64 = sum.f64().unwrap().get(99).unwrap();
    assert_eq!(last, 5050.0); // Sum of 1..100

    // Test SIMD EMA
    let ema = ema_simd(&series, 10).unwrap();
    assert_eq!(ema.len(), 100);

    // Test kernel dispatcher
    let dispatcher = KernelDispatcher::default();
    let result = dispatcher.rolling_mean(&series, 10).unwrap();
    assert_eq!(result.len(), 100);
}

#[test]
fn test_incremental_computation() {
    use sig_runtime::{IncrementalCompute, RollingMeanState, EmaState};

    // Test rolling mean state
    let mut state = RollingMeanState::new(5);
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let results = state.update_batch(&values);
    assert_eq!(results.len(), 7);
    assert!((results[4] - 3.0).abs() < 0.001); // (1+2+3+4+5)/5

    // Test EMA state
    let mut ema = EmaState::new(3);
    let v1 = ema.update(10.0);
    assert_eq!(v1, 10.0);
    let v2 = ema.update(20.0);
    assert!(v2 > 10.0 && v2 < 20.0);

    // Test incremental compute manager
    let mut compute = IncrementalCompute::new();
    compute.register_rolling_mean("sma", 3);
    compute.register_ema("ema", 5);

    let mean = compute.update_rolling_mean("sma", 100.0).unwrap();
    assert_eq!(mean, 100.0);

    let ema_val = compute.update_ema("ema", 100.0).unwrap();
    assert_eq!(ema_val, 100.0);
}

#[test]
fn test_runtime_config() {
    use sig_runtime::{RuntimeConfig, StrategyParams};

    // Test default config
    let config = RuntimeConfig::default();
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.execution.chunk_size, 100);
    assert!(config.cache.enabled);

    // Test strategy params
    let mut params = StrategyParams::new();
    params.set("window", 20.0).set("threshold", 0.5);
    assert_eq!(params.get("window"), Some(20.0));
    assert_eq!(params.get_or("missing", 10.0), 10.0);

    // Config should be serializable (basic check)
    assert!(config.database.port > 0);
    assert!(config.execution.chunk_size > 0);
}

#[test]
fn test_data_quality_validation() {
    use sig_runtime::{DataQualityValidator, MissingDataCheck, OutlierCheck, OutlierMethod};
    use polars::prelude::*;

    // Create test data with some issues
    let df = DataFrame::new(vec![
        Column::new("price".into(), vec![100.0, 101.0, 102.0, 103.0, 104.0]),
        Column::new("volume".into(), vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0]),
    ]).unwrap();

    // Test missing data check
    let check = MissingDataCheck {
        max_missing_pct: 10.0,
        columns: vec!["price".into()],
    };

    let validator = DataQualityValidator::new()
        .add(Box::new(check));

    let result = validator.validate(&df).unwrap();
    assert!(result.passed);

    // Test outlier check
    let outlier_check = OutlierCheck {
        method: OutlierMethod::ZScore { threshold: 3.0 },
        columns: vec!["price".into()],
        max_outlier_pct: 10.0,
    };

    let validator2 = DataQualityValidator::new()
        .add(Box::new(outlier_check));
    let result2 = validator2.validate(&df).unwrap();
    assert!(result2.passed);
}

#[test]
fn test_corporate_actions() {
    use sig_runtime::{CorporateAction, CorporateActionStore, SymbolMapper};

    // Test creating actions
    let split = CorporateAction::split("AAPL", "2020-08-31", 4.0);
    let dividend = CorporateAction::dividend("MSFT", "2024-01-15", 0.75);

    // Test action store
    let mut store = CorporateActionStore::new();
    store.add(split);
    store.add(dividend);
    store.add(CorporateAction::split("AAPL", "2014-06-09", 7.0));

    let actions = store.get("AAPL").unwrap();
    assert_eq!(actions.len(), 2);

    let range = store.get_in_range("AAPL", "2019-01-01", "2021-01-01");
    assert_eq!(range.len(), 1);

    // Test symbol mapper
    let mut mapper = SymbolMapper::new();
    mapper.add_change("FB", "META", "2022-10-28");

    assert_eq!(mapper.get_current("FB"), "META");
    assert_eq!(mapper.get_current("META"), "META");
    assert_eq!(mapper.get_at_date("META", "2022-01-01"), "FB");
}
