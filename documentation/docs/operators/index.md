# Operators

sigc includes 120+ built-in operators for signal construction.

## Operator Categories

| Category | Count | Description | Link |
|----------|-------|-------------|------|
| [Arithmetic](arithmetic.md) | 15 | Math operations | `abs`, `log`, `sqrt`, `pow` |
| [Comparison](comparison.md) | 6 | Relational operators | `>`, `<`, `>=`, `<=` |
| [Logical](logical.md) | 4 | Boolean logic | `and`, `or`, `not`, `where` |
| [Data Handling](data-handling.md) | 8 | Missing data, cumulative | `fill_nan`, `cumsum` |
| [Time-Series](time-series.md) | 20+ | Per-asset over time | `ret`, `lag`, `rolling_*` |
| [Cross-Sectional](cross-sectional.md) | 12 | Across assets | `zscore`, `rank`, `winsor` |
| [Technical](technical.md) | 4 | Technical indicators | `rsi`, `macd`, `atr` |
| [Portfolio](portfolio.md) | 3 | Weight construction | `long_short`, `neutralize` |

## Quick Reference

### Most Common Operators

```sig
// Returns
ret(prices, 20)           // 20-day return

// Time-series
lag(x, 5)                 // Lag by 5 periods
rolling_mean(x, 20)       // 20-day moving average
rolling_std(x, 20)        // 20-day rolling std

// Cross-sectional
zscore(x)                 // Standardize
rank(x)                   // Rank 0-1
winsor(x, p=0.01)         // Clip outliers

// Portfolio
rank(signal).long_short(top=0.2, bottom=0.2)
```

## Operator Signatures

Each operator has a signature:

```
operator(input1: Type, input2: Type, ...) -> OutputType
```

Example:

```
ret(x: Panel, n: Scalar) -> Panel
zscore(x: Panel) -> Panel
rolling_mean(x: Panel, window: Scalar) -> Panel
```

## Using Operators

### Basic Usage

```sig
signal example:
  returns = ret(prices, 20)
  normalized = zscore(returns)
  emit normalized
```

### Chaining

```sig
signal chained:
  step1 = ret(prices, 20)
  step2 = zscore(step1)
  step3 = winsor(step2, p=0.01)
  emit step3
```

### Composition

```sig
signal composed:
  emit winsor(zscore(ret(prices, 20)), p=0.01)
```

## Complete Operator Table

### Arithmetic

| Operator | Syntax | Description |
|----------|--------|-------------|
| `+` | `a + b` | Addition |
| `-` | `a - b` | Subtraction |
| `*` | `a * b` | Multiplication |
| `/` | `a / b` | Division |
| `abs` | `abs(x)` | Absolute value |
| `sign` | `sign(x)` | Sign (-1, 0, 1) |
| `log` | `log(x)` | Natural log |
| `exp` | `exp(x)` | Exponential |
| `pow` | `pow(x, n)` | Power |
| `sqrt` | `sqrt(x)` | Square root |
| `floor` | `floor(x)` | Round down |
| `ceil` | `ceil(x)` | Round up |
| `round` | `round(x)` | Round nearest |
| `min` | `min(a, b)` | Minimum |
| `max` | `max(a, b)` | Maximum |
| `clip` | `clip(x, lo, hi)` | Clamp range |

### Comparison

| Operator | Syntax | Description |
|----------|--------|-------------|
| `>` | `a > b` | Greater than |
| `<` | `a < b` | Less than |
| `>=` | `a >= b` | Greater or equal |
| `<=` | `a <= b` | Less or equal |
| `==` | `a == b` | Equal |
| `!=` | `a != b` | Not equal |

### Logical

| Operator | Syntax | Description |
|----------|--------|-------------|
| `and` | `a and b` | Logical AND |
| `or` | `a or b` | Logical OR |
| `not` | `not(x)` | Logical NOT |
| `where` | `where(c, a, b)` | Conditional |

### Data Handling

| Operator | Syntax | Description |
|----------|--------|-------------|
| `is_nan` | `is_nan(x)` | Check NaN |
| `fill_nan` | `fill_nan(x, v)` | Replace NaN |
| `coalesce` | `coalesce(a, b)` | First non-NaN |
| `cumsum` | `cumsum(x)` | Cumulative sum |
| `cumprod` | `cumprod(x)` | Cumulative product |
| `cummax` | `cummax(x)` | Cumulative max |
| `cummin` | `cummin(x)` | Cumulative min |

### Time-Series

| Operator | Syntax | Description |
|----------|--------|-------------|
| `lag` | `lag(x, n)` | Shift back |
| `ret` | `ret(x, n)` | n-period return |
| `delta` | `delta(x, n)` | n-period diff |
| `rolling_mean` | `rolling_mean(x, n)` | Moving avg |
| `rolling_std` | `rolling_std(x, n)` | Moving std |
| `rolling_sum` | `rolling_sum(x, n)` | Moving sum |
| `rolling_min` | `rolling_min(x, n)` | Moving min |
| `rolling_max` | `rolling_max(x, n)` | Moving max |
| `rolling_corr` | `rolling_corr(a, b, n)` | Moving corr |
| `ema` | `ema(x, span)` | Exponential MA |
| `decay_linear` | `decay_linear(x, n)` | Linear decay |
| `ts_argmax` | `ts_argmax(x, n)` | Index of max |
| `ts_argmin` | `ts_argmin(x, n)` | Index of min |
| `ts_rank` | `ts_rank(x, n)` | TS rank |
| `ts_skew` | `ts_skew(x, n)` | Rolling skew |
| `ts_kurt` | `ts_kurt(x, n)` | Rolling kurtosis |
| `ts_product` | `ts_product(x, n)` | Rolling product |
| `ts_zscore` | `ts_zscore(x, n)` | TS z-score |

### Cross-Sectional

| Operator | Syntax | Description |
|----------|--------|-------------|
| `zscore` | `zscore(x)` | Standardize |
| `rank` | `rank(x)` | Rank 0-1 |
| `rank_pct` | `rank_pct(x)` | Percentile |
| `scale` | `scale(x)` | Scale sum=1 |
| `demean` | `demean(x)` | Remove mean |
| `winsor` | `winsor(x, p)` | Winsorize |
| `neutralize` | `neutralize(x, g)` | Group neutral |
| `quantile` | `quantile(x, q)` | q-th quantile |
| `bucket` | `bucket(x, n)` | n buckets |
| `median` | `median(x)` | CS median |
| `mad` | `mad(x)` | Median abs dev |

### Technical

| Operator | Syntax | Description |
|----------|--------|-------------|
| `rsi` | `rsi(x, n)` | RSI |
| `macd` | `macd(x, f, s, sig)` | MACD |
| `atr` | `atr(h, l, c, n)` | ATR |
| `vwap` | `vwap(p, v)` | VWAP |

### Portfolio

| Operator | Syntax | Description |
|----------|--------|-------------|
| `long_short` | `long_short(top, bot)` | L/S weights |
| `neutralize` | `neutralize(x, by)` | Group neutral |
| `clip` | `clip(x, lo, hi)` | Bound values |

## Section Index

| Page | Description |
|------|-------------|
| [Arithmetic](arithmetic.md) | Math operations |
| [Comparison](comparison.md) | Relational operators |
| [Logical](logical.md) | Boolean logic |
| [Data Handling](data-handling.md) | Missing data, cumulative |
| [Time-Series](time-series.md) | Per-asset operators |
| [Cross-Sectional](cross-sectional.md) | Across-asset operators |
| [Technical](technical.md) | Technical indicators |
| [Portfolio](portfolio.md) | Weight construction |

## Next Steps

Start with [Time-Series](time-series.md) and [Cross-Sectional](cross-sectional.md) as these are the most commonly used operators.
