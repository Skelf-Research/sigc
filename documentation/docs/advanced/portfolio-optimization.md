# Portfolio Optimization

Construct optimal portfolios beyond simple equal-weighting.

## Overview

Portfolio optimization finds weights that maximize objectives subject to constraints:

$$\max_w \quad f(w) \quad \text{subject to} \quad g(w) \leq 0$$

## Mean-Variance Optimization

### Classic Markowitz

```sig
portfolio mean_variance:
  weights = optimize(
    objective = maximize("sharpe"),
    expected_returns = signal,
    covariance = rolling_cov(ret(prices, 1), 60),
    constraints:
      sum_weights = 1.0
      min_weight = 0.0
      max_weight = 0.10
  )
```

### With Risk Aversion

```sig
portfolio risk_averse:
  weights = optimize(
    objective = maximize("utility"),
    utility = expected_return - 0.5 * risk_aversion * variance,
    risk_aversion = 3.0,  // Higher = more risk averse
    constraints:
      sum_weights = 1.0
  )
```

## Risk-Based Optimization

### Minimum Variance

```sig
portfolio min_variance:
  weights = optimize(
    objective = minimize("variance"),
    covariance = rolling_cov(ret(prices, 1), 60),
    constraints:
      sum_weights = 1.0
      min_weight = 0.0
  )
```

### Risk Parity

Equal risk contribution from each asset:

```sig
portfolio risk_parity:
  // Each asset contributes equally to portfolio risk
  weights = optimize(
    objective = equalize("risk_contribution"),
    covariance = rolling_cov(ret(prices, 1), 60),
    constraints:
      sum_weights = 1.0
      min_weight = 0.01
  )
```

### Simplified Risk Parity

```sig
signal risk_parity_weights:
  vol = rolling_std(ret(prices, 1), 60)
  inv_vol = 1 / vol
  weights = inv_vol / sum(inv_vol)
  emit weights
```

### Maximum Diversification

```sig
portfolio max_diversification:
  weights = optimize(
    objective = maximize("diversification_ratio"),
    // DR = weighted avg vol / portfolio vol
    covariance = rolling_cov(ret(prices, 1), 60),
    constraints:
      sum_weights = 1.0
      min_weight = 0.0
  )
```

## Constrained Optimization

### Long-Short with Constraints

```sig
portfolio constrained:
  weights = optimize(
    objective = maximize("expected_return"),
    expected_returns = signal,
    covariance = cov_matrix,
    constraints:
      // Exposure constraints
      gross_exposure = 2.0      // 200% gross
      net_exposure = 0.0        // Dollar neutral

      // Position constraints
      min_weight = -0.05
      max_weight = 0.05

      // Sector constraints
      max_sector_exposure = 0.25

      // Risk constraints
      max_volatility = 0.15
  )
```

### Tracking Error Constraint

```sig
portfolio tracking:
  weights = optimize(
    objective = maximize("information_ratio"),
    benchmark = spy_weights,
    constraints:
      max_tracking_error = 0.05  // 5% TE
      max_active_weight = 0.03
  )
```

## Black-Litterman

Combine market equilibrium with views:

```sig
portfolio black_litterman:
  // Market equilibrium returns (from CAPM)
  equilibrium = market_cap_weights * risk_aversion * cov_matrix

  // Your views
  views:
    - asset: AAPL
      return: 0.15
      confidence: 0.8
    - asset: MSFT
      return: 0.12
      confidence: 0.6

  weights = optimize(
    method = "black_litterman",
    equilibrium_returns = equilibrium,
    views = views,
    covariance = cov_matrix,
    tau = 0.05  // Uncertainty in equilibrium
  )
```

## Hierarchical Risk Parity

```sig
portfolio hrp:
  weights = optimize(
    method = "hierarchical_risk_parity",
    returns = ret(prices, 1),
    lookback = 252,
    linkage = "ward"  // Clustering method
  )
```

## Robust Optimization

### Uncertainty in Returns

```sig
portfolio robust:
  weights = optimize(
    method = "robust_mean_variance",
    expected_returns = signal,
    return_uncertainty = 0.02,  // ±2% uncertainty
    covariance = cov_matrix,
    constraints:
      max_weight = 0.10
  )
```

### Resampled Optimization

```sig
portfolio resampled:
  weights = optimize(
    method = "resampled_efficient_frontier",
    expected_returns = signal,
    covariance = cov_matrix,
    n_samples = 1000,
    constraints:
      sum_weights = 1.0
  )
```

## Transaction Cost Aware

```sig
portfolio turnover_penalized:
  weights = optimize(
    objective = maximize("return - turnover_cost"),
    expected_returns = signal,
    current_weights = previous_weights,
    turnover_cost = 0.001,  // 10 bps per unit turnover
    constraints:
      sum_weights = 1.0
  )
```

## Multi-Period Optimization

```sig
portfolio multi_period:
  weights = optimize(
    method = "multi_period",
    horizon = 5,  // 5 periods ahead
    expected_returns = [signal_t1, signal_t2, signal_t3, signal_t4, signal_t5],
    transaction_costs = tc.bps(10),
    constraints:
      max_turnover_per_period = 0.25
  )
```

## Factor-Constrained

```sig
portfolio factor_constrained:
  weights = optimize(
    objective = maximize("alpha"),
    alpha = signal,
    factor_exposures:
      market: [0.9, 1.1]     // Beta between 0.9 and 1.1
      size: [-0.1, 0.1]      // Size neutral
      value: [0.0, 0.3]      // Slight value tilt
    constraints:
      gross_exposure = 2.0
      net_exposure = 0.0
  )
```

## Complete Example

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Alpha signal
signal alpha:
  momentum = zscore(ret(prices, 60))
  value = zscore(book_to_market)
  quality = zscore(roe)
  combined = 0.4 * momentum + 0.4 * value + 0.2 * quality
  emit neutralize(combined, by=sectors)

// Covariance estimation
signal covariance:
  returns = ret(prices, 1)
  // Ledoit-Wolf shrinkage
  cov = shrunk_covariance(returns, 252, method="ledoit_wolf")
  emit cov

// Optimized portfolio
portfolio optimized:
  weights = optimize(
    objective = maximize("sharpe"),
    expected_returns = alpha,
    covariance = covariance,

    constraints:
      // Exposure
      gross_exposure = 2.0
      net_exposure = 0.0

      // Position limits
      min_weight = -0.05
      max_weight = 0.05

      // Sector limits
      max_sector = 0.25

      // Risk limits
      max_volatility = 0.12

      // Factor limits
      beta: [0.8, 1.2]

      // Turnover
      max_turnover = 0.25
  )

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Optimization Methods

| Method | Best For | Complexity |
|--------|----------|------------|
| Mean-Variance | Simple cases | Low |
| Risk Parity | Equal risk | Low |
| Min Variance | Risk reduction | Medium |
| Black-Litterman | Incorporating views | Medium |
| HRP | Robustness | Medium |
| Robust | Uncertain inputs | High |

## Best Practices

### 1. Use Shrinkage Estimation

```sig
cov = shrunk_covariance(returns, 252)
```

### 2. Add Position Limits

Unconstrained optimization produces extreme weights.

### 3. Penalize Turnover

```sig
objective = return - turnover_cost * turnover
```

### 4. Test Robustness

Small input changes shouldn't cause large weight changes.

### 5. Monitor Factor Exposures

Ensure optimization doesn't create unintended bets.

## Next Steps

- [Risk Models](risk-models.md) - Covariance estimation
- [Factor Models](factor-models.md) - Factor constraints
- [Constraints](../backtesting/constraints.md) - Constraint types
