# Statistical Reversion Strategy

Mean reversion using z-scores and statistical measures.

## Strategy Overview

Buy stocks that are statistically oversold, short stocks that are overbought, based on deviation from historical norms.

## The Signal

```sig
signal statistical_reversion:
  // Z-score of price vs moving average
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  zscore = (prices - ma) / std

  // Negative zscore = oversold = buy
  emit -zscore
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Statistical reversion signal
signal stat_reversion:
  // Multiple timeframe z-scores
  zscore_5 = (prices - rolling_mean(prices, 5)) / rolling_std(prices, 5)
  zscore_20 = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)

  // Combine short and medium term
  combined = -0.4 * zscore_5 - 0.6 * zscore_20

  // Sector neutralize
  emit neutralize(combined, by=sectors)

portfolio stat_reversion:
  weights = rank(stat_reversion).long_short(
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

### Percentile-Based

```sig
signal percentile_reversion:
  // Where is price in its historical range?
  percentile = ts_rank(prices, 252) / 252

  // Below 20% = oversold, above 80% = overbought
  signal = 0.5 - percentile

  emit zscore(signal)
```

### Return-Based

```sig
signal return_reversion:
  // Revert recent returns
  ret_5d = ret(prices, 5)
  ret_20d = ret(prices, 20)

  // Negative returns = buy signal
  emit -zscore(0.5 * ret_5d + 0.5 * ret_20d)
```

### Residual Reversion

```sig
signal residual_reversion:
  // Market-adjusted deviation
  ret_5d = ret(prices, 5)
  market_ret = ret(market, 5)

  // How much did stock over/underperform market?
  residual = ret_5d - market_ret

  // Revert the residual
  emit -zscore(residual)
```

## Multi-Horizon Reversion

```sig
signal multi_horizon:
  // Short-term (5-day)
  z5 = (prices - rolling_mean(prices, 5)) / rolling_std(prices, 5)

  // Medium-term (20-day)
  z20 = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)

  // Long-term (60-day)
  z60 = (prices - rolling_mean(prices, 60)) / rolling_std(prices, 60)

  // Weight by decay speed
  combined = -0.5 * z5 - 0.35 * z20 - 0.15 * z60

  emit neutralize(combined, by=sectors)
```

## With Volume Confirmation

```sig
signal volume_confirmed:
  // Price deviation
  price_z = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)

  // Volume spike = stronger signal
  vol_ratio = volume / rolling_mean(volume, 20)
  vol_spike = vol_ratio > 1.5

  // Boost signal when volume confirms
  signal = -price_z * where(vol_spike, 1.3, 1.0)

  emit zscore(signal)
```

## Regime Filter

```sig
signal filtered_reversion:
  // Base signal
  reversion = -zscore((prices - rolling_mean(prices, 20)) / rolling_std(prices, 20))

  // Trend filter - don't revert in strong trends
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  trend_strength = abs(ma_50 - ma_200) / ma_200

  // Low trend = good for reversion
  low_trend = trend_strength < 0.05

  emit where(low_trend, reversion, reversion * 0.5)
```

## Expected Results

```
Backtest Results: stat_reversion
================================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 62%
  Annual Return: 5.1%
  Annual Volatility: 8.3%
  Sharpe Ratio: 0.61

Turnover:
  Annual Turnover: 520%
  Avg Holding Period: 14 days

Performance by Regime:
  Low Trend: Sharpe 0.85
  High Trend: Sharpe 0.25
```

## Risk Considerations

### Trend Risk

Mean reversion fails in trends:

```sig
// Stop loss mechanism
signal with_stop:
  reversion = stat_reversion

  // If position moving against us significantly, reduce
  position_pnl = position * ret(prices, 5)
  stop = position_pnl < -0.10  // 10% loss on position

  emit where(stop, 0, reversion)
```

### Correlation Risk

All reversion trades can fail together:

```sig
// Add diversification
constraints:
  max_position = 0.02  // Smaller positions
  min_positions = 50   // More positions
```

## See Also

- [RSI Reversion](rsi-reversion.md)
- [Bollinger Bands](bollinger-bands.md)
- [Short-Term Reversal](short-term-reversal.md)
