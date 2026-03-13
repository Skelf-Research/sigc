# sigc Language Reference

This document describes the syntax and semantics of the sigc DSL.

## Program Structure

A sigc program consists of four main sections:

```
data:
  <data declarations>

params:
  <parameter definitions>

signal <name>:
  <computations>
  emit <output>

portfolio <name>:
  <portfolio construction>
  backtest from <date> to <date>
```

## Data Section

Declare data sources:

```
data:
  prices: load csv from "data/prices.csv"
  volume: load parquet from "s3://bucket/volume.parquet"
```

Supported formats:
- `csv` - Comma-separated values
- `parquet` - Apache Parquet
- `arrow` - Apache Arrow IPC

## Parameters Section

Define tunable parameters with default values:

```
params:
  lookback = 10
  threshold = 0.5
  top_pct = 0.2
```

Parameters can be used anywhere in expressions and are optimizable via GridSearch.

## Signal Section

Define computed signals:

```
signal momentum:
  returns = ret(prices, lookback)
  normalized = zscore(returns)
  cleaned = winsor(normalized, p=0.01)
  emit cleaned
```

- Variable assignments: `name = expression`
- Final output: `emit expression`
- Each signal produces a time series

## Portfolio Section

Construct portfolio weights and run backtests:

```
portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Operators

### Time-Series Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `ret(x, n)` | Series, int → Series | n-period returns |
| `lag(x, n)` | Series, int → Series | Lag by n periods |
| `diff(x, n)` | Series, int → Series | n-period difference |
| `rolling_mean(x, n)` | Series, int → Series | Rolling mean |
| `rolling_std(x, n)` | Series, int → Series | Rolling std deviation |
| `rolling_sum(x, n)` | Series, int → Series | Rolling sum |
| `rolling_min(x, n)` | Series, int → Series | Rolling minimum |
| `rolling_max(x, n)` | Series, int → Series | Rolling maximum |

### Cross-Sectional Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `zscore(x)` | Series → Series | Standardize to z-scores |
| `rank(x)` | Series → Series | Rank (0 to 1) |
| `demean(x)` | Series → Series | Remove mean |
| `scale(x)` | Series → Series | Scale to sum to 1 |
| `winsor(x, p)` | Series, float → Series | Winsorize at p percentile |
| `quantile(x, q)` | Series, float → Series | q-th quantile |
| `bucket(x, n)` | Series, int → Series | Assign to n buckets |
| `median(x)` | Series → Series | Median value |
| `mad(x)` | Series → Series | Median absolute deviation |

### Technical Indicators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `rsi(x, n)` | Series, int → Series | Relative Strength Index |
| `macd(x, fast, slow, sig)` | Series, int, int, int → Series | MACD |
| `atr(h, l, c, n)` | Series×3, int → Series | Average True Range |
| `vwap(p, v)` | Series×2 → Series | Volume-Weighted Avg Price |

### Math Operators

| Operator | Signature | Description |
|----------|-----------|-------------|
| `abs(x)` | Series → Series | Absolute value |
| `sqrt(x)` | Series → Series | Square root |
| `floor(x)` | Series → Series | Floor |
| `ceil(x)` | Series → Series | Ceiling |
| `round(x)` | Series → Series | Round |
| `log(x)` | Series → Series | Natural logarithm |
| `exp(x)` | Series → Series | Exponential |

### Data Handling

| Operator | Signature | Description |
|----------|-----------|-------------|
| `is_nan(x)` | Series → Series | Check for NaN |
| `fill_nan(x, v)` | Series, float → Series | Fill NaN with value |
| `coalesce(x, y)` | Series×2 → Series | First non-NaN value |
| `cumsum(x)` | Series → Series | Cumulative sum |
| `cumprod(x)` | Series → Series | Cumulative product |
| `cummax(x)` | Series → Series | Cumulative maximum |
| `cummin(x)` | Series → Series | Cumulative minimum |

### Portfolio Construction

| Operator | Signature | Description |
|----------|-----------|-------------|
| `long_short(x, top, bottom)` | Series, float, float → Series | Long/short weights |
| `neutralize(x, factor)` | Series×2 → Series | Neutralize to factor |
| `clip(x, lo, hi)` | Series, float, float → Series | Clip to range |

## Comments

Single-line comments start with `//`:

```
// This is a comment
signal example:
  x = ret(prices, 5)  // Inline comment
  emit x
```

## Examples

### Momentum Strategy

```
data:
  prices: load parquet from "prices.parquet"

params:
  lookback = 20
  top = 0.1

signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit winsor(score, p=0.01)

portfolio strat:
  weights = rank(momentum).long_short(top=top, bottom=top)
  backtest from 2024-01-01 to 2024-12-31
```

### Mean Reversion Strategy

```
data:
  prices: load parquet from "prices.parquet"

params:
  window = 20
  entry = 2.0

signal mean_rev:
  ma = rolling_mean(prices, window)
  std = rolling_std(prices, window)
  zscore = (prices - ma) / std
  emit -1 * zscore  // Negative because we fade extremes

portfolio strat:
  weights = rank(mean_rev).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Multi-Factor Combination

```
data:
  prices: load parquet from "prices.parquet"
  volume: load parquet from "volume.parquet"

params:
  mom_period = 20
  vol_period = 10

signal momentum:
  r = ret(prices, mom_period)
  emit zscore(r)

signal volume_trend:
  v = rolling_mean(volume, vol_period)
  emit zscore(v)

signal combined:
  m = momentum
  v = volume_trend
  emit (m + v) / 2

portfolio strat:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```
