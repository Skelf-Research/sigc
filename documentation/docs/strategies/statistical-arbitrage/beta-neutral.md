# Beta Neutral Strategy

Construct portfolios with zero market exposure.

## Strategy Overview

Adjust position sizes so portfolio beta equals zero. Returns are independent of market direction.

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Calculate betas
signal stock_beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
  emit beta

// Alpha signal
signal alpha:
  emit neutralize(zscore(ret(prices, 60)), by=sectors)

// Beta-adjusted weights
signal beta_adjusted:
  raw_signal = alpha
  beta = stock_beta

  // Adjust signal to target beta = 0
  // Higher beta stocks get smaller positions
  adjustment = 1 / (abs(beta) + 0.5)

  emit raw_signal * adjustment

portfolio beta_neutral:
  weights = rank(beta_adjusted).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    beta: [-0.05, 0.05]  // Near-zero beta

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Portfolio Beta Calculation

```sig
// Portfolio beta = sum(weight × beta)
signal portfolio_beta:
  weighted_beta = sum(weights * stock_beta)
  emit weighted_beta
```

## Dynamic Beta Hedging

```sig
signal beta_hedged:
  raw_weights = rank(alpha).long_short(top=0.2, bottom=0.2)

  // Calculate portfolio beta
  port_beta = sum(raw_weights * stock_beta)

  // Hedge with market index
  hedge_weight = -port_beta  // Short market to neutralize

  // Final weights include hedge
  emit raw_weights  // Plus hedge_weight on market index
```

## Expected Results

```
Backtest Results: beta_neutral
==============================
Period: 2015-01-01 to 2024-12-31

Returns:
  Annual Return: 4.2%
  Sharpe Ratio: 0.95

Market Neutrality:
  Avg Beta: 0.01
  Max Beta: 0.08
  Correlation with SPY: 0.05
```

## See Also

- [Pairs Trading](pairs-trading.md)
- [Sector Neutral](sector-neutral.md)
