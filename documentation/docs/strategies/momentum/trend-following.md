# Trend Following Strategy

Systematic strategies using moving average crossovers and trend indicators.

## Strategy Overview

Follow price trends using technical indicators. Buy when prices trend up, sell when they trend down.

## Basic Moving Average Crossover

```sig
signal ma_crossover:
  ma_fast = rolling_mean(prices, 50)
  ma_slow = rolling_mean(prices, 200)

  // Above slow MA = bullish
  trend = (ma_fast - ma_slow) / ma_slow

  emit zscore(trend)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Trend signal
signal trend:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)

  // Percentage above/below slow MA
  trend_strength = (ma_50 - ma_200) / ma_200

  // Sector neutralize
  emit neutralize(zscore(trend_strength), by=sectors)

// Portfolio
portfolio trend_following:
  weights = rank(trend).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.25

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Variations

### Triple Moving Average

```sig
signal triple_ma:
  ma_20 = rolling_mean(prices, 20)
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)

  // All aligned = strong trend
  bull_aligned = (ma_20 > ma_50) and (ma_50 > ma_200)
  bear_aligned = (ma_20 < ma_50) and (ma_50 < ma_200)

  signal = where(bull_aligned, 1,
           where(bear_aligned, -1, 0))

  // Weight by alignment strength
  alignment = (ma_20 - ma_200) / ma_200

  emit zscore(signal * abs(alignment))
```

### Exponential Moving Average

```sig
signal ema_trend:
  ema_20 = ema(prices, 20)
  ema_50 = ema(prices, 50)

  trend = (ema_20 - ema_50) / ema_50

  emit zscore(trend)
```

### Price vs Moving Average

```sig
signal price_ma_deviation:
  ma_50 = rolling_mean(prices, 50)

  // How far above/below the MA
  deviation = (prices - ma_50) / ma_50

  emit zscore(deviation)
```

## Trend Strength Indicators

### ADX-Based Trend

```sig
signal adx_trend:
  // Simplified ADX calculation
  high_change = high - lag(high, 1)
  low_change = lag(low, 1) - low

  plus_dm = where(high_change > low_change and high_change > 0, high_change, 0)
  minus_dm = where(low_change > high_change and low_change > 0, low_change, 0)

  atr_14 = rolling_mean(true_range, 14)

  plus_di = 100 * rolling_mean(plus_dm, 14) / atr_14
  minus_di = 100 * rolling_mean(minus_dm, 14) / atr_14

  // Direction
  direction = plus_di - minus_di

  // Strength
  dx = 100 * abs(direction) / (plus_di + minus_di)
  adx = rolling_mean(dx, 14)

  // Strong trends only
  strong = adx > 25

  emit where(strong, zscore(direction), 0)
```

### Donchian Channel Breakout

```sig
signal donchian_breakout:
  high_20 = rolling_max(prices, 20)
  low_20 = rolling_min(prices, 20)

  // Position within channel
  position = (prices - low_20) / (high_20 - low_20)

  // Near high = bullish, near low = bearish
  emit zscore(position - 0.5)
```

## Time-Series Momentum

### Individual Asset Trends

```sig
signal time_series_momentum:
  // Each stock's own trend
  ret_60 = ret(prices, 60)
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Positive past return = long, negative = short
  // Scale by inverse volatility
  signal = ret_60 / vol_60

  emit signal
```

### Absolute vs Relative

```sig
signal absolute_momentum:
  // Long only if positive absolute return
  ret_60 = ret(prices, 60)
  positive_momentum = ret_60 > 0

  relative = zscore(ret_60)

  // Only long positive momentum stocks
  emit where(positive_momentum, relative, -abs(relative))
```

## Multi-Timeframe

```sig
signal multi_timeframe:
  // Short-term trend
  trend_20 = (prices - rolling_mean(prices, 20)) / rolling_mean(prices, 20)

  // Medium-term trend
  trend_50 = (prices - rolling_mean(prices, 50)) / rolling_mean(prices, 50)

  // Long-term trend
  trend_200 = (prices - rolling_mean(prices, 200)) / rolling_mean(prices, 200)

  // Combine
  combined = 0.2 * zscore(trend_20) + 0.3 * zscore(trend_50) + 0.5 * zscore(trend_200)

  emit combined
```

## Risk Management

### Trend with Volatility Scaling

```sig
signal vol_scaled_trend:
  trend = trend_signal

  // Scale by inverse vol
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  target_vol = 0.15

  scale = target_vol / vol

  emit trend * scale
```

### Trend Confirmation

```sig
signal confirmed_trend:
  ma_cross = rolling_mean(prices, 50) > rolling_mean(prices, 200)
  price_above_ma = prices > rolling_mean(prices, 50)

  // Both conditions = confirmed
  confirmed = ma_cross and price_above_ma

  raw_signal = (prices - rolling_mean(prices, 200)) / rolling_mean(prices, 200)

  emit where(confirmed, zscore(raw_signal), 0)
```

## Expected Results

```
Backtest Results: trend_following
=================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 118%
  Annual Return: 5.6%
  Annual Volatility: 9.8%
  Sharpe Ratio: 0.57

Characteristics:
  Avg Holding Period: 45 days
  Win Rate: 48%
  Profit Factor: 1.4

Performance by Regime:
  Bull Markets: +8.2%
  Bear Markets: +1.5%
  High Vol: +3.8%
  Low Vol: +6.2%
```

## See Also

- [Price Momentum](price-momentum.md)
- [Momentum + Quality](momentum-quality.md)
