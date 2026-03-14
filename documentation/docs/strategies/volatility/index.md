# Volatility Strategies

Strategies that trade volatility patterns.

## Overview

Volatility strategies exploit patterns in risk:
- Low volatility stocks outperform
- Volatility clusters and mean-reverts
- Volatility risk premium exists

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Low Volatility](low-volatility.md) | Defensive low-vol | Basic |
| [Volatility Targeting](volatility-targeting.md) | Risk-managed | Intermediate |
| [Vol-Adjusted Returns](vol-adjusted-returns.md) | Risk-normalized | Intermediate |
| [Volatility Breakout](volatility-breakout.md) | Volatility expansion | Intermediate |

## Quick Example

```sig
signal low_volatility:
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  emit -zscore(vol)  // Low vol = high score

portfolio main:
  weights = rank(low_volatility).long_only(top=0.3)
  backtest from 2015-01-01 to 2024-12-31
```

## Key Considerations

### Low Vol Anomaly

Lower risk stocks have historically delivered better risk-adjusted returns. Theories:
- Leverage constraints
- Lottery preferences
- Benchmarking behavior

### Volatility Clustering

High vol today predicts high vol tomorrow:

```sig
signal vol_persistence:
  vol_20 = rolling_std(ret(prices, 1), 20)
  vol_60 = rolling_std(ret(prices, 1), 60)
  emit vol_20 / vol_60  // Rising vol indicator
```

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.5 - 1.0 |
| Annual Return | 4% - 8% |
| Max Drawdown | 8% - 20% |
| Beta | 0.5 - 0.8 |
