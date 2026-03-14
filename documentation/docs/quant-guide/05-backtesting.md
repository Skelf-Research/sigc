# Chapter 5: Backtesting

Validate strategies properly with historical simulation.

## What is Backtesting?

Backtesting simulates how a strategy would have performed historically:

```sig
portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

Output:
```
Sharpe: 0.85
Return: 8.2% annual
Max Drawdown: -15.3%
```

## Why Backtesting Matters

### Validates Ideas

Before risking real money:
- Does the signal work historically?
- How does it behave in different regimes?
- What's the worst-case scenario?

### Identifies Issues

- Implementation errors
- Data problems
- Unrealistic assumptions

## Backtesting Fundamentals

### The Backtest Loop

```
For each trading day:
  1. Observe today's data (NOT future)
  2. Compute signals
  3. Generate target weights
  4. Execute trades (with realistic costs)
  5. Mark-to-market
  6. Record performance
```

### Key Principles

1. **No look-ahead bias**: Only use data available at decision time
2. **Realistic execution**: Include transaction costs
3. **Survivorship-free data**: Include delisted stocks
4. **Point-in-time data**: Use data as it was known then

## Basic Backtesting

### Simple Backtest

```sig
signal momentum:
  emit zscore(ret(prices, 60))

portfolio simple:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

### With Rebalancing

```sig
portfolio monthly:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest rebal=21 from 2020-01-01 to 2024-12-31  // Rebalance monthly
```

### With Transaction Costs

```sig
portfolio realistic:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(10)  // 10 basis points round-trip
  backtest from 2020-01-01 to 2024-12-31
```

## Performance Metrics

### Return Metrics

| Metric | Description | Good Value |
|--------|-------------|------------|
| Total Return | Cumulative return | >50% |
| Annual Return | Annualized return | >5% |
| Excess Return | Return vs benchmark | >0% |

### Risk Metrics

| Metric | Description | Good Value |
|--------|-------------|------------|
| Volatility | Annualized std dev | <15% |
| Max Drawdown | Largest peak-to-trough | <20% |
| VaR (95%) | Value at Risk | <2% daily |

### Risk-Adjusted Metrics

| Metric | Formula | Good Value |
|--------|---------|------------|
| Sharpe Ratio | Return / Vol | >0.5 |
| Sortino Ratio | Return / Downside Vol | >1.0 |
| Calmar Ratio | Return / Max DD | >0.5 |
| Information Ratio | Active Return / TE | >0.5 |

### Interpreting Results

```
Backtest Results
================
Total Return: 85%
Annual Return: 8.2%
Annual Volatility: 9.6%
Sharpe Ratio: 0.85

Max Drawdown: -15.3%
Avg Drawdown: -4.2%
Drawdown Duration: 45 days (avg)

Turnover: 320% annual
Win Rate: 54%
```

## Common Pitfalls

### 1. Look-Ahead Bias

Using information not available at decision time:

```sig
// WRONG: Using future data
signal bad:
  future_return = ret(prices, -60)  // Future returns!
  emit future_return

// CORRECT: Only past data
signal good:
  past_return = ret(prices, 60)
  emit zscore(past_return)
```

### 2. Survivorship Bias

Only testing on stocks that still exist:

```
Full Universe (2015):      1000 stocks
Survivors (to 2024):        800 stocks
Survivorship bias: Testing only on 800 winners
```

Solution: Use survivorship-free data that includes delisted stocks.

### 3. Overfitting

Finding patterns that only exist in sample:

```sig
// High risk of overfitting
params:
  p1: range(1, 100, 1)
  p2: range(1, 100, 1)
  p3: range(1, 100, 1)
  // 1,000,000 combinations tested!
```

Signs of overfitting:
- Perfect in-sample, terrible out-of-sample
- Complex, unintuitive parameters
- Very different optimal params across periods

### 4. Ignoring Transaction Costs

```sig
// Unrealistic: No costs
portfolio no_costs:
  weights = ...
  backtest rebal=1  // Daily rebalancing, no costs
  // Result: 20% annual return! (unrealistic)

// Realistic: Include costs
portfolio with_costs:
  weights = ...
  costs = tc.bps(10)
  backtest rebal=1
  // Result: 2% annual return (costs destroy alpha)
```

### 5. Data Mining

Testing many hypotheses until one works:

```
Test 100 random signals
By chance, ~5 will be "significant" (p < 0.05)
These are false positives!
```

Solution: Multiple testing correction, out-of-sample validation.

## Walk-Forward Analysis

### The Right Way to Test

```sig
params:
  lookback: range(20, 100, 20)

portfolio validated:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)

  backtest walk_forward(
    train_years = 5,   // Optimize on 5 years
    test_years = 1     // Test on 1 year (unseen)
  ) from 2010-01-01 to 2024-12-31
```

### How It Works

```
Period 1:
  Train: 2010-2014 → Find optimal params
  Test:  2015      → Evaluate (out-of-sample)

Period 2:
  Train: 2011-2015 → Re-optimize
  Test:  2016      → Evaluate

...

Combined test results = True expected performance
```

### Interpreting Walk-Forward Results

```
Walk-Forward Results
====================
In-Sample Sharpe:  1.2
Out-of-Sample Sharpe: 0.65

Degradation: 46%
```

- **<30% degradation**: Robust strategy
- **30-50% degradation**: Some overfitting
- **>50% degradation**: Significant overfitting

## Robustness Checks

### 1. Sensitivity Analysis

```sig
params:
  lookback: [40, 50, 60, 70, 80]

// Run all parameter values
// Check: Are results similar across all values?
```

### 2. Subsample Testing

```sig
// Test on different time periods
backtest from 2010-01-01 to 2014-12-31  // Period 1
backtest from 2015-01-01 to 2019-12-31  // Period 2
backtest from 2020-01-01 to 2024-12-31  // Period 3

// Results should be consistent
```

### 3. Universe Variations

Test on different asset subsets:
- Large cap vs small cap
- Different sectors
- Different geographies

### 4. Transaction Cost Sensitivity

```sig
// Test different cost assumptions
costs = tc.bps(5)   // Optimistic
costs = tc.bps(10)  // Realistic
costs = tc.bps(20)  // Conservative
```

## Complete Backtesting Example

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Parameters for optimization
params:
  mom_lookback: [40, 60, 80]
  value_weight: [0.3, 0.4, 0.5]
  top_pct: [0.15, 0.20, 0.25]

// Signals
signal momentum:
  emit neutralize(zscore(ret(prices, mom_lookback)), by=sectors)

signal value:
  emit neutralize(zscore(book_to_market), by=sectors)

signal combined:
  mom_wt = 1 - value_weight
  emit mom_wt * momentum + value_weight * value

// Portfolio with constraints
portfolio main:
  weights = rank(combined).long_short(
    top = top_pct,
    bottom = top_pct,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  // Walk-forward validation
  backtest walk_forward(
    train_years = 5,
    test_years = 1,
    metric = "sharpe"
  ) rebal=21 from 2010-01-01 to 2024-12-31
```

Run:
```bash
sigc run strategy.sig --walk-forward --verbose
```

## Best Practices Checklist

- [ ] Use survivorship-free data
- [ ] Include realistic transaction costs
- [ ] Test out-of-sample (walk-forward)
- [ ] Check parameter stability
- [ ] Test across different time periods
- [ ] Verify no look-ahead bias
- [ ] Check for reasonable turnover
- [ ] Compare to benchmark
- [ ] Document assumptions

## Exercises

1. Backtest a simple momentum strategy with different lookback periods
2. Add transaction costs and measure the impact
3. Run walk-forward validation and measure degradation
4. Test strategy on different time periods

## Next Chapter

Continue to [Chapter 6: Risk Management](06-risk.md) to learn about controlling portfolio risk.
