# Technical Strategies

Strategies based on price and volume patterns.

## Overview

Technical strategies use price action and chart patterns rather than fundamentals.

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Moving Averages](moving-averages.md) | MA crossover systems | Basic |
| [Breakouts](breakouts.md) | Support/resistance breaks | Basic |
| [MACD Strategy](macd-strategy.md) | MACD signals | Intermediate |
| [Volume Patterns](volume-patterns.md) | Volume-based signals | Intermediate |

## Quick Example

```sig
signal macd:
  fast = ema(prices, 12)
  slow = ema(prices, 26)
  macd_line = fast - slow
  signal_line = ema(macd_line, 9)

  emit zscore(macd_line - signal_line)

portfolio main:
  weights = rank(macd).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.3 - 0.6 |
| Annual Return | 3% - 8% |
| Turnover | 300% - 600% |
