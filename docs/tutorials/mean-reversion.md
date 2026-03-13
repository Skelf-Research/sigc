# Tutorial: Mean Reversion Strategy

Build a contrarian strategy that bets on price normalization.

## Concept

Mean reversion assumes prices deviate from fair value temporarily and will revert. We:
1. Measure deviation from moving average
2. Short overextended assets
3. Long undervalued assets

## Step 1: Basic Z-Score

Create `meanrev_v1.sig`:

```
data:
  prices: load csv from "docs/examples/data/sample_prices.csv"

params:
  lookback = 20

signal mean_reversion:
  # Moving average
  ma = rolling_mean(prices, lookback)

  # Standard deviation
  std = rolling_std(prices, lookback)

  # Z-score: how many stds from mean
  z = (prices - ma) / std

  # Negative because we SHORT high z-scores
  emit -z

portfolio main:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-05-31
```

## Step 2: Cross-Sectional Normalization

```
signal mean_reversion:
  ma = rolling_mean(prices, lookback)
  std = rolling_std(prices, lookback)
  z = (prices - ma) / std

  # Normalize across assets
  normalized = zscore(-z)

  emit normalized
```

## Step 3: RSI-Based Version

Alternative using RSI (overbought/oversold):

```
params:
  rsi_period = 14
  overbought = 70
  oversold = 30

signal rsi_reversion:
  r = rsi(prices, rsi_period)

  # Convert to signal: short overbought, long oversold
  # RSI 70+ -> negative, RSI 30- -> positive
  signal = 50 - r

  emit zscore(signal)
```

## Step 4: Production Version

```
data:
  prices: load csv from "docs/examples/data/sample_prices.csv"

params:
  fast_lookback = 5
  slow_lookback = 20
  vol_lookback = 60
  winsor_pct = 0.01

signal mean_reversion:
  # Short-term vs long-term mean
  fast_ma = rolling_mean(prices, fast_lookback)
  slow_ma = rolling_mean(prices, slow_lookback)

  # Deviation from trend
  deviation = (prices - slow_ma) / slow_ma

  # Volatility adjustment
  vol = rolling_std(ret(prices, 1), vol_lookback)
  adjusted = deviation / vol

  # Clean and normalize
  cleaned = winsor(-adjusted, winsor_pct)
  normalized = zscore(cleaned)

  emit normalized

portfolio main:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-05-31
```

## Key Differences from Momentum

| Aspect | Momentum | Mean Reversion |
|--------|----------|----------------|
| Bet | Winners keep winning | Losers will recover |
| Signal | High return → long | High return → short |
| Timeframe | 1-12 months | Days to weeks |
| Risk | Trend reversals | Trends continuing |

## Combining with Momentum

They can complement each other:

```
signal combo:
  # Long-term momentum
  mom = zscore(ret(prices, 60))

  # Short-term reversion
  rev = -zscore(ret(prices, 5))

  # Combine: momentum for direction, reversion for timing
  combined = 0.7 * mom + 0.3 * rev

  emit combined
```
