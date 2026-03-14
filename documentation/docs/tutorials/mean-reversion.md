# Tutorial: Mean Reversion Strategy

Build a mean reversion strategy that profits from price returning to equilibrium.

## Overview

Mean reversion assumes prices deviate from their "fair value" and eventually return:

- Overbought stocks tend to fall
- Oversold stocks tend to rise
- Works best in range-bound markets

## The Strategy

We'll build a strategy that:

1. Identifies oversold/overbought conditions
2. Buys oversold, shorts overbought
3. Exits when prices normalize

## Step 1: Basic Mean Reversion

### Z-Score Approach

```sig
data:
  source = "prices.parquet"
  format = parquet

signal mean_reversion:
  // Calculate z-score of price vs moving average
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  zscore = (prices - ma) / std

  // Negative zscore = oversold = buy
  emit -zscore

portfolio basic:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 2: RSI-Based Reversion

### Relative Strength Index

```sig
signal rsi:
  // Calculate RSI
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)

  avg_gain = rolling_mean(gains, 14)
  avg_loss = rolling_mean(losses, 14)

  rs = avg_gain / (avg_loss + 0.0001)  // Avoid division by zero
  rsi = 100 - (100 / (1 + rs))

  emit rsi

signal rsi_reversion:
  // Buy oversold (low RSI), short overbought (high RSI)
  reversion = 50 - rsi
  emit zscore(reversion)

portfolio rsi_based:
  weights = rank(rsi_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 3: Bollinger Band Reversion

### Using Bollinger Bands

```sig
signal bollinger_reversion:
  // Bollinger Bands
  ma_20 = rolling_mean(prices, 20)
  std_20 = rolling_std(prices, 20)

  upper_band = ma_20 + 2 * std_20
  lower_band = ma_20 - 2 * std_20

  // Position within bands (0 = lower, 1 = upper)
  band_position = (prices - lower_band) / (upper_band - lower_band)

  // Revert from extremes
  reversion = 0.5 - band_position

  emit zscore(reversion)

portfolio bollinger:
  weights = rank(bollinger_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 4: Multi-Horizon Reversion

### Combining Multiple Timeframes

```sig
signal multi_horizon:
  // Short-term (5-day) reversion
  ma_5 = rolling_mean(prices, 5)
  short_dev = (prices - ma_5) / ma_5

  // Medium-term (20-day) reversion
  ma_20 = rolling_mean(prices, 20)
  medium_dev = (prices - ma_20) / ma_20

  // Combine signals
  combined = -0.6 * zscore(short_dev) - 0.4 * zscore(medium_dev)

  emit combined

portfolio multi_horizon:
  weights = rank(multi_horizon).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 5: Volume-Adjusted Reversion

### Stronger Signals with Volume Confirmation

```sig
signal volume_adjusted:
  // Price deviation
  ma_20 = rolling_mean(prices, 20)
  price_zscore = zscore((prices - ma_20) / ma_20)

  // Volume spike (unusual volume = stronger signal)
  vol_ma = rolling_mean(volume, 20)
  vol_ratio = volume / vol_ma
  vol_zscore = zscore(vol_ratio)

  // Combine: strong reversion when volume confirms
  // High volume + oversold = strong buy signal
  signal = -price_zscore * (1 + 0.3 * vol_zscore)

  emit signal

portfolio vol_adjusted:
  weights = rank(volume_adjusted).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 6: Regime-Aware Reversion

### Only Trade in Range-Bound Markets

```sig
signal trend_indicator:
  // Detect trending vs range-bound
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  trend_strength = abs(ma_50 - ma_200) / ma_200

  // High trend strength = trending market
  emit trend_strength

signal adaptive_reversion:
  // Basic reversion signal
  ma_20 = rolling_mean(prices, 20)
  reversion = -zscore((prices - ma_20) / ma_20)

  // Reduce signal in trending markets
  trend = trend_indicator
  trend_threshold = quantile(trend, 0.7)
  is_trending = trend > trend_threshold

  // Scale down in trends
  adjusted = where(is_trending, reversion * 0.3, reversion)

  emit adjusted

portfolio regime_aware:
  weights = rank(adaptive_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Step 7: Complete Strategy

### Full Implementation with Risk Controls

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Core reversion signal
signal price_reversion:
  ma_20 = rolling_mean(prices, 20)
  std_20 = rolling_std(prices, 20)
  zscore = (prices - ma_20) / std_20
  emit -zscore

// RSI component
signal rsi_signal:
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)
  avg_gain = ema(gains, 14)
  avg_loss = ema(losses, 14)
  rs = avg_gain / (avg_loss + 0.0001)
  rsi = 100 - (100 / (1 + rs))
  emit zscore(50 - rsi)

// Volume confirmation
signal volume_signal:
  vol_ma = rolling_mean(volume, 20)
  vol_ratio = volume / vol_ma
  emit zscore(vol_ratio)

// Trend filter
signal trend_filter:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  trend_strength = abs(ma_50 - ma_200) / ma_200
  // 1 = range-bound (good), 0 = trending (bad)
  emit 1 - zscore(trend_strength)

// Combined signal
signal mean_reversion:
  // Combine reversion components
  base = 0.5 * price_reversion + 0.3 * rsi_signal

  // Volume boost
  vol_boost = 1 + 0.2 * clip(volume_signal, -1, 1)

  // Trend scaling
  trend_scale = clip(0.5 + 0.5 * trend_filter, 0.3, 1.0)

  // Final signal
  signal = base * vol_boost * trend_scale

  // Sector neutralize
  emit neutralize(signal, by=sectors)

portfolio mean_reversion:
  weights = rank(mean_reversion).long_short(
    top = 0.15,
    bottom = 0.15,
    cap = 0.03
  )

  constraints:
    max_sector = 0.20
    gross_exposure = 2.0
    net_exposure = 0.0

  costs = tc.bps(10)

  backtest rebal=5 from 2018-01-01 to 2024-12-31
```

## Step 8: Run and Analyze

### Execute Backtest

```bash
sigc run mean_reversion.sig
```

### Expected Output

```
Backtest Results: mean_reversion
================================
Period: 2018-01-01 to 2024-12-31

Returns:
  Total Return: 42.3%
  Annual Return: 5.2%
  Annual Volatility: 8.1%
  Sharpe Ratio: 0.64

Risk:
  Max Drawdown: -12.4%
  Avg Drawdown: -3.2%

Turnover:
  Annual Turnover: 620%
  Avg Holding Period: 12 days
```

## Step 9: Parameter Optimization

### Find Optimal Parameters

```sig
params:
  ma_window: range(10, 40, 5)
  rebal_days: [3, 5, 7, 10]

signal mean_reversion:
  ma = rolling_mean(prices, ma_window)
  std = rolling_std(prices, ma_window)
  emit -zscore((prices - ma) / std)

portfolio optimized:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(10)
  backtest rebal=rebal_days from 2020-01-01 to 2024-12-31
```

```bash
sigc run mean_reversion.sig --optimize --metric sharpe
```

## Key Insights

### When Mean Reversion Works

1. **Range-bound markets** - Clear support/resistance
2. **High volatility** - More mispricings to exploit
3. **Short horizons** - Quick reversals

### When It Fails

1. **Strong trends** - Prices keep going
2. **Regime changes** - New equilibrium
3. **Momentum markets** - Winners keep winning

### Risk Management

1. **Stop losses** - Limit damage when trends persist
2. **Sector neutralization** - Avoid sector bets
3. **Short holding periods** - Exit before trends develop

## Common Enhancements

### 1. Pairs Trading

```sig
signal pairs:
  // Trade spread between related stocks
  spread = prices[A] - beta * prices[B]
  spread_zscore = zscore(spread, lookback=60)
  emit -spread_zscore
```

### 2. Cointegration

```sig
signal coint_reversion:
  // Use cointegrated pairs
  residual = prices - cointegration_residual(prices, market)
  emit -zscore(residual)
```

### 3. Fundamental Anchoring

```sig
signal fundamental_reversion:
  // Revert to fundamental value
  fair_value = book_value * avg_pb_ratio
  deviation = (prices - fair_value) / fair_value
  emit -zscore(deviation)
```

## Next Steps

- [Multi-Factor Tutorial](multi-factor.md) - Combine strategies
- [Volatility Strategy](volatility-strategy.md) - Trade volatility
- [Walk-Forward Optimization](walk-forward-optimization.md) - Robust testing
