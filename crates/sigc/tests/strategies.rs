//! Strategy library tests
//!
//! Tests that all strategy files in the strategies/ directory compile correctly.

use std::fs;
use std::path::Path;

/// Test that a strategy file compiles successfully
fn test_strategy_compiles(path: &str) {
    let source = fs::read_to_string(path).expect(&format!("Failed to read {}", path));
    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(&source);

    assert!(
        result.is_ok(),
        "Strategy {} failed to compile: {:?}",
        path,
        result.err()
    );

    let ir = result.unwrap();
    assert!(!ir.nodes.is_empty(), "Strategy {} produced no IR nodes", path);
    assert!(
        !ir.outputs.is_empty(),
        "Strategy {} produced no outputs",
        path
    );
}

// ============================================================================
// Momentum Strategies
// ============================================================================

#[test]
fn test_strategy_time_series_momentum() {
    test_strategy_compiles("../../strategies/momentum/time_series_momentum.sig");
}

#[test]
fn test_strategy_cross_sectional_momentum() {
    test_strategy_compiles("../../strategies/momentum/cross_sectional_momentum.sig");
}

#[test]
fn test_strategy_momentum_crash_protected() {
    test_strategy_compiles("../../strategies/momentum/momentum_crash_protected.sig");
}

#[test]
fn test_strategy_momentum_quality() {
    test_strategy_compiles("../../strategies/momentum/momentum_quality.sig");
}

// ============================================================================
// Mean Reversion Strategies
// ============================================================================

#[test]
fn test_strategy_short_term_reversal() {
    test_strategy_compiles("../../strategies/mean_reversion/short_term_reversal.sig");
}

#[test]
fn test_strategy_bollinger_mean_reversion() {
    test_strategy_compiles("../../strategies/mean_reversion/bollinger_mean_reversion.sig");
}

#[test]
fn test_strategy_pairs_trading() {
    test_strategy_compiles("../../strategies/mean_reversion/pairs_trading.sig");
}

#[test]
fn test_strategy_sector_rotation() {
    test_strategy_compiles("../../strategies/mean_reversion/sector_rotation.sig");
}

// ============================================================================
// Volatility Strategies
// ============================================================================

#[test]
fn test_strategy_low_volatility() {
    test_strategy_compiles("../../strategies/volatility/low_volatility.sig");
}

#[test]
fn test_strategy_volatility_timing() {
    test_strategy_compiles("../../strategies/volatility/volatility_timing.sig");
}

#[test]
fn test_strategy_variance_risk_premium() {
    test_strategy_compiles("../../strategies/volatility/variance_risk_premium.sig");
}

#[test]
fn test_strategy_vol_of_vol() {
    test_strategy_compiles("../../strategies/volatility/vol_of_vol.sig");
}

// ============================================================================
// Multi-Factor Strategies
// ============================================================================

#[test]
fn test_strategy_value_momentum() {
    test_strategy_compiles("../../strategies/multi_factor/value_momentum.sig");
}

#[test]
fn test_strategy_fama_french() {
    test_strategy_compiles("../../strategies/multi_factor/fama_french.sig");
}

#[test]
fn test_strategy_quality_value_momentum() {
    test_strategy_compiles("../../strategies/multi_factor/quality_value_momentum.sig");
}

#[test]
fn test_strategy_defensive_equity() {
    test_strategy_compiles("../../strategies/multi_factor/defensive_equity.sig");
}

// ============================================================================
// Technical Strategies
// ============================================================================

#[test]
fn test_strategy_trend_following() {
    test_strategy_compiles("../../strategies/technical/trend_following.sig");
}

#[test]
fn test_strategy_rsi() {
    test_strategy_compiles("../../strategies/technical/rsi_strategy.sig");
}

#[test]
fn test_strategy_macd_momentum() {
    test_strategy_compiles("../../strategies/technical/macd_momentum.sig");
}

#[test]
fn test_strategy_breakout() {
    test_strategy_compiles("../../strategies/technical/breakout.sig");
}

// ============================================================================
// Statistical Arbitrage Strategies
// ============================================================================

#[test]
fn test_strategy_residual_momentum() {
    test_strategy_compiles("../../strategies/statistical_arbitrage/residual_momentum.sig");
}

#[test]
fn test_strategy_industry_momentum() {
    test_strategy_compiles("../../strategies/statistical_arbitrage/industry_momentum.sig");
}

#[test]
fn test_strategy_factor_timing() {
    test_strategy_compiles("../../strategies/statistical_arbitrage/factor_timing.sig");
}

// ============================================================================
// Strategy Metadata Tests
// ============================================================================

#[test]
fn test_strategy_metadata_extraction() {
    let source = r#"
data:
  px: load csv from "data/prices.csv"

params:
  lookback = 20
  threshold = 0.5
  decay = 0.94

signal test:
  r = ret(px, lookback)
  emit zscore(r)

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    // Should extract all parameters
    assert_eq!(ir.metadata.parameters.len(), 3);

    // Should extract data source
    assert_eq!(ir.metadata.data_sources.len(), 1);
}

#[test]
fn test_multiple_signals_and_dependencies() {
    let source = r#"
data:
  px: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"

params:
  short = 5
  long = 20

signal momentum:
  r = ret(px, long)
  emit zscore(r)

signal volume_signal:
  v = rolling_mean(volume, short)
  emit zscore(v)

signal combined:
  emit 0.6 * momentum + 0.4 * volume_signal

portfolio main:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let ir = compiler.compile(source).unwrap();

    // Should have nodes from all signals
    assert!(ir.nodes.len() > 5, "Should have multiple IR nodes");

    // Should have outputs
    assert!(!ir.outputs.is_empty());
}

#[test]
fn test_strategy_with_all_param_types() {
    let source = r#"
data:
  px: load csv from "data/prices.csv"

params:
  int_param = 10
  float_param = 0.5
  large_param = 252

signal test:
  r = ret(px, int_param)
  scaled = r * float_param
  ma = rolling_mean(px, large_param)
  emit zscore(scaled)

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok());
}
