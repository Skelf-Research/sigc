# Moving Average Strategies

Trading systems based on moving average crossovers.

## Strategy Overview

Use moving average crossovers to identify trend changes and generate signals.

## Golden/Death Cross

```sig
signal golden_cross:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)

  // Golden cross: 50-day crosses above 200-day
  golden = ma_50 > ma_200

  emit where(golden, 1, -1)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal ma_crossover:
  ma_20 = rolling_mean(prices, 20)
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)

  // Price above all MAs = strong uptrend
  above_all = (prices > ma_20) and (ma_20 > ma_50) and (ma_50 > ma_200)
  below_all = (prices < ma_20) and (ma_20 < ma_50) and (ma_50 < ma_200)

  // Trend strength
  trend = (prices - ma_200) / ma_200

  signal = where(above_all, trend,
           where(below_all, trend, trend * 0.5))

  emit neutralize(zscore(signal), by=sectors)

portfolio ma_strategy:
  weights = rank(ma_crossover).long_short(
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

## Variations

### Exponential MAs

```sig
signal ema_crossover:
  ema_12 = ema(prices, 12)
  ema_26 = ema(prices, 26)

  emit zscore(ema_12 - ema_26)
```

### Weighted MA

```sig
signal wma_trend:
  // Linear-weighted MA gives more weight to recent prices
  wma_20 = wma(prices, 20)
  sma_50 = rolling_mean(prices, 50)

  emit zscore((wma_20 - sma_50) / sma_50)
```

## Expected Results

```
Annual Return: 4.5%
Sharpe: 0.48
Max Drawdown: -18%
```

## See Also

- [Trend Following](../momentum/trend-following.md)
- [MACD Strategy](macd-strategy.md)
