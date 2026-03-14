# Multi-Factor Strategies

Combine multiple alpha sources for robust performance.

## Overview

Multi-factor strategies diversify across different return drivers, reducing dependence on any single factor.

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Classic Multi-Factor](classic-multi-factor.md) | Value + Momentum + Quality | Intermediate |
| [Quality Factor](quality-factor.md) | Profitability and stability | Basic |
| [Factor Timing](factor-timing.md) | Dynamic factor weights | Advanced |
| [Custom Factors](custom-factors.md) | Building your own | Advanced |

## Quick Example

```sig
signal multi_factor:
  momentum = zscore(ret(prices, 60))
  value = zscore(book_to_market)
  quality = zscore(roe)

  emit 0.4 * momentum + 0.3 * value + 0.3 * quality

portfolio main:
  weights = rank(multi_factor).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Why Multi-Factor?

| Single Factor | Multi-Factor |
|---------------|--------------|
| Can underperform for years | More consistent |
| Higher volatility | Diversified risk |
| Factor timing risk | Reduced timing dependency |
| Simpler | More robust |

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.6 - 1.0 |
| Annual Return | 6% - 12% |
| Max Drawdown | 12% - 22% |
