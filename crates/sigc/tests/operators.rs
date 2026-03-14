//! Operator coverage tests
//!
//! Ensures all sigc operators compile correctly.
//! These tests verify the parser and compiler accept valid operator syntax.

/// Helper to test an operator compiles
fn test_operator_compiles(signal_body: &str) {
    let source = format!(
        r#"
data:
  px: load csv from "test.csv"

params:
  n = 10

signal test:
{}
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#,
        signal_body
    );

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(&source);

    assert!(
        result.is_ok(),
        "Operator failed to compile: {:?}",
        result.err()
    );

    let ir = result.unwrap();
    assert!(!ir.nodes.is_empty(), "Should produce IR nodes");
}

// ============================================================================
// Arithmetic Operators
// ============================================================================

#[test]
fn test_op_addition() {
    test_operator_compiles("  result = px + 1");
}

#[test]
fn test_op_subtraction() {
    test_operator_compiles("  result = px - 1");
}

#[test]
fn test_op_multiplication() {
    test_operator_compiles("  result = px * 2");
}

#[test]
fn test_op_division() {
    test_operator_compiles("  result = px / 2");
}

#[test]
fn test_op_abs() {
    test_operator_compiles("  result = abs(px - 100)");
}

#[test]
fn test_op_sign() {
    test_operator_compiles("  result = sign(px - 100)");
}

#[test]
fn test_op_log() {
    test_operator_compiles("  result = log(px)");
}

#[test]
fn test_op_exp() {
    test_operator_compiles("  result = exp(px / 100)");
}

#[test]
fn test_op_pow() {
    test_operator_compiles("  result = pow(px / 100, 2)");
}

#[test]
fn test_op_sqrt() {
    test_operator_compiles("  result = sqrt(px)");
}

#[test]
fn test_op_floor() {
    test_operator_compiles("  result = floor(px)");
}

#[test]
fn test_op_ceil() {
    test_operator_compiles("  result = ceil(px)");
}

#[test]
fn test_op_round() {
    test_operator_compiles("  result = round(px)");
}

#[test]
fn test_op_min() {
    test_operator_compiles("  result = min(px, 105)");
}

#[test]
fn test_op_max() {
    test_operator_compiles("  result = max(px, 105)");
}

#[test]
fn test_op_clip() {
    test_operator_compiles("  result = clip(px, 95, 110)");
}

// ============================================================================
// Data Handling Operators
// ============================================================================

#[test]
fn test_op_fill_nan() {
    test_operator_compiles("  result = fill_nan(px, 0)");
}

#[test]
fn test_op_coalesce() {
    test_operator_compiles("  result = coalesce(px, 100)");
}

#[test]
fn test_op_cumsum() {
    test_operator_compiles("  result = cumsum(ret(px, 1))");
}

#[test]
fn test_op_cumprod() {
    test_operator_compiles("  result = cumprod(1 + ret(px, 1) / 100)");
}

#[test]
fn test_op_cummax() {
    test_operator_compiles("  result = cummax(px)");
}

#[test]
fn test_op_cummin() {
    test_operator_compiles("  result = cummin(px)");
}

// ============================================================================
// Time-Series Operators
// ============================================================================

#[test]
fn test_op_lag() {
    test_operator_compiles("  result = lag(px, n)");
}

#[test]
fn test_op_ret() {
    test_operator_compiles("  result = ret(px, n)");
}

#[test]
fn test_op_delta() {
    test_operator_compiles("  result = delta(px, n)");
}

#[test]
fn test_op_rolling_mean() {
    test_operator_compiles("  result = rolling_mean(px, n)");
}

#[test]
fn test_op_rolling_std() {
    test_operator_compiles("  result = rolling_std(px, n)");
}

#[test]
fn test_op_rolling_sum() {
    test_operator_compiles("  result = rolling_sum(px, n)");
}

#[test]
fn test_op_rolling_min() {
    test_operator_compiles("  result = rolling_min(px, n)");
}

#[test]
fn test_op_rolling_max() {
    test_operator_compiles("  result = rolling_max(px, n)");
}

#[test]
fn test_op_ema() {
    test_operator_compiles("  result = ema(px, n)");
}

#[test]
fn test_op_decay_linear() {
    test_operator_compiles("  result = decay_linear(px, n)");
}

#[test]
fn test_op_ts_argmax() {
    test_operator_compiles("  result = ts_argmax(px, n)");
}

#[test]
fn test_op_ts_argmin() {
    test_operator_compiles("  result = ts_argmin(px, n)");
}

#[test]
fn test_op_ts_rank() {
    test_operator_compiles("  result = ts_rank(px, n)");
}

#[test]
fn test_op_ts_skew() {
    test_operator_compiles("  result = ts_skew(px, n)");
}

#[test]
fn test_op_ts_kurt() {
    test_operator_compiles("  result = ts_kurt(px, n)");
}

#[test]
fn test_op_ts_product() {
    test_operator_compiles("  result = ts_product(1 + ret(px, 1) / 100, n)");
}

#[test]
fn test_op_ts_zscore() {
    test_operator_compiles("  result = ts_zscore(px, n)");
}

// ============================================================================
// Cross-Sectional Operators
// ============================================================================

#[test]
fn test_op_zscore() {
    test_operator_compiles("  result = zscore(px)");
}

#[test]
fn test_op_rank() {
    test_operator_compiles("  result = rank(px)");
}

#[test]
fn test_op_rank_pct() {
    test_operator_compiles("  result = rank_pct(px)");
}

#[test]
fn test_op_scale() {
    test_operator_compiles("  result = scale(abs(px))");
}

#[test]
fn test_op_demean() {
    test_operator_compiles("  result = demean(px)");
}

#[test]
fn test_op_winsor() {
    test_operator_compiles("  result = winsor(px, p=0.01)");
}

#[test]
fn test_op_quantile() {
    test_operator_compiles("  result = quantile(px, q=0.5)");
}

#[test]
fn test_op_bucket() {
    test_operator_compiles("  result = bucket(px, n=5)");
}

#[test]
fn test_op_median() {
    test_operator_compiles("  result = median(px)");
}

#[test]
fn test_op_mad() {
    test_operator_compiles("  result = mad(px)");
}

// ============================================================================
// Technical Indicators
// ============================================================================

#[test]
fn test_op_rsi() {
    test_operator_compiles("  result = rsi(px, n)");
}

#[test]
fn test_op_macd() {
    test_operator_compiles("  result = macd(px, 12, 26, 9)");
}

// ============================================================================
// Complex Expression Tests
// ============================================================================

#[test]
fn test_complex_arithmetic() {
    test_operator_compiles(
        r#"  r = ret(px, n)
  vol = rolling_std(r, n)
  result = r / vol"#,
    );
}

#[test]
fn test_nested_functions() {
    test_operator_compiles("  result = zscore(winsor(ret(px, n), p=0.01))");
}

#[test]
fn test_chained_operations() {
    test_operator_compiles(
        r#"  r = ret(px, n)
  z = zscore(r)
  w = winsor(z, p=0.01)
  result = clip(w, -3, 3)"#,
    );
}

#[test]
fn test_multiple_inputs() {
    let source = r#"
data:
  px: load csv from "test.csv"
  vol: load csv from "volume.csv"

params:
  n = 10

signal test:
  r = ret(px, n)
  v = rolling_mean(vol, n)
  result = r * v
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Multiple inputs should compile: {:?}", result.err());
}

#[test]
fn test_signal_combination() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  short = 5
  long = 20

signal fast:
  r = ret(px, short)
  emit zscore(r)

signal slow:
  r = ret(px, long)
  emit zscore(r)

signal combined:
  emit 0.5 * fast + 0.5 * slow

portfolio main:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Signal combination should compile: {:?}", result.err());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_constant_expression() {
    test_operator_compiles("  result = px * 0 + 1");
}

#[test]
fn test_identity_transform() {
    // Simple identity may not produce IR nodes, just check it compiles
    let source = r#"
data:
  px: load csv from "test.csv"

signal test:
  result = px
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Identity transform should compile: {:?}", result.err());
}

#[test]
fn test_double_negation() {
    test_operator_compiles("  result = -(-px)");
}

#[test]
fn test_self_division() {
    test_operator_compiles("  result = px / px");
}

#[test]
fn test_large_window() {
    let source = r#"
data:
  px: load csv from "test.csv"

params:
  n = 50

signal test:
  result = rolling_mean(px, n)
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_ok(), "Large window should compile: {:?}", result.err());
}

// ============================================================================
// Error Detection Tests
// ============================================================================

#[test]
fn test_undefined_variable_error() {
    let source = r#"
data:
  px: load csv from "test.csv"

signal test:
  result = undefined_var + 1
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "Should fail on undefined variable");
}

#[test]
fn test_syntax_error_missing_colon() {
    let source = r#"
data:
  px: load csv from "test.csv"

signal test
  result = px
  emit result

portfolio main:
  weights = rank(test).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "Should fail on missing colon");
}

#[test]
fn test_empty_signal_error() {
    let source = r#"
data:
  px: load csv from "test.csv"

signal empty:

portfolio main:
  weights = rank(empty).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"#;

    let compiler = sig_compiler::Compiler::new();
    let result = compiler.compile(source);
    assert!(result.is_err(), "Should fail on empty signal");
}

// ============================================================================
// Parentheses and Precedence Tests
// ============================================================================

#[test]
fn test_parentheses_grouping() {
    test_operator_compiles("  result = (px + 1) * 2");
}

#[test]
fn test_nested_parentheses() {
    test_operator_compiles("  result = ((px + 1) * 2) / 3");
}

#[test]
fn test_operator_precedence() {
    test_operator_compiles("  result = px + 1 * 2");  // Should be px + (1*2)
}

// ============================================================================
// Keyword Argument Tests
// ============================================================================

#[test]
fn test_kwarg_single() {
    test_operator_compiles("  result = winsor(px, p=0.05)");
}

#[test]
fn test_kwarg_multiple() {
    // Test function with multiple kwargs if available
    test_operator_compiles("  result = clip(px, lo=90, hi=110)");
}

// ============================================================================
// Long Signal Chain Tests
// ============================================================================

#[test]
fn test_long_computation_chain() {
    test_operator_compiles(
        r#"  r1 = ret(px, 5)
  r2 = ret(px, 10)
  r3 = ret(px, 20)
  avg = (r1 + r2 + r3) / 3
  z = zscore(avg)
  w = winsor(z, p=0.01)
  result = clip(w, -3, 3)"#,
    );
}

#[test]
fn test_multiple_rolling_operations() {
    test_operator_compiles(
        r#"  ma5 = rolling_mean(px, 5)
  ma10 = rolling_mean(px, n)
  ma20 = rolling_mean(px, 20)
  trend = (ma5 - ma20) / ma20
  result = zscore(trend)"#,
    );
}
