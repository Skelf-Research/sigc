# Bollinger Bands Strategy

Mean reversion using Bollinger Bands.

## Strategy Overview

Bollinger Bands create dynamic support and resistance levels. Trade reversions when prices touch the bands.

## Bollinger Bands

```
Middle Band = 20-day SMA
Upper Band = Middle + 2 × 20-day Std Dev
Lower Band = Middle - 2 × 20-day Std Dev
```

## The Signal

```sig
signal bollinger:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)

  upper = ma + 2 * std
  lower = ma - 2 * std

  // Position within bands (0 to 1)
  band_position = (prices - lower) / (upper - lower)

  // Buy at lower band, sell at upper
  emit zscore(0.5 - band_position)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Bollinger Bands
signal bollinger_bands:
  ma_20 = rolling_mean(prices, 20)
  std_20 = rolling_std(prices, 20)

  upper = ma_20 + 2 * std_20
  lower = ma_20 - 2 * std_20

  // Normalized position (0 = lower, 1 = upper)
  position = (prices - lower) / (upper - lower)

  // Revert from extremes
  signal = 0.5 - position

  emit neutralize(zscore(signal), by=sectors)

portfolio bollinger_strategy:
  weights = rank(bollinger_bands).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## Variations

### Band Width Filter

```sig
signal bandwidth_filtered:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)

  upper = ma + 2 * std
  lower = ma - 2 * std

  // Band width
  bandwidth = (upper - lower) / ma

  // Wider bands = higher volatility = stronger signals
  wide_bands = bandwidth > rolling_mean(bandwidth, 60)

  position = (prices - lower) / (upper - lower)
  base_signal = 0.5 - position

  // Stronger signal when bands are wide
  emit where(wide_bands, base_signal * 1.3, base_signal)
```

### Double Bollinger

```sig
signal double_bollinger:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)

  // Inner bands (1 std)
  inner_upper = ma + 1 * std
  inner_lower = ma - 1 * std

  // Outer bands (2 std)
  outer_upper = ma + 2 * std
  outer_lower = ma - 2 * std

  // Zones
  extreme_high = prices > outer_upper
  extreme_low = prices < outer_lower
  moderate_high = prices > inner_upper and prices <= outer_upper
  moderate_low = prices < inner_lower and prices >= outer_lower

  signal = where(extreme_low, 2,
           where(moderate_low, 1,
           where(extreme_high, -2,
           where(moderate_high, -1, 0))))

  emit zscore(signal)
```

### Bollinger Squeeze

```sig
signal bollinger_squeeze:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)

  bandwidth = (2 * std) / ma

  // Squeeze = narrow bands (low volatility)
  squeeze = bandwidth < rolling_min(bandwidth, 120)

  // After squeeze, expect volatility expansion
  // Don't mean revert during squeeze
  position = (prices - (ma - 2*std)) / (4 * std)
  base_signal = 0.5 - position

  emit where(squeeze, 0, base_signal)
```

### Multi-Period Bands

```sig
signal multi_period:
  // Short-term bands
  ma_10 = rolling_mean(prices, 10)
  std_10 = rolling_std(prices, 10)
  pos_10 = (prices - (ma_10 - 2*std_10)) / (4 * std_10)

  // Medium-term bands
  ma_20 = rolling_mean(prices, 20)
  std_20 = rolling_std(prices, 20)
  pos_20 = (prices - (ma_20 - 2*std_20)) / (4 * std_20)

  // Long-term bands
  ma_50 = rolling_mean(prices, 50)
  std_50 = rolling_std(prices, 50)
  pos_50 = (prices - (ma_50 - 2*std_50)) / (4 * std_50)

  // Average position
  avg_pos = (pos_10 + pos_20 + pos_50) / 3

  emit zscore(0.5 - avg_pos)
```

## With Trend Confirmation

```sig
signal trend_confirmed:
  ma_20 = rolling_mean(prices, 20)
  std_20 = rolling_std(prices, 20)

  upper = ma_20 + 2 * std_20
  lower = ma_20 - 2 * std_20

  // Band touches
  at_lower = prices <= lower
  at_upper = prices >= upper

  // Trend filter
  ma_50 = rolling_mean(prices, 50)
  uptrend = prices > ma_50
  downtrend = prices < ma_50

  // In uptrend, buy at lower band is stronger
  // In downtrend, short at upper band is stronger
  signal = where(at_lower and uptrend, 1.5,
           where(at_lower, 1.0,
           where(at_upper and downtrend, -1.5,
           where(at_upper, -1.0, 0))))

  emit zscore(signal)
```

## %B Indicator

```sig
signal percent_b:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)

  upper = ma + 2 * std
  lower = ma - 2 * std

  // %B = position within bands
  // 0 = at lower, 1 = at upper, negative = below lower
  pct_b = (prices - lower) / (upper - lower)

  // Extreme readings
  oversold = pct_b < 0
  overbought = pct_b > 1

  signal = where(oversold, 1 - pct_b,
           where(overbought, 1 - pct_b,
           0.5 - pct_b))

  emit zscore(signal)
```

## Expected Results

```
Backtest Results: bollinger_strategy
====================================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 52%
  Annual Return: 4.5%
  Annual Volatility: 7.5%
  Sharpe Ratio: 0.60

Band Statistics:
  Avg Position (longs): 0.18
  Avg Position (shorts): 0.82
  Band Touches: 15% of signals
```

## See Also

- [Statistical Reversion](statistical-reversion.md)
- [Short-Term Reversal](short-term-reversal.md)
