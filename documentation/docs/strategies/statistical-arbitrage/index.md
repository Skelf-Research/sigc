# Statistical Arbitrage Strategies

Market-neutral strategies exploiting price relationships.

## Overview

Statistical arbitrage (stat arb) strategies exploit temporary mispricings between related securities, typically maintaining market neutrality.

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Pairs Trading](pairs-trading.md) | Trade related stock pairs | Intermediate |
| [Sector Neutral](sector-neutral.md) | Neutral within sectors | Intermediate |
| [Beta Neutral](beta-neutral.md) | Market-neutral construction | Advanced |

## Quick Example

```sig
signal pairs:
  // Trade spread between related stocks
  spread = prices[A] - beta * prices[B]
  zscore = (spread - rolling_mean(spread, 60)) / rolling_std(spread, 60)

  // Revert spread extremes
  emit -zscore

portfolio main:
  weights = pairs.long_short()
  constraints:
    net_exposure = 0.0  // Market neutral
  backtest from 2015-01-01 to 2024-12-31
```

## Key Characteristics

- **Market neutral**: Long and short positions offset
- **Low correlation**: Returns independent of market direction
- **Higher turnover**: Active trading to capture small mispricings
- **Capacity constrained**: Limited by liquidity

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.8 - 1.5 |
| Annual Return | 4% - 10% |
| Max Drawdown | 5% - 15% |
| Market Beta | -0.1 to 0.1 |
