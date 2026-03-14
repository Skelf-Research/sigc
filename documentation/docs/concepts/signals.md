# What Are Signals?

Signals are the foundation of quantitative trading strategies in sigc. This page explains what signals are, how they work, and how to build them.

## Definition

A **signal** is a numerical score computed for each asset at each point in time. Higher scores indicate more attractive assets; lower scores indicate less attractive ones.

```sig
signal momentum:
  returns = ret(prices, 20)
  score = zscore(returns)
  emit score
```

## Signal Characteristics

### Cross-Sectional

Signals rank assets relative to each other at each point in time:

```
Date: 2024-01-15
  AAPL:  +1.2  (high score - attractive)
  MSFT:  +0.5
  GOOGL: -0.3
  AMZN:  -1.4  (low score - unattractive)
```

### Time-Varying

Signal values change as new data arrives:

```
         AAPL    MSFT    GOOGL   AMZN
2024-01: +1.2    +0.5    -0.3    -1.4
2024-02: +0.8    +1.1    +0.2    -2.1
2024-03: -0.5    +0.9    +1.3    -1.7
```

### Normalized

Well-designed signals are normalized (mean ≈ 0, std ≈ 1):

```sig
signal good:
  raw = ret(prices, 20)
  emit zscore(raw)  // Normalized: mean=0, std=1
```

## Signal Types

### Factor-Based Signals

Based on academic research and economic intuition:

| Factor | Description | Example |
|--------|-------------|---------|
| Momentum | Past returns predict future | 12-1 month returns |
| Value | Cheap assets outperform | Book-to-market ratio |
| Quality | High-quality outperforms | ROE, earnings stability |
| Size | Small-cap premium | Market cap (inverted) |
| Volatility | Low-vol anomaly | Historical volatility |

```sig
signal momentum:
  r12 = ret(prices, 252)  // 12-month return
  r1 = ret(prices, 21)    // 1-month return
  mom = r12 - r1          // Skip recent month
  emit zscore(mom)

signal value:
  emit zscore(book_to_market)

signal quality:
  emit zscore(roe)
```

### Technical Signals

Based on price patterns and indicators:

```sig
signal rsi_signal:
  rsi_val = rsi(prices, 14)
  centered = rsi_val - 50
  emit -centered / 25  // Contrarian: buy oversold

signal macd_signal:
  emit macd(prices, 12, 26, 9)

signal trend:
  fast = ema(prices, 10)
  slow = ema(prices, 50)
  emit zscore(fast - slow)
```

### Statistical Signals

Based on statistical relationships:

```sig
signal mean_reversion:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  z = (prices - ma) / std
  emit -zscore(z)  // Fade extremes

signal residual_momentum:
  // Market-neutral momentum
  market_ret = rolling_mean(ret(prices, 1), 20)
  stock_ret = ret(prices, 20)
  residual = stock_ret - market_ret
  emit zscore(residual)
```

## Building Effective Signals

### Step 1: Start with Raw Data

```sig
signal step1:
  raw = ret(prices, 20)
  emit raw  // Not normalized - bad!
```

### Step 2: Normalize Cross-Sectionally

```sig
signal step2:
  raw = ret(prices, 20)
  normalized = zscore(raw)
  emit normalized  // Better - mean=0, std=1
```

### Step 3: Handle Outliers

```sig
signal step3:
  raw = ret(prices, 20)
  normalized = zscore(raw)
  cleaned = winsor(normalized, p=0.01)
  emit cleaned  // Best - outliers clipped
```

### Step 4: Adjust for Volatility

```sig
signal step4:
  raw = ret(prices, 20)
  vol = rolling_std(ret(prices, 1), 60)
  vol_adj = raw / vol
  normalized = zscore(vol_adj)
  cleaned = winsor(normalized, p=0.01)
  emit cleaned  // High-vol stocks don't dominate
```

## Signal Properties

### Information Coefficient (IC)

The correlation between signal and future returns:

- **IC > 0.05**: Good predictive signal
- **IC ≈ 0**: No predictive power
- **IC < 0**: Negative (contrarian) signal

### Turnover

How much the signal changes over time:

- **Low turnover**: Stable signal, lower trading costs
- **High turnover**: Responsive signal, higher trading costs

### Decay

How quickly the signal's predictive power fades:

```sig
// Short-horizon signal (decays quickly)
signal short_term:
  emit zscore(ret(prices, 5))

// Long-horizon signal (slower decay)
signal long_term:
  emit zscore(ret(prices, 252))
```

## Combining Signals

### Linear Combination

```sig
signal combo:
  mom = zscore(ret(prices, 60))
  rev = -zscore(ret(prices, 5))
  vol = -zscore(rolling_std(ret(prices, 1), 20))

  // Weight by confidence/research
  combined = 0.5 * mom + 0.3 * rev + 0.2 * vol
  emit combined
```

### Dynamic Weighting

```sig
signal adaptive:
  mom = zscore(ret(prices, 60))
  rev = -zscore(ret(prices, 5))

  // More momentum when trending
  trend_strength = abs(rolling_mean(ret(prices, 1), 20))
  mom_weight = where(trend_strength > 0.01, 0.7, 0.3)

  emit mom_weight * mom + (1 - mom_weight) * rev
```

## Common Patterns

### Momentum with Skip

Skip recent returns to avoid short-term reversal:

```sig
signal momentum_skip:
  total = ret(prices, 252)
  recent = ret(prices, 21)
  emit zscore(total - recent)
```

### Sector Neutralization

Remove sector bias:

```sig
signal sector_neutral:
  raw = zscore(ret(prices, 20))
  neutral = neutralize(raw, by=sector)
  emit neutral
```

### Volatility Targeting

Scale by volatility:

```sig
signal vol_target:
  raw = ret(prices, 20)
  vol = rolling_std(ret(prices, 1), 60)
  target_vol = 0.15  // 15% annualized
  scaled = raw * (target_vol / vol)
  emit zscore(scaled)
```

## Debugging Signals

### Check Distribution

A well-behaved signal should have:

- Mean ≈ 0
- Std ≈ 1
- Few extreme values

### Check Coverage

Ensure signal is computed for all assets:

```sig
signal with_coverage:
  raw = ret(prices, 20)
  // Handle missing data
  filled = fill_nan(raw, 0)
  emit zscore(filled)
```

### Check Stability

Highly unstable signals are suspicious:

```sig
// Compare signal values day-over-day
signal_change = abs(today_signal - yesterday_signal)
// If change is very large, investigate
```

## Best Practices

1. **Always normalize** - Use `zscore` for cross-sectional comparability
2. **Handle outliers** - Use `winsor` to clip extremes
3. **Adjust for volatility** - Don't let high-vol stocks dominate
4. **Consider turnover** - Balance responsiveness with trading costs
5. **Document your logic** - Explain the economic rationale
6. **Test out-of-sample** - Don't overfit to historical data

## Next Steps

- [Portfolio Construction](portfolio-construction.md) - Convert signals to weights
- [Operators Reference](../operators/index.md) - All available operators
- [Tutorials](../tutorials/index.md) - Build complete strategies
