# RSI Reversion Strategy

Mean reversion using the Relative Strength Index.

## Strategy Overview

Use RSI to identify overbought and oversold conditions. Buy when RSI is low (oversold), short when RSI is high (overbought).

## The RSI Indicator

```
RSI = 100 - (100 / (1 + RS))
RS = Average Gain / Average Loss
```

- RSI < 30: Oversold (buy signal)
- RSI > 70: Overbought (sell signal)

## The Signal

```sig
signal rsi:
  // Calculate RSI
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)

  avg_gain = ema(gains, 14)
  avg_loss = ema(losses, 14)

  rs = avg_gain / (avg_loss + 0.0001)
  rsi = 100 - (100 / (1 + rs))

  emit rsi

signal rsi_reversion:
  // Buy oversold, short overbought
  reversion = 50 - rsi  // Positive when oversold

  emit zscore(reversion)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// RSI calculation
signal rsi_14:
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)

  avg_gain = ema(gains, 14)
  avg_loss = ema(losses, 14)

  rs = avg_gain / (avg_loss + 0.0001)
  emit 100 - (100 / (1 + rs))

// RSI reversion signal
signal rsi_reversion:
  rsi = rsi_14

  // Distance from neutral (50)
  deviation = 50 - rsi

  // Sector neutralize
  emit neutralize(zscore(deviation), by=sectors)

portfolio rsi_strategy:
  weights = rank(rsi_reversion).long_short(
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

### Different Periods

```sig
// Shorter RSI (faster signals)
signal rsi_7:
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)
  avg_gain = ema(gains, 7)
  avg_loss = ema(losses, 7)
  rs = avg_gain / (avg_loss + 0.0001)
  emit 100 - (100 / (1 + rs))

// Longer RSI (slower signals)
signal rsi_21:
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)
  avg_gain = ema(gains, 21)
  avg_loss = ema(losses, 21)
  rs = avg_gain / (avg_loss + 0.0001)
  emit 100 - (100 / (1 + rs))
```

### Multi-Timeframe RSI

```sig
signal multi_rsi:
  rsi_7 = rsi_short
  rsi_14 = rsi_medium
  rsi_21 = rsi_long

  // Combine timeframes
  avg_rsi = (rsi_7 + rsi_14 + rsi_21) / 3

  emit zscore(50 - avg_rsi)
```

### Extreme RSI Only

```sig
signal extreme_rsi:
  rsi = rsi_14

  // Only trade at extremes
  extreme_low = rsi < 30
  extreme_high = rsi > 70

  signal = where(extreme_low, 50 - rsi,
           where(extreme_high, 50 - rsi, 0))

  emit zscore(signal)
```

## RSI Divergence

```sig
signal rsi_divergence:
  rsi = rsi_14

  // Price making new lows but RSI not
  price_low = prices == rolling_min(prices, 20)
  rsi_not_low = rsi > rolling_min(rsi, 20)
  bullish_div = price_low and rsi_not_low

  // Price making new highs but RSI not
  price_high = prices == rolling_max(prices, 20)
  rsi_not_high = rsi < rolling_max(rsi, 20)
  bearish_div = price_high and rsi_not_high

  signal = where(bullish_div, 1,
           where(bearish_div, -1, 0))

  emit zscore(signal)
```

## With Trend Filter

```sig
signal filtered_rsi:
  rsi = rsi_14
  reversion = 50 - rsi

  // Trend filter
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  uptrend = ma_50 > ma_200

  // In uptrend, only take long signals
  // In downtrend, only take short signals
  signal = where(uptrend and reversion > 0, reversion,
           where(not(uptrend) and reversion < 0, reversion,
           reversion * 0.3))

  emit zscore(signal)
```

## Stochastic RSI

```sig
signal stoch_rsi:
  rsi = rsi_14

  // Stochastic of RSI
  rsi_min = rolling_min(rsi, 14)
  rsi_max = rolling_max(rsi, 14)

  stoch = (rsi - rsi_min) / (rsi_max - rsi_min + 0.0001)

  // Buy when stoch RSI low, sell when high
  emit zscore(0.5 - stoch)
```

## Expected Results

```
Backtest Results: rsi_reversion
===============================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 48%
  Annual Return: 4.2%
  Annual Volatility: 7.8%
  Sharpe Ratio: 0.54

Turnover:
  Annual Turnover: 480%
  Avg Holding Period: 12 days

Signal Quality:
  Avg RSI (longs): 38.2
  Avg RSI (shorts): 62.5
```

## Risk Considerations

### RSI Can Stay Extreme

In strong trends, RSI can stay overbought/oversold:

```sig
// Add time limit
signal time_limited:
  rsi = rsi_14
  days_extreme = ts_sum(rsi < 30 or rsi > 70, 20)

  // If extreme too long, reduce confidence
  extended = days_extreme > 10

  base_signal = 50 - rsi
  emit where(extended, base_signal * 0.5, base_signal)
```

## See Also

- [Statistical Reversion](statistical-reversion.md)
- [Bollinger Bands](bollinger-bands.md)
