# Momentum + Quality Strategy

Combine momentum with quality to improve risk-adjusted returns.

## Strategy Overview

Pure momentum can crash during market reversals. Quality filtering improves stability by selecting momentum stocks with strong fundamentals.

## Why Quality Helps

| Issue with Momentum | Quality Solution |
|---------------------|------------------|
| Momentum crashes | Quality stocks more resilient |
| Junk rallies | Filter out low-quality names |
| High volatility | Quality reduces vol |
| Factor crowding | Differentiated signal |

## The Signal

```sig
signal momentum_quality:
  // Momentum component
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  momentum = zscore(ret_12m - ret_1m)

  // Quality component
  profitability = zscore(roe)
  stability = -zscore(rolling_std(earnings, 8))
  quality = 0.6 * profitability + 0.4 * stability

  // Combine: momentum in quality stocks
  combined = momentum * (1 + 0.3 * quality)

  emit neutralize(combined, by=sectors)
```

## Complete Strategy

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Pure momentum
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit zscore(ret_12m - ret_1m)

// Quality metrics
signal quality:
  // Profitability
  roe_z = zscore(roe)
  roa_z = zscore(roa)
  margin_z = zscore(gross_margin)

  // Stability
  earnings_stability = -zscore(rolling_std(earnings, 8))
  leverage = -zscore(debt_to_equity)

  // Combined quality
  profitability = 0.4 * roe_z + 0.3 * roa_z + 0.3 * margin_z
  safety = 0.5 * earnings_stability + 0.5 * leverage

  emit 0.7 * profitability + 0.3 * safety

// Quality-filtered momentum
signal momentum_quality:
  mom = momentum
  qual = quality

  // Method 1: Multiplicative
  // Higher quality amplifies momentum signal
  combined = mom * (1 + 0.3 * qual)

  // Sector neutralize
  emit neutralize(combined, by=sectors)

// Portfolio
portfolio momentum_quality:
  weights = rank(momentum_quality).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Combination Methods

### Method 1: Multiplicative

Quality amplifies good momentum signals:

```sig
signal multiplicative:
  combined = momentum * (1 + 0.3 * quality)
  emit combined
```

### Method 2: Additive

Equal consideration of both factors:

```sig
signal additive:
  combined = 0.6 * momentum + 0.4 * quality
  emit combined
```

### Method 3: Quality Filter

Only trade momentum in high-quality stocks:

```sig
signal quality_filtered:
  qual_threshold = quantile(quality, 0.5)
  high_quality = quality > qual_threshold

  // Only take momentum bets in quality stocks
  emit where(high_quality, momentum, 0)
```

### Method 4: Interaction

Long quality momentum, short junk momentum:

```sig
signal interaction:
  // Quality quintiles
  qual_high = quality > quantile(quality, 0.8)
  qual_low = quality < quantile(quality, 0.2)

  // Momentum direction
  mom_high = momentum > 0
  mom_low = momentum < 0

  // Long: high quality + high momentum
  // Short: low quality + low momentum
  signal = where(qual_high and mom_high, momentum,
           where(qual_low and mom_low, momentum * 0.5,
           momentum * 0.3))

  emit signal
```

## Quality Definitions

### Profitability Quality

```sig
signal profitability_quality:
  roe = net_income / equity
  roa = net_income / assets
  gross_margin = (revenue - cogs) / revenue

  emit zscore(0.4 * roe + 0.3 * roa + 0.3 * gross_margin)
```

### Earnings Quality

```sig
signal earnings_quality:
  // Accruals (low = good)
  accruals = (net_income - operating_cash_flow) / assets
  accrual_score = -zscore(accruals)

  // Earnings stability
  stability = -zscore(rolling_std(earnings, 8))

  emit 0.5 * accrual_score + 0.5 * stability
```

### Safety Quality

```sig
signal safety_quality:
  // Low leverage
  leverage = debt / equity
  leverage_score = -zscore(leverage)

  // Low volatility
  vol = rolling_std(ret(prices, 1), 252) * sqrt(252)
  vol_score = -zscore(vol)

  emit 0.5 * leverage_score + 0.5 * vol_score
```

## Expected Results

```
Backtest Results: momentum_quality
==================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 185%
  Annual Return: 7.8%
  Annual Volatility: 9.5%
  Sharpe Ratio: 0.82

vs Pure Momentum:
  Momentum Sharpe: 0.65
  Quality Improvement: +26%

Risk:
  Max Drawdown: -16.2%
  (vs Momentum -24.5%)

Quality Metrics:
  Avg ROE (longs): 18.2%
  Avg ROE (shorts): 6.8%
```

## Risk Benefits

### Drawdown Reduction

```
Max Drawdowns:
  Pure Momentum:     -24.5%
  Momentum + Quality: -16.2%
  Improvement:        34%
```

### Crisis Performance

```
2020 COVID Crash:
  Pure Momentum:     -18.3%
  Momentum + Quality: -12.1%

2022 Bear Market:
  Pure Momentum:      -9.5%
  Momentum + Quality:  -5.8%
```

## Parameter Optimization

```sig
params:
  mom_lookback: [126, 189, 252]
  quality_weight: range(0.2, 0.5, 0.1)

signal optimized:
  mom = zscore(ret(prices, mom_lookback) - ret(prices, 21))
  qual = quality

  mom_weight = 1 - quality_weight
  combined = mom_weight * mom + quality_weight * qual

  emit neutralize(combined, by=sectors)

portfolio optimized:
  weights = rank(optimized).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(train_years=5, test_years=1) from 2010-01-01 to 2024-12-31
```

## See Also

- [Price Momentum](price-momentum.md)
- [Multi-Factor Strategies](../multi-factor/index.md)
- [Quality Factor](../multi-factor/quality-factor.md)
