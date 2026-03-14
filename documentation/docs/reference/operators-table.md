# Operators Table

Complete reference of all sigc operators.

## Arithmetic Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `+` | `a + b` | Addition |
| `-` | `a - b` | Subtraction |
| `*` | `a * b` | Multiplication |
| `/` | `a / b` | Division |
| `%` | `a % b` | Modulo |
| `**` | `a ** b` | Power |
| `abs(x)` | `abs(x: Numeric) -> Numeric` | Absolute value |
| `sqrt(x)` | `sqrt(x: Numeric) -> Numeric` | Square root |
| `log(x)` | `log(x: Numeric) -> Numeric` | Natural logarithm |
| `log10(x)` | `log10(x: Numeric) -> Numeric` | Base-10 logarithm |
| `exp(x)` | `exp(x: Numeric) -> Numeric` | Exponential (e^x) |
| `pow(x, n)` | `pow(x: Numeric, n: Scalar) -> Numeric` | Power |
| `sign(x)` | `sign(x: Numeric) -> Numeric` | Sign (-1, 0, 1) |
| `floor(x)` | `floor(x: Numeric) -> Numeric` | Floor |
| `ceil(x)` | `ceil(x: Numeric) -> Numeric` | Ceiling |
| `round(x)` | `round(x: Numeric) -> Numeric` | Round to nearest |
| `min(a, b)` | `min(a: Numeric, b: Numeric) -> Numeric` | Element-wise minimum |
| `max(a, b)` | `max(a: Numeric, b: Numeric) -> Numeric` | Element-wise maximum |
| `clip(x, lo, hi)` | `clip(x: Numeric, lo: Scalar, hi: Scalar) -> Numeric` | Clip to range |

## Comparison Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `>` | `a > b` | Greater than |
| `<` | `a < b` | Less than |
| `>=` | `a >= b` | Greater than or equal |
| `<=` | `a <= b` | Less than or equal |
| `==` | `a == b` | Equal |
| `!=` | `a != b` | Not equal |

## Logical Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `and` | `a and b` | Logical AND |
| `or` | `a or b` | Logical OR |
| `not(x)` | `not(x: Boolean) -> Boolean` | Logical NOT |
| `where(cond, a, b)` | `where(cond: Boolean, a: T, b: T) -> T` | Conditional |

## Time-Series Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `lag(x, n)` | `lag(x: Panel, n: Scalar) -> Panel` | Shift back n periods |
| `lead(x, n)` | `lead(x: Panel, n: Scalar) -> Panel` | Shift forward n periods |
| `ret(x, n)` | `ret(x: Panel, n: Scalar) -> Panel` | N-period return |
| `delta(x, n)` | `delta(x: Panel, n: Scalar) -> Panel` | N-period difference |
| `rolling_mean(x, n)` | `rolling_mean(x: Panel, n: Scalar) -> Panel` | Rolling mean |
| `rolling_sum(x, n)` | `rolling_sum(x: Panel, n: Scalar) -> Panel` | Rolling sum |
| `rolling_std(x, n)` | `rolling_std(x: Panel, n: Scalar) -> Panel` | Rolling std dev |
| `rolling_var(x, n)` | `rolling_var(x: Panel, n: Scalar) -> Panel` | Rolling variance |
| `rolling_min(x, n)` | `rolling_min(x: Panel, n: Scalar) -> Panel` | Rolling minimum |
| `rolling_max(x, n)` | `rolling_max(x: Panel, n: Scalar) -> Panel` | Rolling maximum |
| `rolling_corr(x, y, n)` | `rolling_corr(x: Panel, y: Panel, n: Scalar) -> Panel` | Rolling correlation |
| `rolling_cov(x, y, n)` | `rolling_cov(x: Panel, y: Panel, n: Scalar) -> Panel` | Rolling covariance |
| `ema(x, n)` | `ema(x: Panel, n: Scalar) -> Panel` | Exponential moving average |
| `decay_linear(x, n)` | `decay_linear(x: Panel, n: Scalar) -> Panel` | Linear decay weighted avg |
| `ts_argmax(x, n)` | `ts_argmax(x: Panel, n: Scalar) -> Panel` | Index of max in window |
| `ts_argmin(x, n)` | `ts_argmin(x: Panel, n: Scalar) -> Panel` | Index of min in window |
| `ts_rank(x, n)` | `ts_rank(x: Panel, n: Scalar) -> Panel` | Time-series rank |
| `ts_zscore(x, n)` | `ts_zscore(x: Panel, n: Scalar) -> Panel` | Time-series z-score |
| `ts_skew(x, n)` | `ts_skew(x: Panel, n: Scalar) -> Panel` | Rolling skewness |
| `ts_kurt(x, n)` | `ts_kurt(x: Panel, n: Scalar) -> Panel` | Rolling kurtosis |
| `ts_product(x, n)` | `ts_product(x: Panel, n: Scalar) -> Panel` | Rolling product |

## Cross-Sectional Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `zscore(x)` | `zscore(x: Panel) -> Panel` | Cross-sectional z-score |
| `rank(x)` | `rank(x: Panel) -> Panel` | Cross-sectional rank (0-1) |
| `rank_pct(x)` | `rank_pct(x: Panel) -> Panel` | Percentile rank (0-100) |
| `scale(x)` | `scale(x: Panel) -> Panel` | Scale to sum to 1 |
| `demean(x)` | `demean(x: Panel) -> Panel` | Remove mean |
| `winsor(x, p)` | `winsor(x: Panel, p: Scalar) -> Panel` | Winsorize at percentile |
| `neutralize(x, by)` | `neutralize(x: Panel, by: Panel) -> Panel` | Group neutralization |
| `quantile(x, q)` | `quantile(x: Panel, q: Scalar) -> Panel` | Cross-sectional quantile |
| `bucket(x, n)` | `bucket(x: Panel, n: Scalar) -> Panel` | Assign to n buckets |
| `median(x)` | `median(x: Panel) -> Panel` | Cross-sectional median |
| `mad(x)` | `mad(x: Panel) -> Panel` | Median absolute deviation |
| `mean(x)` | `mean(x: Panel) -> Scalar` | Cross-sectional mean |
| `std(x)` | `std(x: Panel) -> Scalar` | Cross-sectional std dev |
| `sum(x)` | `sum(x: Panel) -> Scalar` | Cross-sectional sum |
| `count(x)` | `count(x: Panel) -> Scalar` | Count non-null values |

## Technical Indicators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `rsi(x, n)` | `rsi(x: Panel, n: Scalar) -> Panel` | Relative Strength Index |
| `macd(x, fast, slow, signal)` | `macd(x: Panel, fast: Scalar, slow: Scalar, signal: Scalar) -> Panel` | MACD histogram |
| `atr(high, low, close, n)` | `atr(high: Panel, low: Panel, close: Panel, n: Scalar) -> Panel` | Average True Range |
| `vwap(price, volume)` | `vwap(price: Panel, volume: Panel) -> Panel` | Volume-weighted avg price |

## Portfolio Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `long_short(top, bottom, cap)` | `rank.long_short(top: Scalar, bottom: Scalar, cap: Scalar = None) -> Panel` | Long-short weights |

## Data Handling Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `is_nan(x)` | `is_nan(x: Numeric) -> Boolean` | Check for NaN |
| `fill_nan(x, value)` | `fill_nan(x: Numeric, value: Scalar) -> Numeric` | Replace NaN |
| `coalesce(a, b)` | `coalesce(a: Numeric, b: Numeric) -> Numeric` | First non-NaN |
| `cumsum(x)` | `cumsum(x: Numeric) -> Numeric` | Cumulative sum |
| `cumprod(x)` | `cumprod(x: Numeric) -> Numeric` | Cumulative product |
| `cummax(x)` | `cummax(x: Numeric) -> Numeric` | Cumulative maximum |
| `cummin(x)` | `cummin(x: Numeric) -> Numeric` | Cumulative minimum |

## Operator Categories

### By Input Type

| Category | Operators |
|----------|-----------|
| Panel → Panel | `zscore`, `rank`, `rolling_*`, `lag`, `ret` |
| Panel → Scalar | `mean`, `std`, `sum`, `count` |
| Scalar → Scalar | `abs`, `sqrt`, `log`, `exp` |
| Boolean → Boolean | `and`, `or`, `not` |

### By Application

| Application | Operators |
|-------------|-----------|
| Signal Normalization | `zscore`, `rank`, `winsor`, `demean` |
| Momentum | `ret`, `rolling_mean`, `ema` |
| Volatility | `rolling_std`, `atr` |
| Mean Reversion | `zscore`, `ts_zscore` |
| Portfolio Construction | `long_short`, `scale`, `clip` |

## Quick Reference

### Most Common

```sig
// Normalization
zscore(x)              // Z-score normalize
rank(x)                // Rank 0-1
winsor(x, 0.01)        // Handle outliers

// Time-Series
ret(prices, 60)        // 60-day return
rolling_mean(x, 20)    // 20-day moving average
rolling_std(x, 20)     // 20-day volatility
lag(x, 1)              // Previous value

// Portfolio
rank(signal).long_short(top=0.2, bottom=0.2)
```

### Signal Construction

```sig
// Momentum
momentum = zscore(ret(prices, 60))

// Value
value = zscore(book_to_market)

// Quality
quality = zscore(roe)

// Low Volatility
low_vol = -zscore(rolling_std(ret(prices, 1), 252))

// Combine
composite = 0.25 * momentum + 0.25 * value + 0.25 * quality + 0.25 * low_vol
```

## See Also

- [Time-Series Operators](../operators/time-series.md) - Detailed documentation
- [Cross-Sectional Operators](../operators/cross-sectional.md) - Detailed documentation
- [Technical Indicators](../operators/technical.md) - Detailed documentation
