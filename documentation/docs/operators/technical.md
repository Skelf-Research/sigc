# Technical Indicators

Classic technical analysis indicators.

## RSI

### `rsi(x, n)`

Relative Strength Index.

```sig
rsi(x: Panel, n: Scalar) -> Panel
```

Returns value between 0 and 100.

- RSI > 70: Overbought
- RSI < 30: Oversold

```sig
signal example:
  rsi_14 = rsi(prices, 14)
  emit rsi_14
```

### RSI Signal Example

```sig
signal rsi_signal:
  rsi_val = rsi(prices, 14)

  // Contrarian: buy oversold, sell overbought
  oversold = rsi_val < 30
  overbought = rsi_val > 70

  signal = where(oversold, 1, where(overbought, -1, 0))
  emit signal
```

### RSI as Continuous Signal

```sig
signal rsi_continuous:
  rsi_val = rsi(prices, 14)
  // Center around 50, normalize
  centered = rsi_val - 50
  // Scale to approximately [-2, 2]
  scaled = centered / 25
  // Contrarian (negative)
  emit -zscore(scaled)
```

## MACD

### `macd(x, fast, slow, signal)`

Moving Average Convergence/Divergence.

```sig
macd(x: Panel, fast: Scalar, slow: Scalar, signal: Scalar) -> Panel
```

Returns MACD histogram (MACD line - signal line).

- Standard parameters: fast=12, slow=26, signal=9

```sig
signal example:
  macd_hist = macd(prices, 12, 26, 9)
  emit macd_hist
```

### MACD Signal Example

```sig
signal macd_signal:
  hist = macd(prices, 12, 26, 9)

  // Positive histogram = bullish
  bullish = hist > 0

  signal = where(bullish, 1, -1)
  emit signal
```

### MACD Crossover

```sig
signal macd_crossover:
  hist_today = macd(prices, 12, 26, 9)
  hist_yesterday = lag(hist_today, 1)

  // Crossover signals
  bullish_cross = hist_today > 0 and hist_yesterday <= 0
  bearish_cross = hist_today < 0 and hist_yesterday >= 0

  signal = where(bullish_cross, 1, where(bearish_cross, -1, 0))
  emit signal
```

## ATR

### `atr(high, low, close, n)`

Average True Range.

```sig
atr(high: Panel, low: Panel, close: Panel, n: Scalar) -> Panel
```

Measures volatility based on high, low, close prices.

```sig
signal example:
  atr_14 = atr(high, low, close, 14)
  emit atr_14
```

### ATR-Based Position Sizing

```sig
signal atr_signal:
  atr_val = atr(high, low, close, 14)

  // Inverse ATR for position sizing (less position for volatile stocks)
  inv_atr = 1 / atr_val

  // Normalize
  emit zscore(inv_atr)
```

### ATR Channel

```sig
signal atr_channel:
  atr_val = atr(high, low, close, 20)
  ma = rolling_mean(close, 20)

  // Position in ATR channel
  distance = (close - ma) / atr_val
  emit distance
```

## VWAP

### `vwap(price, volume)`

Volume-Weighted Average Price.

```sig
vwap(price: Panel, volume: Panel) -> Panel
```

```sig
signal example:
  vwap_price = vwap(prices, volume)
  emit vwap_price
```

### VWAP Deviation Signal

```sig
signal vwap_signal:
  vwap_price = vwap(prices, volume)

  // Distance from VWAP
  deviation = (prices - vwap_price) / vwap_price

  // Mean reversion: fade moves away from VWAP
  emit -zscore(deviation)
```

## Common Technical Patterns

### Trend Following

```sig
signal trend:
  // EMA crossover
  fast = ema(prices, 10)
  slow = ema(prices, 50)
  trend = (fast - slow) / slow
  emit zscore(trend)
```

### Mean Reversion

```sig
signal reversion:
  // Bollinger Band position
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  z = (prices - ma) / std
  emit -zscore(z)
```

### Momentum + RSI

```sig
signal mom_rsi:
  // Combine momentum with RSI
  mom = zscore(ret(prices, 60))
  rsi_val = rsi(prices, 14)
  rsi_sig = -zscore((rsi_val - 50) / 25)

  // Blend
  combined = 0.7 * mom + 0.3 * rsi_sig
  emit combined
```

### Volatility Breakout

```sig
signal vol_breakout:
  atr_val = atr(high, low, close, 14)
  prev_close = lag(close, 1)

  // Breakout if moved > 2 ATR
  up_break = close > prev_close + 2 * atr_val
  down_break = close < prev_close - 2 * atr_val

  signal = where(up_break, 1, where(down_break, -1, 0))
  emit signal
```

### MACD + Trend Filter

```sig
signal filtered_macd:
  // MACD signal
  macd_hist = macd(prices, 12, 26, 9)

  // Trend filter: only take MACD signals in trend direction
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  uptrend = ma_50 > ma_200

  // Only long signals in uptrend
  signal = where(uptrend, where(macd_hist > 0, 1, 0),
                          where(macd_hist < 0, -1, 0))
  emit signal
```

## Building Custom Indicators

### Williams %R

```sig
signal williams_r:
  high_n = rolling_max(high, 14)
  low_n = rolling_min(low, 14)
  wr = (high_n - close) / (high_n - low_n) * -100
  // -100 to 0, oversold < -80, overbought > -20
  emit zscore(-wr)
```

### Stochastic Oscillator

```sig
signal stochastic:
  low_14 = rolling_min(low, 14)
  high_14 = rolling_max(high, 14)
  k = (close - low_14) / (high_14 - low_14) * 100
  d = rolling_mean(k, 3)
  emit zscore(k - 50)
```

### On-Balance Volume (OBV)

```sig
signal obv:
  price_change = delta(close, 1)
  signed_vol = where(price_change > 0, volume, where(price_change < 0, -volume, 0))
  obv = cumsum(signed_vol)
  // OBV trend
  obv_trend = obv - rolling_mean(obv, 20)
  emit zscore(obv_trend)
```

### Money Flow Index

```sig
signal mfi:
  typical_price = (high + low + close) / 3
  money_flow = typical_price * volume

  pos_flow = where(delta(typical_price, 1) > 0, money_flow, 0)
  neg_flow = where(delta(typical_price, 1) < 0, money_flow, 0)

  pos_sum = rolling_sum(pos_flow, 14)
  neg_sum = rolling_sum(neg_flow, 14)

  mfi = 100 - (100 / (1 + pos_sum / neg_sum))
  emit zscore(mfi - 50)
```

## Best Practices

### 1. Normalize Indicators

```sig
// Raw RSI: 0-100
raw_rsi = rsi(prices, 14)

// Normalized: approximately [-2, 2]
normalized = (raw_rsi - 50) / 25
emit zscore(normalized)
```

### 2. Combine with Fundamentals

```sig
signal combined:
  // Technical
  rsi_sig = -zscore((rsi(prices, 14) - 50) / 25)

  // Fundamental
  value_sig = zscore(earnings_yield)

  // Blend
  emit 0.5 * rsi_sig + 0.5 * value_sig
```

### 3. Use Confirmation

```sig
signal confirmed:
  macd_sig = macd(prices, 12, 26, 9) > 0
  rsi_sig = rsi(prices, 14) < 70

  // Only long when both agree
  signal = where(macd_sig and rsi_sig, 1, 0)
  emit signal
```

## Next Steps

- [Portfolio](portfolio.md) - Weight construction
- [Tutorials](../tutorials/index.md) - Complete strategies
- [Time-Series](time-series.md) - Building blocks
