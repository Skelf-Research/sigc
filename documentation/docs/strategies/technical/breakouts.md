# Breakout Strategies

Trade price breakouts from ranges and patterns.

## Strategy Overview

Buy when prices break above resistance, short when they break below support.

## Donchian Channel Breakout

```sig
signal donchian:
  high_20 = rolling_max(prices, 20)
  low_20 = rolling_min(prices, 20)

  // Breakout above = bullish
  breakout_up = prices >= high_20
  breakout_down = prices <= low_20

  signal = where(breakout_up, 1, where(breakout_down, -1, 0))

  emit zscore(signal)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal breakout:
  // N-day high/low channels
  high_20 = rolling_max(prices, 20)
  low_20 = rolling_min(prices, 20)

  // Position in range
  range_position = (prices - low_20) / (high_20 - low_20 + 0.0001)

  // Near extremes
  near_high = range_position > 0.9
  near_low = range_position < 0.1

  // Recent volume surge (confirms breakout)
  vol_ratio = volume / rolling_mean(volume, 20)
  high_volume = vol_ratio > 1.5

  // Breakout with volume confirmation
  signal = where(near_high and high_volume, 1,
           where(near_low and high_volume, -1, 0))

  emit neutralize(zscore(signal), by=sectors)

portfolio breakout:
  weights = rank(breakout).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## 52-Week High/Low

```sig
signal yearly_breakout:
  high_252 = rolling_max(prices, 252)
  low_252 = rolling_min(prices, 252)

  near_high = prices > high_252 * 0.95
  near_low = prices < low_252 * 1.05

  emit where(near_high, 1, where(near_low, -1, 0))
```

## Expected Results

```
Annual Return: 5.2%
Sharpe: 0.52
Win Rate: 45%
Profit Factor: 1.35
```

## See Also

- [Trend Following](../momentum/trend-following.md)
- [Volume Patterns](volume-patterns.md)
