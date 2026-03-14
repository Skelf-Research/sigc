# Mean Reversion Strategies

Strategies that profit from prices returning to equilibrium.

## Overview

Mean reversion strategies bet that extreme price moves will reverse:
- Oversold stocks will rise
- Overbought stocks will fall
- Prices gravitate toward "fair value"

## Why Mean Reversion Works

1. **Overreaction**: Markets overreact to news
2. **Liquidity**: Forced sellers create opportunities
3. **Behavioral**: Fear and greed cause extremes
4. **Fundamental anchor**: Prices tied to value long-term

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Statistical Reversion](statistical-reversion.md) | Z-score based reversion | Basic |
| [RSI Reversion](rsi-reversion.md) | Relative Strength Index | Basic |
| [Bollinger Bands](bollinger-bands.md) | Band-based signals | Intermediate |
| [Short-Term Reversal](short-term-reversal.md) | 1-week reversals | Basic |

## Quick Example

```sig
data:
  source = "prices.parquet"
  format = parquet

signal mean_reversion:
  // Price vs 20-day moving average
  ma_20 = rolling_mean(prices, 20)
  deviation = (prices - ma_20) / rolling_std(prices, 20)

  // Buy oversold (negative z), short overbought (positive z)
  emit -zscore(deviation)

portfolio main:
  weights = rank(mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Key Considerations

### Works Best In

- Range-bound markets
- High volatility (more mispricings)
- Liquid markets

### Fails In

- Strong trends
- Regime changes
- Momentum markets

### Risk Management

```sig
// Add trend filter to avoid trending markets
signal safe_reversion:
  reversion = mean_reversion_signal
  trend_strength = abs(ma_50 - ma_200) / ma_200

  // Only revert in range-bound markets
  low_trend = trend_strength < 0.05

  emit where(low_trend, reversion, reversion * 0.3)
```

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.3 - 0.7 |
| Annual Return | 3% - 8% |
| Max Drawdown | 10% - 25% |
| Turnover | 400% - 800% |

## Research References

- Jegadeesh (1990): "Evidence of Predictable Behavior of Security Returns"
- Lo & MacKinlay (1990): "When Are Contrarian Profits Due to Stock Market Overreaction?"
- DeBondt & Thaler (1985): "Does the Stock Market Overreact?"
