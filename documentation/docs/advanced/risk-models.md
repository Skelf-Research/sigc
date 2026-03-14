# Risk Models

Estimate and manage portfolio risk.

## Overview

Risk models help:

- Estimate portfolio volatility
- Decompose risk by factors
- Set risk budgets
- Monitor risk in real-time

## Volatility Estimation

### Historical Volatility

```sig
signal hist_vol:
  daily_ret = ret(prices, 1)
  vol_20d = rolling_std(daily_ret, 20) * sqrt(252)
  vol_60d = rolling_std(daily_ret, 60) * sqrt(252)

  emit vol_60d
```

### Exponential Weighted

```sig
signal ewma_vol:
  daily_ret = ret(prices, 1)

  // Exponentially weighted variance
  decay = 0.94  // RiskMetrics decay factor
  var = ema(daily_ret * daily_ret, 20)
  vol = sqrt(var) * sqrt(252)

  emit vol
```

### GARCH-Style

```sig
signal garch_vol:
  daily_ret = ret(prices, 1)

  // Simplified GARCH(1,1)
  omega = 0.00001
  alpha = 0.1
  beta = 0.85

  // Update variance
  prev_var = lag(variance, 1)
  variance = omega + alpha * daily_ret * daily_ret + beta * prev_var
  vol = sqrt(variance) * sqrt(252)

  emit vol
```

## Correlation Estimation

### Rolling Correlation

```sig
signal rolling_corr:
  ret_a = ret(stock_a, 1)
  ret_b = ret(stock_b, 1)

  corr_60d = rolling_corr(ret_a, ret_b, 60)

  emit corr_60d
```

### Shrinkage Estimation

For stable correlation matrices:

```sig
signal shrunk_corr:
  // Shrink sample correlation toward identity
  sample_corr = correlation_matrix(returns)
  shrinkage = 0.2

  shrunk = (1 - shrinkage) * sample_corr + shrinkage * identity

  emit shrunk
```

## Covariance Matrix

### Sample Covariance

```sig
signal sample_cov:
  returns = ret(prices, 1)
  cov = rolling_cov(returns, 60)
  emit cov
```

### Ledoit-Wolf Shrinkage

```yaml
risk:
  covariance:
    method: ledoit_wolf
    lookback: 252
    shrinkage: auto  # Optimal shrinkage
```

### Factor Covariance

```yaml
risk:
  covariance:
    method: factor
    factors: [market, size, value, momentum]
    specific_risk: diagonal
```

## Portfolio Risk

### Portfolio Volatility

```sig
signal portfolio_vol:
  // weights: vector of portfolio weights
  // cov: covariance matrix

  // Portfolio variance = w' Σ w
  port_var = quadratic_form(weights, cov)
  port_vol = sqrt(port_var) * sqrt(252)

  emit port_vol
```

### Value at Risk

```sig
signal var_95:
  returns = ret(prices, 1)
  port_ret = sum(weights * returns)

  // Historical VaR
  var = quantile(port_ret, 0.05)

  emit var
```

### Conditional VaR (CVaR)

```sig
signal cvar_95:
  returns = ret(prices, 1)
  port_ret = sum(weights * returns)

  // Average of worst 5%
  var = quantile(port_ret, 0.05)
  cvar = mean(where(port_ret < var, port_ret, 0))

  emit cvar
```

## Factor Risk

### Factor Decomposition

```
Portfolio Risk = Factor Risk + Specific Risk

Factor Risk = Σ (βi × βj × σi × σj × ρij)
Specific Risk = Σ (wi² × σi_specific²)
```

### Risk Attribution

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  risk_attribution:
    factors: [market, size, value, momentum]
    output: risk_report
```

Output:

```
Risk Attribution:
Factor      | Beta  | Factor Vol | Contribution | % Total
------------+-------+------------+--------------+--------
Market      |  0.72 |    16.5%   |     8.5%     |   56%
Size        |  0.35 |     8.2%   |     2.1%     |   14%
Value       | -0.15 |     6.5%   |     0.8%     |    5%
Momentum    |  0.45 |     9.2%   |     2.2%     |   15%
Specific    |       |            |     1.5%     |   10%
------------+-------+------------+--------------+--------
Total       |       |            |    15.1%     |  100%
```

## Risk Constraints

### Volatility Target

```sig
portfolio vol_targeted:
  raw_weights = rank(signal).long_short(top=0.2, bottom=0.2)

  // Scale to target volatility
  port_vol = estimate_volatility(raw_weights)
  scale = target_vol / port_vol
  weights = raw_weights * scale

  constraints:
    target_volatility = 0.10  # 10% annual vol
```

### Risk Parity

```sig
portfolio risk_parity:
  // Equal risk contribution from each asset
  vol = rolling_std(ret(prices, 1), 60)
  inv_vol = 1 / vol

  // Weight inversely to volatility
  weights = inv_vol / sum(inv_vol)
```

### Maximum Drawdown

```sig
portfolio drawdown_controlled:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    max_expected_drawdown = 0.15
```

## Risk Monitoring

### Real-Time Risk

```yaml
monitoring:
  risk:
    interval_seconds: 60
    metrics:
      - portfolio_volatility
      - var_95
      - max_position_risk
      - factor_exposures
```

### Risk Alerts

```yaml
alerting:
  rules:
    - name: high_volatility
      condition: "portfolio_vol > 0.20"
      severity: warning

    - name: var_breach
      condition: "var_95 > 0.03"
      severity: high

    - name: drawdown
      condition: "drawdown > 0.10"
      severity: critical
```

## Risk Budgeting

### Budget by Factor

```sig
portfolio factor_budgeted:
  weights = optimize(
    objective = maximize("return"),
    constraints:
      market_risk_budget = 0.50    # 50% from market
      factor_risk_budget = 0.30   # 30% from factors
      specific_risk_budget = 0.20 # 20% specific
  )
```

### Budget by Sector

```sig
portfolio sector_budgeted:
  weights = optimize(
    objective = maximize("sharpe"),
    constraints:
      sector_risk:
        Technology: 0.20
        Healthcare: 0.15
        Financials: 0.15
        # ...
  )
```

## Complete Example

```sig
data:
  source = "prices.parquet"
  format = parquet

// Risk estimation
signal volatility:
  daily_ret = ret(prices, 1)
  vol = rolling_std(daily_ret, 60) * sqrt(252)
  emit vol

signal correlation:
  ret_matrix = ret(prices, 1)
  corr = rolling_corr_matrix(ret_matrix, 60)
  emit corr

// Alpha signal
signal momentum:
  emit zscore(ret(prices, 60))

// Risk-aware portfolio
portfolio risk_managed:
  // Raw weights from signal
  raw_weights = rank(momentum).long_short(top=0.2, bottom=0.2)

  // Estimate portfolio risk
  port_vol = portfolio_volatility(raw_weights, volatility, correlation)

  // Scale to target volatility
  target_vol = 0.10
  scale = where(port_vol > target_vol, target_vol / port_vol, 1.0)
  weights = raw_weights * scale

  constraints:
    max_position = 0.05
    max_sector = 0.25
    dollar_neutral = true

  risk_monitoring:
    enabled: true
    metrics: [portfolio_vol, var_95, factor_exposures]

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Best Practices

### 1. Use Shrinkage Estimation

Unstable with many assets:

```yaml
covariance:
  method: ledoit_wolf
```

### 2. Monitor Risk in Real-Time

```yaml
monitoring:
  risk:
    interval_seconds: 60
```

### 3. Set Multiple Risk Limits

```yaml
constraints:
  target_volatility = 0.10
  max_drawdown = 0.15
  max_var_95 = 0.02
```

### 4. Decompose Risk by Factor

Understand risk sources.

### 5. Stress Test

Test under extreme scenarios.

## Next Steps

- [Factor Models](factor-models.md) - Factor construction
- [Portfolio Optimization](portfolio-optimization.md) - Risk-aware optimization
- [Constraints](../backtesting/constraints.md) - Risk constraints
