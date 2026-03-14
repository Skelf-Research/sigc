# Volume Pattern Strategies

Trade using volume-based signals.

## Strategy Overview

Volume confirms price moves. High volume + price move = stronger signal.

## Volume Surge

```sig
signal volume_surge:
  vol_ratio = volume / rolling_mean(volume, 20)

  // High volume days
  surge = vol_ratio > 2.0

  // Direction from price
  direction = sign(ret(prices, 1))

  // Surge in direction of price
  emit where(surge, direction * vol_ratio, 0)
```

## Complete Strategy

```sig
data:
  source = "prices_volume.parquet"
  format = parquet

signal volume_momentum:
  // Price momentum
  price_mom = ret(prices, 20)

  // Volume trend
  vol_ma = rolling_mean(volume, 20)
  vol_change = volume / vol_ma

  // Rising volume + positive price = bullish
  signal = price_mom * vol_change

  emit neutralize(zscore(signal), by=sectors)

portfolio volume_strategy:
  weights = rank(volume_momentum).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## On-Balance Volume (OBV)

```sig
signal obv_momentum:
  // OBV: cumulative volume based on price direction
  direction = sign(diff(prices, 1))
  obv = cumsum(direction * volume)

  // OBV momentum
  obv_ma = rolling_mean(obv, 20)
  obv_trend = (obv - obv_ma) / obv_ma

  emit zscore(obv_trend)
```

## Volume-Price Divergence

```sig
signal vol_price_divergence:
  // Price making new highs but volume declining = bearish
  price_high = prices == rolling_max(prices, 20)
  vol_low = volume < rolling_mean(volume, 20)

  bearish_div = price_high and vol_low

  price_low = prices == rolling_min(prices, 20)
  vol_high = volume > rolling_mean(volume, 20)

  bullish_div = price_low and vol_high

  emit where(bullish_div, 1, where(bearish_div, -1, 0))
```

## Expected Results

```
Annual Return: 4.2%
Sharpe: 0.42
```

## See Also

- [Breakouts](breakouts.md)
- [Volatility Breakout](../volatility/volatility-breakout.md)
