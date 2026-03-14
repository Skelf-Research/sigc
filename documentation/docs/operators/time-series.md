# Time-Series Operators

Operators that compute values over time for each asset independently.

## Basic Operations

### `lag(x, n)`

Shift values back by n periods.

```sig
lag(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  previous_price = lag(prices, 1)     // Yesterday's price
  week_ago_price = lag(prices, 5)     // 5 days ago
  month_ago_price = lag(prices, 21)   // ~1 month ago
  emit previous_price
```

### `ret(x, n)`

Compute n-period return: (x - lag(x, n)) / lag(x, n)

```sig
ret(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  daily_return = ret(prices, 1)       // 1-day return
  weekly_return = ret(prices, 5)      // 5-day return
  monthly_return = ret(prices, 21)    // ~1-month return
  annual_return = ret(prices, 252)    // ~1-year return
  emit monthly_return
```

### `delta(x, n)`

Compute n-period difference: x - lag(x, n)

```sig
delta(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  price_change = delta(prices, 1)     // $ change
  volume_change = delta(volume, 5)    // Volume change
  emit price_change
```

## Rolling Statistics

### `rolling_mean(x, n)`

Rolling mean (moving average).

```sig
rolling_mean(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  sma_20 = rolling_mean(prices, 20)   // 20-day SMA
  sma_50 = rolling_mean(prices, 50)   // 50-day SMA
  avg_volume = rolling_mean(volume, 20)
  emit sma_20
```

### `rolling_std(x, n)`

Rolling standard deviation.

```sig
rolling_std(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, 20)  // 20-day vol
  annualized_vol = volatility * sqrt(252)
  emit volatility
```

### `rolling_sum(x, n)`

Rolling sum.

```sig
rolling_sum(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  volume_20d = rolling_sum(volume, 20)
  cum_return_5d = rolling_sum(ret(prices, 1), 5)
  emit volume_20d
```

### `rolling_min(x, n)`

Rolling minimum.

```sig
rolling_min(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  low_20d = rolling_min(prices, 20)
  support = rolling_min(prices, 52 * 5)  // 52-week low
  emit low_20d
```

### `rolling_max(x, n)`

Rolling maximum.

```sig
rolling_max(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  high_20d = rolling_max(prices, 20)
  resistance = rolling_max(prices, 52 * 5)  // 52-week high
  emit high_20d
```

### `rolling_corr(x, y, n)`

Rolling correlation between two series.

```sig
rolling_corr(x: Panel, y: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Correlation with market
  market_corr = rolling_corr(stock_returns, market_returns, 60)

  // Pairs correlation
  pair_corr = rolling_corr(ret(stock_a, 1), ret(stock_b, 1), 20)

  emit market_corr
```

## Exponential Moving Average

### `ema(x, span)`

Exponential moving average.

```sig
ema(x: Panel, span: Scalar) -> Panel
```

```sig
signal example:
  ema_10 = ema(prices, 10)
  ema_20 = ema(prices, 20)
  ema_50 = ema(prices, 50)

  // EMA crossover signal
  crossover = ema_10 - ema_50

  emit crossover
```

EMA formula: $\alpha = 2 / (span + 1)$

### `decay_linear(x, n)`

Linear decay weighted average.

```sig
decay_linear(x: Panel, n: Scalar) -> Panel
```

Recent values have higher weights:

```sig
signal example:
  // Recent days weighted more heavily
  weighted_return = decay_linear(ret(prices, 1), 20)
  emit weighted_return
```

## Advanced Time-Series

### `ts_argmax(x, n)`

Index of maximum value in window (days since max).

```sig
ts_argmax(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Days since 20-day high
  days_since_high = ts_argmax(prices, 20)
  emit days_since_high
```

### `ts_argmin(x, n)`

Index of minimum value in window (days since min).

```sig
ts_argmin(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Days since 20-day low
  days_since_low = ts_argmin(prices, 20)
  emit days_since_low
```

### `ts_rank(x, n)`

Time-series rank (rank within asset's own history).

```sig
ts_rank(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // How current price ranks vs last 20 days
  ts_rank_20 = ts_rank(prices, 20)
  emit ts_rank_20
```

### `ts_skew(x, n)`

Rolling skewness.

```sig
ts_skew(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  return_skew = ts_skew(ret(prices, 1), 60)
  emit return_skew
```

### `ts_kurt(x, n)`

Rolling kurtosis.

```sig
ts_kurt(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  return_kurt = ts_kurt(ret(prices, 1), 60)
  // High kurtosis = fat tails
  emit return_kurt
```

### `ts_product(x, n)`

Rolling product.

```sig
ts_product(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Cumulative return over window
  growth_factors = 1 + ret(prices, 1)
  cum_return = ts_product(growth_factors, 20) - 1
  emit cum_return
```

### `ts_zscore(x, n)`

Time-series z-score (normalize within asset's history).

```sig
ts_zscore(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Current price relative to own history
  price_zscore = ts_zscore(prices, 252)
  // Positive = above historical mean
  emit price_zscore
```

## Common Patterns

### Momentum

```sig
signal momentum:
  // 12-month return minus 1-month return
  r12 = ret(prices, 252)
  r1 = ret(prices, 21)
  mom = r12 - r1
  emit zscore(mom)
```

### Moving Average Crossover

```sig
signal ma_crossover:
  fast = ema(prices, 10)
  slow = ema(prices, 50)
  crossover = (fast - slow) / slow
  emit zscore(crossover)
```

### Volatility

```sig
signal volatility:
  daily_ret = ret(prices, 1)
  vol_20 = rolling_std(daily_ret, 20)
  vol_60 = rolling_std(daily_ret, 60)
  // Ratio of short to long vol
  vol_ratio = vol_20 / vol_60
  emit zscore(vol_ratio)
```

### Bollinger Bands

```sig
signal bollinger:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  z = (prices - ma) / std
  emit z  // Position within bands
```

### Relative Strength

```sig
signal relative_strength:
  // Price relative to 52-week range
  high_52 = rolling_max(prices, 252)
  low_52 = rolling_min(prices, 252)
  rs = (prices - low_52) / (high_52 - low_52)
  emit rs
```

### Channel Breakout

```sig
signal breakout:
  high_20 = rolling_max(prices, 20)
  low_20 = rolling_min(prices, 20)
  mid = (high_20 + low_20) / 2

  // Position in channel (0 = bottom, 1 = top)
  position = (prices - low_20) / (high_20 - low_20)
  emit position
```

## Warm-up Period

Rolling operators require a warm-up period:

```sig
// rolling_mean(x, 20) produces NaN for first 19 rows
// Because there aren't enough observations yet
```

Handle with fill_nan if needed:

```sig
signal handle_warmup:
  raw = rolling_mean(prices, 20)
  safe = fill_nan(raw, 0)
  emit safe
```

## Type Behavior

All time-series operators:

- Input: Panel (dates × assets)
- Output: Panel (same shape)
- Operate on each asset independently

## Next Steps

- [Cross-Sectional](cross-sectional.md) - Across-asset operators
- [Technical](technical.md) - Technical indicators
- [Signal Section](../language/signal-section.md) - Using in signals
