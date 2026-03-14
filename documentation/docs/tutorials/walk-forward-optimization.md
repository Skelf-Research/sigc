# Tutorial: Walk-Forward Optimization

Build robust strategies using proper out-of-sample validation.

## Overview

Walk-forward optimization prevents overfitting by:

- Training on historical data
- Testing on unseen future data
- Rolling forward through time
- Simulating real trading conditions

## The Problem with Simple Backtesting

### Overfitting Risk

```
Simple Backtest:
├── Train on 2015-2024 ──────────────────────────┐
│   Find "optimal" parameters                     │
├── Test on... same 2015-2024 data?              │
│   ❌ Already seen this data!                   │
└── Result: Overfitted, won't work live          │
```

### Walk-Forward Solution

```
Walk-Forward:
├── Train: 2015-2019 → Optimize params
│   Test:  2020      → Out-of-sample
├── Train: 2016-2020 → Re-optimize
│   Test:  2021      → Out-of-sample
├── Train: 2017-2021 → Re-optimize
│   Test:  2022      → Out-of-sample
├── Train: 2018-2022 → Re-optimize
│   Test:  2023      → Out-of-sample
└── Combined test results = True performance
```

## Basic Walk-Forward

### Simple Configuration

```sig
data:
  source = "prices.parquet"
  format = parquet

signal momentum:
  emit zscore(ret(prices, lookback))

params:
  lookback: range(20, 120, 20)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)

  backtest walk_forward(
    train_years = 5,
    test_years = 1
  ) from 2015-01-01 to 2024-12-31
```

## Walk-Forward Configuration

### Parameters Explained

```sig
backtest walk_forward(
  train_years = 5,     // Years to train on
  test_years = 1,      // Years to test (out-of-sample)
  step_years = 1,      // How far to move each iteration
  warmup_years = 0.5,  // Data before train start for warmup
  metric = "sharpe"    // Optimization metric
) from 2010-01-01 to 2024-12-31
```

### Timeline Example

```
Total: 2010-01-01 to 2024-12-31

Iteration 1:
  Warmup: 2009-07-01 to 2009-12-31  (6 months)
  Train:  2010-01-01 to 2014-12-31  (5 years)
  Test:   2015-01-01 to 2015-12-31  (1 year)

Iteration 2:
  Warmup: 2010-07-01 to 2010-12-31
  Train:  2011-01-01 to 2015-12-31
  Test:   2016-01-01 to 2016-12-31

... continues ...

Iteration 10:
  Warmup: 2018-07-01 to 2018-12-31
  Train:  2019-01-01 to 2023-12-31
  Test:   2024-01-01 to 2024-12-31
```

## Multi-Parameter Optimization

### Optimizing Multiple Parameters

```sig
params:
  lookback: range(20, 120, 20)      // 6 values
  top_pct: range(0.10, 0.30, 0.05)  // 5 values
  rebal_freq: [5, 10, 21]           // 3 values
  // Total: 6 × 5 × 3 = 90 combinations

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio optimized:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)

  backtest walk_forward(
    train_years = 5,
    test_years = 1
  ) rebal=rebal_freq from 2010-01-01 to 2024-12-31
```

## Optimization Metrics

### Available Metrics

```sig
// Sharpe ratio (default)
backtest walk_forward(metric = "sharpe") ...

// Other options
backtest walk_forward(metric = "sortino") ...     // Downside risk
backtest walk_forward(metric = "calmar") ...      // Return / max drawdown
backtest walk_forward(metric = "return") ...      // Total return
backtest walk_forward(metric = "risk_adjusted") ... // Custom combination
```

### Custom Metric

```sig
params:
  lookback: range(20, 100, 20)

optimization:
  metric: custom
  formula: sharpe - 0.5 * max_drawdown - 0.1 * turnover

portfolio optimized:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(train_years=5, test_years=1) from 2010-01-01 to 2024-12-31
```

## Anchored vs Rolling Walk-Forward

### Anchored (Expanding Window)

```sig
backtest walk_forward(
  mode = "anchored",   // Training window expands
  initial_train_years = 5,
  test_years = 1
) from 2010-01-01 to 2024-12-31
```

```
Iteration 1: Train 2010-2014, Test 2015
Iteration 2: Train 2010-2015, Test 2016  (train expands)
Iteration 3: Train 2010-2016, Test 2017  (train expands)
```

### Rolling (Fixed Window)

```sig
backtest walk_forward(
  mode = "rolling",    // Training window stays fixed
  train_years = 5,
  test_years = 1
) from 2010-01-01 to 2024-12-31
```

```
Iteration 1: Train 2010-2014, Test 2015
Iteration 2: Train 2011-2015, Test 2016  (window rolls)
Iteration 3: Train 2012-2016, Test 2017  (window rolls)
```

## Complete Walk-Forward Strategy

### Production Example

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// ============ PARAMETERS TO OPTIMIZE ============

params:
  // Momentum parameters
  mom_lookback: [40, 60, 80, 100]
  mom_skip: [0, 21]

  // Value parameters
  value_weight: range(0.2, 0.5, 0.1)

  // Portfolio parameters
  top_bottom: [0.15, 0.20, 0.25]
  position_cap: [0.02, 0.03, 0.04]

// ============ SIGNALS ============

signal momentum:
  ret_full = ret(prices, mom_lookback)
  ret_skip = where(mom_skip > 0, ret(prices, mom_skip), 0)
  emit neutralize(zscore(ret_full - ret_skip), by=sectors)

signal value:
  emit neutralize(zscore(book_to_market), by=sectors)

signal quality:
  emit neutralize(zscore(roe), by=sectors)

signal combined:
  mom_weight = 1 - value_weight - 0.2  // Quality gets 0.2
  emit mom_weight * momentum + value_weight * value + 0.2 * quality

// ============ PORTFOLIO ============

portfolio multi_factor:
  weights = rank(combined).long_short(
    top = top_bottom,
    bottom = top_bottom,
    cap = position_cap
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest walk_forward(
    train_years = 5,
    test_years = 1,
    step_years = 1,
    metric = "sharpe",
    parallel = true
  ) rebal=21 from 2010-01-01 to 2024-12-31
```

## Running Walk-Forward

### Execute

```bash
sigc run strategy.sig --walk-forward
```

### Output

```
Walk-Forward Optimization Results
=================================

Configuration:
  Train Window: 5 years
  Test Window: 1 year
  Step: 1 year
  Total Iterations: 10

Parameter Selection by Period:
  2015: lookback=60, top_pct=0.20, cap=0.03
  2016: lookback=80, top_pct=0.20, cap=0.03
  2017: lookback=60, top_pct=0.15, cap=0.02
  2018: lookback=60, top_pct=0.20, cap=0.03
  ...

In-Sample Performance (Training):
  Avg Sharpe: 1.42
  Std Sharpe: 0.31

Out-of-Sample Performance (Testing):
  Sharpe: 0.68
  Return: 6.2% annual
  Volatility: 9.1%
  Max Drawdown: -14.2%

Degradation: 52% (in-sample to out-of-sample)
  (Lower is better, <50% is good)
```

## Analyzing Results

### Stability Analysis

```bash
sigc run strategy.sig --walk-forward --stability-analysis
```

```
Parameter Stability Analysis
============================

lookback:
  Most Selected: 60 (6/10 periods)
  Range: 40-80
  Stable: ✓

top_pct:
  Most Selected: 0.20 (7/10 periods)
  Range: 0.15-0.25
  Stable: ✓

position_cap:
  Most Selected: 0.03 (5/10 periods)
  Range: 0.02-0.04
  Moderately Stable: ~

Recommendation: Parameters are reasonably stable.
Consider fixing lookback=60, top_pct=0.20.
```

### Performance by Period

```bash
sigc run strategy.sig --walk-forward --period-analysis
```

```
Out-of-Sample Performance by Test Period
========================================

Period     Sharpe  Return  MaxDD   Params
2015       0.82    +8.1%   -6.2%   lb=60, top=0.20
2016       0.45    +4.2%   -9.8%   lb=80, top=0.20
2017       1.21   +12.3%   -4.1%   lb=60, top=0.15
2018      -0.32    -3.1%  -18.5%   lb=60, top=0.20
2019       1.05   +10.1%   -5.3%   lb=60, top=0.20
2020       0.78    +9.8%  -14.2%   lb=40, top=0.25
2021       0.92   +10.5%   -7.1%   lb=60, top=0.20
2022      -0.15    -1.8%  -12.3%   lb=80, top=0.20
2023       0.88    +8.3%   -5.8%   lb=60, top=0.20
2024       0.61    +5.9%   -8.1%   lb=60, top=0.20

Average    0.63    +6.4%   -9.1%
Std        0.48     5.1%    4.2%
```

## Best Practices

### 1. Sufficient Training Data

```sig
// Minimum 3-5 years of training data
backtest walk_forward(
  train_years = 5,  // At least 5 years
  test_years = 1
) ...
```

### 2. Avoid Over-Parameterization

```sig
// Bad: Too many parameters
params:
  p1: range(1, 100, 1)    // 100 values
  p2: range(1, 100, 1)    // 100 values
  p3: range(1, 100, 1)    // 100 values
  // 1,000,000 combinations - massive overfitting risk!

// Good: Focused parameter space
params:
  lookback: [40, 60, 80]  // 3 values
  weight: [0.3, 0.4, 0.5] // 3 values
  // 9 combinations - manageable
```

### 3. Check Degradation

```
In-Sample Sharpe: 1.5
Out-of-Sample Sharpe: 0.7

Degradation: 53%

Interpretation:
  <30%: Excellent, robust strategy
  30-50%: Good, some overfitting
  50-70%: Moderate overfitting
  >70%: Severe overfitting - simplify!
```

### 4. Examine Parameter Stability

If optimal parameters change dramatically each period, the strategy may be data-mined.

### 5. Use Parallel Processing

```sig
backtest walk_forward(
  train_years = 5,
  test_years = 1,
  parallel = true  // Use all CPU cores
) ...
```

## Common Pitfalls

### 1. Look-Ahead Bias

```sig
// Wrong: Using future data
signal bad:
  future_vol = rolling_std(ret(prices, 1), 60)  // Uses future 60 days!
  emit -future_vol

// Correct: Only past data
signal good:
  past_vol = lag(rolling_std(ret(prices, 1), 60), 1)
  emit -past_vol
```

### 2. Survivorship Bias

Ensure your data includes delisted stocks.

### 3. Too Short Test Periods

```sig
// Bad: 1 month test
backtest walk_forward(
  train_years = 5,
  test_months = 1  // Too short!
) ...

// Better: At least 1 year
backtest walk_forward(
  train_years = 5,
  test_years = 1
) ...
```

### 4. Ignoring Transaction Costs

```sig
// Always include realistic costs
portfolio main:
  weights = ...
  costs = tc.bps(10)  // Include transaction costs
  backtest walk_forward(...) ...
```

## Advanced: Custom Walk-Forward

### Regime-Conditional Optimization

```sig
// Different parameters for different regimes
params:
  bull_lookback: [40, 60]
  bear_lookback: [80, 100]

signal regime:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  emit where(ma_50 > ma_200, 1, 0)

signal adaptive_momentum:
  bull = regime > 0.5
  lb = where(bull, bull_lookback, bear_lookback)
  emit zscore(ret(prices, lb))

portfolio adaptive:
  weights = rank(adaptive_momentum).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(train_years=5, test_years=1) ...
```

## Next Steps

- [Production Deployment](production-deployment.md) - Deploy validated strategies
- [Python Workflow](python-workflow.md) - Advanced analysis in Python
- [Risk Models](../advanced/risk-models.md) - Risk management
