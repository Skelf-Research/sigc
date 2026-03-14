# Momentum Strategies

Strategies that profit from price trends continuing.

## Overview

Momentum strategies exploit the tendency for winning stocks to keep winning and losing stocks to keep losing. This is one of the most well-documented anomalies in finance.

## Why Momentum Works

1. **Behavioral**: Investors underreact to news initially
2. **Herding**: Success attracts more buyers
3. **Slow information diffusion**: Not everyone gets news at once
4. **Analyst coverage**: Forecasts adjust slowly

## Strategies in This Section

| Strategy | Description | Complexity |
|----------|-------------|------------|
| [Price Momentum](price-momentum.md) | Classic 12-1 momentum | Basic |
| [Industry Momentum](industry-momentum.md) | Sector rotation | Intermediate |
| [Trend Following](trend-following.md) | Moving average systems | Basic |
| [Momentum + Quality](momentum-quality.md) | Quality-filtered momentum | Intermediate |

## Quick Example

```sig
data:
  source = "prices.parquet"
  format = parquet

signal momentum:
  // 12-month return, skip last month (12-1)
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  momentum = ret_12m - ret_1m
  emit zscore(momentum)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Key Considerations

### Momentum Crashes

Momentum can experience sharp reversals:
- 2009 momentum crash
- Market turning points

Mitigation:
```sig
// Add volatility scaling
vol_scale = where(vix > 30, 0.5, 1.0)
weights = base_weights * vol_scale
```

### Turnover

Momentum strategies can have high turnover. Balance:
- Rebalancing frequency
- Transaction costs
- Signal decay

### Capacity

Works best with smaller positions due to:
- Market impact
- Signal crowding

## Expected Performance

| Metric | Typical Range |
|--------|---------------|
| Sharpe | 0.4 - 0.8 |
| Annual Return | 5% - 12% |
| Max Drawdown | 15% - 30% |
| Turnover | 200% - 400% |

## Research References

- Jegadeesh & Titman (1993): "Returns to Buying Winners and Selling Losers"
- Carhart (1997): "On Persistence in Mutual Fund Performance"
- Asness et al. (2013): "Value and Momentum Everywhere"
