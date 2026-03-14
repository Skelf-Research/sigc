# Chapter 3: Building Trading Signals

Transform data into actionable trading signals.

## What is a Signal?

A signal is a numerical score that predicts expected returns:

- **Higher score** → expect higher returns → buy more
- **Lower score** → expect lower returns → sell/short

```sig
signal momentum:
  // Stocks with higher past returns get higher scores
  emit zscore(ret(prices, 60))
```

## Signal Properties

### Good Signals Have:

| Property | Description | Example |
|----------|-------------|---------|
| Predictive | Forecasts future returns | High momentum → high future returns |
| Stable | Doesn't change too rapidly | Monthly, not minute-by-minute |
| Diversified | Works across many assets | Not just one stock |
| Economically sensible | Has logical explanation | "Winners keep winning" |

### Signal Quality Metrics

**Information Coefficient (IC):**
Correlation between signal and forward returns.

```
IC = 0.05 → Good signal
IC = 0.10 → Excellent signal
IC > 0.15 → Suspicious (check for errors)
```

## Building Your First Signal

### Step 1: Start with a Hypothesis

"Stocks with strong recent performance continue to outperform."

### Step 2: Implement Mathematically

```sig
signal momentum:
  // Calculate 60-day return
  past_return = ret(prices, 60)

  // Standardize cross-sectionally
  emit zscore(past_return)
```

### Step 3: Test and Validate

```sig
portfolio test_momentum:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Common Signal Types

### Momentum Signals

Past returns predict future returns:

```sig
// Simple momentum
signal momentum_simple:
  emit zscore(ret(prices, 60))

// 12-1 momentum (skip last month)
signal momentum_12_1:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit zscore(ret_12m - ret_1m)

// Relative strength
signal relative_strength:
  stock_ret = ret(prices, 60)
  market_ret = ret(market, 60)
  emit zscore(stock_ret - market_ret)
```

### Value Signals

Cheap stocks outperform:

```sig
// Book-to-market
signal value_btm:
  emit zscore(book_value / market_cap)

// Earnings yield
signal value_ey:
  emit zscore(earnings / prices)

// Combined value
signal value_composite:
  btm = zscore(book_value / market_cap)
  ey = zscore(earnings / prices)
  emit 0.5 * btm + 0.5 * ey
```

### Quality Signals

High-quality companies outperform:

```sig
// Profitability
signal quality_roe:
  emit zscore(net_income / equity)

// Earnings stability
signal quality_stability:
  emit -zscore(rolling_std(earnings, 8))

// Combined quality
signal quality_composite:
  roe = zscore(net_income / equity)
  stability = -zscore(rolling_std(earnings, 8))
  emit 0.6 * roe + 0.4 * stability
```

### Low Volatility Signals

Low-risk stocks outperform (risk-adjusted):

```sig
// Inverse volatility
signal low_vol:
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  emit -zscore(vol)

// Beta
signal low_beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
  emit -zscore(beta)
```

### Mean Reversion Signals

Extreme moves revert:

```sig
// Price vs moving average
signal mean_reversion:
  ma = rolling_mean(prices, 20)
  deviation = (prices - ma) / ma
  emit -zscore(deviation)

// RSI-based
signal rsi_reversion:
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)
  rs = ema(gains, 14) / (ema(losses, 14) + 0.0001)
  rsi = 100 - 100 / (1 + rs)
  emit zscore(50 - rsi)
```

## Signal Transformation

### Standardization

Make signals comparable:

```sig
// Z-score (mean=0, std=1)
signal standardized:
  emit zscore(raw_signal)

// Rank (uniform distribution)
signal ranked:
  emit rank(raw_signal) / count(raw_signal)
```

### Winsorization

Handle outliers:

```sig
signal winsorized:
  lower = quantile(raw_signal, 0.01)
  upper = quantile(raw_signal, 0.99)
  emit clip(raw_signal, lower, upper)
```

### Sector Neutralization

Remove sector bias:

```sig
signal sector_neutral:
  // Demean within each sector
  emit neutralize(raw_signal, by=sectors)
```

### Smoothing

Reduce noise:

```sig
signal smoothed:
  // Exponential moving average of signal
  emit ema(raw_signal, 5)
```

## Combining Signals

### Simple Average

```sig
signal combined_avg:
  emit (momentum + value + quality) / 3
```

### Weighted Average

```sig
signal combined_weighted:
  emit 0.4 * momentum + 0.4 * value + 0.2 * quality
```

### IC-Weighted

Weight by predictive power:

```sig
signal combined_ic:
  // Weights based on historical IC
  ic_mom = 0.04
  ic_val = 0.03
  ic_qual = 0.02
  total = ic_mom + ic_val + ic_qual

  emit (ic_mom * momentum + ic_val * value + ic_qual * quality) / total
```

### Rank Then Average

```sig
signal combined_rank:
  r_mom = rank(momentum)
  r_val = rank(value)
  r_qual = rank(quality)
  emit (r_mom + r_val + r_qual) / 3
```

## Signal Decay

Signals lose predictive power over time:

```
Day 1: IC = 0.05
Day 5: IC = 0.04
Day 21: IC = 0.03
Day 63: IC = 0.01
```

### Managing Decay

```sig
// More frequent rebalancing
portfolio fast_decay:
  weights = ...
  backtest rebal=5 from 2015-01-01 to 2024-12-31  // Every 5 days

// Less frequent for slow decay
portfolio slow_decay:
  weights = ...
  backtest rebal=21 from 2015-01-01 to 2024-12-31  // Monthly
```

## Complete Example

### Multi-Factor Signal

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Individual factors
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit neutralize(zscore(ret_12m - ret_1m), by=sectors)

signal value:
  btm = zscore(book_value / market_cap)
  ey = zscore(earnings / prices)
  raw = 0.5 * btm + 0.5 * ey
  emit neutralize(raw, by=sectors)

signal quality:
  prof = zscore(net_income / equity)
  stab = -zscore(rolling_std(earnings, 8))
  raw = 0.6 * prof + 0.4 * stab
  emit neutralize(raw, by=sectors)

signal low_vol:
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  emit neutralize(-zscore(vol), by=sectors)

// Combined signal
signal multi_factor:
  emit 0.30 * momentum + 0.30 * value + 0.25 * quality + 0.15 * low_vol

// Portfolio
portfolio main:
  weights = rank(multi_factor).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    max_sector = 0.20

  costs = tc.bps(10)

  backtest from 2015-01-01 to 2024-12-31
```

## Best Practices

### 1. Start Simple

```sig
// Start here
signal simple:
  emit zscore(ret(prices, 60))

// Not here
signal complex:
  emit complicated_formula(many, parameters, here)
```

### 2. Use Economic Intuition

Ask: "Why should this work?"

### 3. Normalize Everything

```sig
// Always standardize
emit zscore(raw_signal)

// Or rank
emit rank(raw_signal)
```

### 4. Sector Neutralize

```sig
// Avoid unintended sector bets
emit neutralize(signal, by=sectors)
```

### 5. Test Out-of-Sample

Use walk-forward validation to avoid overfitting.

## Exercises

1. Build a momentum signal using 12-1 month returns
2. Create a value signal combining book-to-market and earnings yield
3. Combine momentum and value with equal weights
4. Sector-neutralize your combined signal

## Next Chapter

Continue to [Chapter 4: The sigc Language](04-language.md) to master the DSL syntax.
