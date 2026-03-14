# Volatility Breakout Strategy

Trade volatility expansion and contraction.

## Strategy Overview

Volatility clusters: high vol leads to high vol, low vol to low vol. Trade the breakouts from low volatility periods.

## The Signal

```sig
signal vol_breakout:
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Vol expansion ratio
  expansion = vol_20 / vol_60

  // Direction from price
  trend = ret(prices, 20)

  // Expanding vol + positive trend = long
  signal = expansion * sign(trend)

  emit zscore(signal)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal vol_breakout:
  // Short-term vs long-term vol
  vol_short = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_long = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Volatility regime
  vol_expanding = vol_short > vol_long * 1.2
  vol_contracting = vol_short < vol_long * 0.8

  // Price direction
  trend = ret(prices, 20)
  positive = trend > 0

  // Breakout signals
  // Expanding + up = buy, Expanding + down = sell
  signal = where(vol_expanding and positive, 1,
           where(vol_expanding and not(positive), -1, 0))

  emit neutralize(zscore(signal), by=sectors)

portfolio vol_breakout:
  weights = rank(vol_breakout).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## Volatility Squeeze

```sig
signal vol_squeeze:
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)

  // Historical vol range
  vol_min = rolling_min(vol_20, 120)
  vol_max = rolling_max(vol_20, 120)

  // Squeeze = vol near lows
  squeeze = (vol_20 - vol_min) / (vol_max - vol_min) < 0.2

  // After squeeze, expect expansion
  // Direction from momentum
  momentum = ret(prices, 20)

  // Trade direction when squeeze releases
  vol_rising = vol_20 > lag(vol_20, 5)
  squeeze_release = lag(squeeze, 1) and vol_rising

  signal = where(squeeze_release, sign(momentum), 0)

  emit zscore(signal)
```

## Expected Results

```
Backtest Results: vol_breakout
==============================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 58%
  Annual Return: 5.2%
  Annual Volatility: 11.5%
  Sharpe Ratio: 0.45

Characteristics:
  Avg Holding Period: 15 days
  Win Rate: 46%
  Profit Factor: 1.25
```

## See Also

- [Trend Following](../momentum/trend-following.md)
- [Low Volatility](low-volatility.md)
