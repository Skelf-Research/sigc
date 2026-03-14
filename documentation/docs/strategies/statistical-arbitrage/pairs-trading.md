# Pairs Trading Strategy

Trade the spread between related securities.

## Strategy Overview

Find pairs of stocks that move together. When they diverge, bet on convergence.

## The Concept

```
Stock A and Stock B typically move together (correlation > 0.8)
Spread = A - β × B

When spread is high → Short A, Long B
When spread is low → Long A, Short B
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Calculate pair relationships
signal pair_spread:
  // For each stock, find its sector peer
  sector_avg = group_mean(prices, by=sectors)

  // Stock vs sector average
  spread = prices - sector_avg

  // Z-score of spread
  spread_z = (spread - rolling_mean(spread, 60)) / rolling_std(spread, 60)

  // Revert extreme spreads
  emit -zscore(spread_z)

portfolio pairs:
  weights = rank(pair_spread).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0  // Market neutral
    max_sector = 0.25

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## Specific Pair Example

```sig
// Trade GOOGL vs META spread
signal googl_meta:
  // Calculate beta
  ret_googl = ret(prices[GOOGL], 1)
  ret_meta = ret(prices[META], 1)
  beta = rolling_cov(ret_googl, ret_meta, 60) / rolling_var(ret_meta, 60)

  // Spread
  spread = prices[GOOGL] - beta * prices[META]

  // Z-score
  z = (spread - rolling_mean(spread, 60)) / rolling_std(spread, 60)

  // Signal: short when z high, long when z low
  emit -z
```

## Cointegration-Based

```sig
signal cointegrated_pairs:
  // Use residuals from cointegration relationship
  // (Requires pre-computed cointegration parameters)

  residual = prices - coint_coefficient * pair_prices

  // Mean-reverting residual
  z = (residual - rolling_mean(residual, 60)) / rolling_std(residual, 60)

  emit -zscore(z)
```

## Expected Results

```
Backtest Results: pairs_trading
===============================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 68%
  Annual Return: 5.5%
  Annual Volatility: 5.2%
  Sharpe Ratio: 1.06

Market Neutrality:
  Avg Beta: 0.02
  Max Beta: 0.12
```

## See Also

- [Sector Neutral](sector-neutral.md)
- [Beta Neutral](beta-neutral.md)
