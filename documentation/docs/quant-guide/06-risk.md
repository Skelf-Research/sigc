# Chapter 6: Risk Management

Control portfolio risk to survive and thrive.

## Why Risk Management?

"Rule #1: Never lose money. Rule #2: Never forget rule #1." - Warren Buffett

Even the best signals fail sometimes. Risk management:

- Limits losses during drawdowns
- Ensures survival through bad periods
- Enables consistent compounding

## Types of Risk

### Market Risk

Exposure to overall market movements:

```sig
signal beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
  emit beta
```

### Specific Risk

Risk unique to individual securities:

```sig
signal idio_risk:
  // Risk unexplained by market
  residual = stock_returns - beta * market_returns
  idio_vol = rolling_std(residual, 60)
  emit idio_vol
```

### Factor Risk

Exposure to common factors (size, value, momentum):

```sig
// Monitor factor exposures
factor_exposure:
  market_beta: [0.9, 1.1]  // Target range
  size: [-0.1, 0.1]        // Neutral
  value: [0.0, 0.3]        // Slight tilt
```

### Concentration Risk

Too much exposure to single positions/sectors:

```sig
constraints:
  max_position = 0.03   // No more than 3% per stock
  max_sector = 0.20     // No more than 20% per sector
```

## Position-Level Risk

### Position Sizing

Limit exposure to individual stocks:

```sig
portfolio main:
  weights = rank(signal).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03  // Max 3% per position
  )
```

### Volatility-Based Sizing

Allocate less to volatile stocks:

```sig
signal vol_adjusted_weights:
  raw_weight = rank(signal) / count(signal)
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  inv_vol = 1 / vol
  adjusted = raw_weight * inv_vol
  emit adjusted / sum(adjusted)  // Normalize
```

## Portfolio-Level Risk

### Gross Exposure

Total absolute exposure:

```sig
constraints:
  gross_exposure = 2.0  // 100% long + 100% short
```

### Net Exposure

Directional market exposure:

```sig
constraints:
  net_exposure = 0.0     // Dollar neutral
  // or
  net_exposure: [-0.1, 0.1]  // Small range
```

### Sector Constraints

Limit sector concentration:

```sig
constraints:
  max_sector = 0.20  // Max 20% per sector
```

### Turnover Control

Limit trading activity:

```sig
constraints:
  max_turnover = 0.25  // Max 25% turnover per rebalance
```

## Volatility Targeting

### Fixed Volatility Target

Scale positions to target volatility:

```sig
portfolio vol_targeted:
  raw_weights = rank(signal).long_short(top=0.2, bottom=0.2)

  // Target 10% annual volatility
  vol_target = 0.10
  current_vol = portfolio_volatility(raw_weights)
  scale = vol_target / current_vol

  weights = raw_weights * scale

  backtest from 2015-01-01 to 2024-12-31
```

### Dynamic Scaling

Reduce exposure in high volatility environments:

```sig
signal vol_scale:
  current_vol = rolling_std(ret(market, 1), 20) * sqrt(252)
  target_vol = 0.15
  scale = clip(target_vol / current_vol, 0.5, 1.5)
  emit scale

portfolio adaptive:
  base_weights = rank(signal).long_short(top=0.2, bottom=0.2)
  weights = base_weights * vol_scale
```

## Drawdown Management

### Maximum Drawdown Monitoring

Track underwater periods:

```sig
signal drawdown:
  peak = rolling_max(portfolio_value, 252)
  dd = (portfolio_value - peak) / peak
  emit dd
```

### Drawdown-Based Deleveraging

Reduce exposure during drawdowns:

```sig
signal dd_scale:
  drawdown = current_drawdown
  // Scale down as drawdown increases
  scale = where(drawdown > -0.05, 1.0,
          where(drawdown > -0.10, 0.75,
          where(drawdown > -0.15, 0.50, 0.25)))
  emit scale
```

## Risk Metrics

### Value at Risk (VaR)

Maximum expected loss at confidence level:

```
VaR(95%) = 1.65 × σ × √days
```

For a portfolio with 10% annual vol:
```
Daily VaR(95%) = 1.65 × (0.10 / √252) = 1.04%
```

### Conditional VaR (CVaR)

Expected loss given VaR breach:

```
CVaR(95%) = Expected loss when loss > VaR(95%)
```

### Tracking Error

Volatility of returns vs benchmark:

```sig
tracking_error = std(portfolio_return - benchmark_return) * sqrt(252)
```

## Complete Risk Framework

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Signals
signal momentum:
  emit neutralize(zscore(ret(prices, 60)), by=sectors)

signal value:
  emit neutralize(zscore(book_to_market), by=sectors)

signal combined:
  emit 0.5 * momentum + 0.5 * value

// Volatility regime
signal vol_regime:
  market_vol = rolling_std(ret(market, 1), 20) * sqrt(252)
  long_vol = rolling_std(ret(market, 1), 60) * sqrt(252)
  high_vol = market_vol > long_vol * 1.3
  emit high_vol

// Risk-aware portfolio
portfolio risk_managed:
  // Base weights
  base_weights = rank(combined).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  // Scale down in high vol regime
  vol_scale = where(vol_regime, 0.7, 1.0)
  weights = base_weights * vol_scale

  constraints:
    // Exposure limits
    gross_exposure = 2.0
    net_exposure: [-0.1, 0.1]

    // Concentration limits
    max_position = 0.03
    max_sector = 0.20

    // Factor limits
    beta: [0.8, 1.2]

    // Turnover
    max_turnover = 0.25

    // Risk limit
    max_volatility = 0.12  // 12% target vol

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Risk Monitoring in Production

### Key Metrics to Monitor

| Metric | Frequency | Alert Level |
|--------|-----------|-------------|
| Daily P&L | Daily | >2% loss |
| Drawdown | Daily | >10% |
| Volatility | Daily | >1.5x target |
| Gross Exposure | Intraday | Outside limits |
| Factor Exposure | Daily | Outside limits |

### Alert Configuration

```yaml
alerts:
  - name: "Drawdown Alert"
    condition: drawdown > 0.10
    action: notify

  - name: "Circuit Breaker"
    condition: daily_loss > 0.03
    action: halt_trading
```

## Risk Budgeting

### Equal Risk Contribution

Each position contributes equally to risk:

```sig
portfolio risk_parity:
  // Weight inversely to volatility
  vol = rolling_std(ret(prices, 1), 60)
  inv_vol = 1 / vol
  weights = inv_vol / sum(inv_vol)
```

### Factor Risk Budgeting

Allocate risk to factors:

```
Total Risk Budget: 10%
  Market: 5%
  Momentum: 2%
  Value: 2%
  Idiosyncratic: 1%
```

## Best Practices

### 1. Start Conservative

```sig
// Start
max_position = 0.02
gross_exposure = 1.5

// Increase over time as confidence grows
max_position = 0.03
gross_exposure = 2.0
```

### 2. Multiple Defense Layers

```sig
// Layer 1: Position limits
cap = 0.03

// Layer 2: Sector limits
max_sector = 0.20

// Layer 3: Volatility targeting
max_volatility = 0.12

// Layer 4: Drawdown circuit breaker
max_daily_loss = 0.03
```

### 3. Stress Testing

Test against historical crises:
- 2008 Financial Crisis
- 2020 COVID Crash
- 2022 Rate Hikes

### 4. Know Your Limits

Define maximum acceptable:
- Daily loss: 3%
- Monthly loss: 10%
- Maximum drawdown: 20%

## Exercises

1. Add position and sector constraints to a strategy
2. Implement volatility targeting
3. Create a drawdown-based scaling mechanism
4. Monitor factor exposures

## Next Chapter

Continue to [Chapter 7: Going to Production](07-production.md) to learn about deploying live trading systems.
