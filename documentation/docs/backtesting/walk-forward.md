# Walk-Forward Testing

Validate strategies with proper out-of-sample testing.

## The Problem with Simple Backtesting

Simple backtests can overfit:

```
1. Test many parameter combinations
2. Pick the "best" one
3. Deploy → Poor real-world performance
```

This is **data snooping** - using future information to select parameters.

## Walk-Forward Validation

Walk-forward testing simulates real trading conditions:

```
┌──────────────────────────────────────────────────────────────┐
│ Window 1:                                                    │
│ [====== Train ======][== Test ==]                            │
│ 2010-2014             2015-2016                              │
├──────────────────────────────────────────────────────────────┤
│ Window 2:                                                    │
│      [====== Train ======][== Test ==]                       │
│      2012-2016             2017-2018                         │
├──────────────────────────────────────────────────────────────┤
│ Window 3:                                                    │
│           [====== Train ======][== Test ==]                  │
│           2014-2018             2019-2020                    │
├──────────────────────────────────────────────────────────────┤
│ Window 4:                                                    │
│                [====== Train ======][== Test ==]             │
│                2016-2020             2021-2022               │
└──────────────────────────────────────────────────────────────┘

Combined out-of-sample results: 2015-2022
```

## Basic Walk-Forward

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  backtest walk_forward(
    train_years = 5,
    test_years = 2,
    step_years = 2
  ) from 2010-01-01 to 2024-12-31
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `train_years` | Length of training window |
| `test_years` | Length of test window |
| `step_years` | How far to move between windows |

## Parameter Optimization

Optimize parameters in each training window:

```sig
params:
  lookback: range(20, 120, 20)  # 20, 40, 60, 80, 100, 120
  top_pct: range(0.1, 0.4, 0.1)  # 0.1, 0.2, 0.3, 0.4

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio optimized:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)

  backtest walk_forward(
    train_years = 5,
    test_years = 2,
    step_years = 2,
    optimize = maximize("sharpe")  # Optimize for Sharpe in train
  ) from 2010-01-01 to 2024-12-31
```

### Optimization Objectives

| Objective | Description |
|-----------|-------------|
| `sharpe` | Maximize Sharpe ratio |
| `return` | Maximize total return |
| `calmar` | Maximize Calmar ratio |
| `sortino` | Maximize Sortino ratio |
| `min_drawdown` | Minimize max drawdown |

## Walk-Forward Output

```
Walk-Forward Results:
====================

Window 1 (Train: 2010-2014, Test: 2015-2016):
  Optimal Params: lookback=60, top_pct=0.2
  Train Sharpe: 1.25
  Test Sharpe: 0.92

Window 2 (Train: 2012-2016, Test: 2017-2018):
  Optimal Params: lookback=60, top_pct=0.2
  Train Sharpe: 1.18
  Test Sharpe: 0.85

Window 3 (Train: 2014-2018, Test: 2019-2020):
  Optimal Params: lookback=80, top_pct=0.2
  Train Sharpe: 1.32
  Test Sharpe: 0.78

Window 4 (Train: 2016-2020, Test: 2021-2022):
  Optimal Params: lookback=60, top_pct=0.3
  Train Sharpe: 1.15
  Test Sharpe: 0.65

Combined Out-of-Sample (2015-2022):
  Sharpe Ratio: 0.80
  CAGR: 11.2%
  Max Drawdown: -22.5%

Degradation: 35% (avg train Sharpe vs avg test Sharpe)
```

## Anchored Walk-Forward

Training always starts from the same date:

```sig
portfolio anchored:
  backtest walk_forward(
    anchor_start = true,  # Always start from beginning
    test_years = 2,
    step_years = 2
  ) from 2010-01-01 to 2024-12-31
```

```
┌──────────────────────────────────────────────────────────────┐
│ Window 1:                                                    │
│ [====== Train ======][== Test ==]                            │
│ 2010-2014             2015-2016                              │
├──────────────────────────────────────────────────────────────┤
│ Window 2:                                                    │
│ [========== Train ==========][== Test ==]                    │
│ 2010-2016                      2017-2018                     │
├──────────────────────────────────────────────────────────────┤
│ Window 3:                                                    │
│ [================ Train ================][== Test ==]        │
│ 2010-2018                                  2019-2020         │
└──────────────────────────────────────────────────────────────┘
```

## Rolling Walk-Forward

Fixed-length training window that rolls forward:

```sig
portfolio rolling:
  backtest walk_forward(
    train_years = 3,
    test_years = 1,
    step_years = 1,
    rolling = true
  ) from 2010-01-01 to 2024-12-31
```

## Cross-Validation

K-fold cross-validation for robustness:

```sig
portfolio cross_val:
  backtest cross_validate(
    folds = 5,
    purge_days = 21  # Gap between train and test to prevent leakage
  ) from 2010-01-01 to 2024-12-31
```

### Purged Cross-Validation

The `purge_days` parameter prevents data leakage from autocorrelation:

```
[=== Train ===][purge][=== Test ===][purge][=== Train ===]
                 ↑                     ↑
              21-day gap           21-day gap
```

## Combinatorial Purged Cross-Validation

More sophisticated validation for overlapping data:

```sig
portfolio cpcv:
  backtest cpcv(
    n_splits = 5,
    n_test_groups = 2,
    purge_days = 21,
    embargo_days = 5
  ) from 2010-01-01 to 2024-12-31
```

## Interpreting Results

### Degradation Ratio

Ratio of out-of-sample to in-sample performance:

```
Degradation = (In-Sample Sharpe - Out-of-Sample Sharpe) / In-Sample Sharpe
```

| Degradation | Interpretation |
|-------------|----------------|
| < 20% | Good - strategy likely robust |
| 20-40% | Acceptable - some overfitting |
| 40-60% | Concerning - significant overfitting |
| > 60% | Poor - strategy likely overfit |

### Parameter Stability

Check if optimal parameters are stable:

```
Window 1: lookback=60, top=0.2
Window 2: lookback=60, top=0.2
Window 3: lookback=80, top=0.2  <- Small variation
Window 4: lookback=60, top=0.3  <- Some variation

Conclusion: Reasonably stable, lookback=60 is robust
```

Unstable parameters indicate overfitting:

```
Window 1: lookback=20, top=0.1
Window 2: lookback=120, top=0.4  <- Huge change!
Window 3: lookback=40, top=0.2
Window 4: lookback=100, top=0.3

Conclusion: Parameters not robust, likely overfit
```

## Best Practices

### 1. Use Sufficient History

```sig
// At least 5 years of training data
backtest walk_forward(
  train_years = 5,  # Not 1 or 2 years
  ...
)
```

### 2. Test Multiple Windows

```sig
// At least 3-4 out-of-sample windows
backtest walk_forward(
  train_years = 5,
  test_years = 2,
  step_years = 2
) from 2005-01-01 to 2024-12-31
// Creates 5+ windows
```

### 3. Don't Over-Optimize

```sig
params:
  // Too many parameters = overfitting
  lookback: range(5, 200, 5)    // 40 values - too many!
  top_pct: range(0.05, 0.5, 0.05)  // 10 values
  // Total: 400 combinations!

// Better: fewer, meaningful values
params:
  lookback: [20, 60, 120]  // 3 values
  top_pct: [0.1, 0.2, 0.3]  // 3 values
  // Total: 9 combinations
```

### 4. Include Transaction Costs

```sig
portfolio main:
  costs = tc.bps(10)  # Include in walk-forward
  backtest walk_forward(...) from ...
```

### 5. Use Purging for Financial Data

```sig
backtest cross_validate(
  folds = 5,
  purge_days = 21  # Important for return autocorrelation
)
```

## Example: Complete Walk-Forward Analysis

```sig
data:
  source = "prices.parquet"
  format = parquet

params:
  lookback: [40, 60, 80]
  top_pct: [0.15, 0.20, 0.25]

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio walk_forward_test:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct, cap=0.03)

  costs = tc.bps(10)

  backtest walk_forward(
    train_years = 5,
    test_years = 2,
    step_years = 2,
    optimize = maximize("sharpe"),
    min_sharpe = 0.5  # Require min in-sample Sharpe
  ) from 2005-01-01 to 2024-12-31
```

### Run Analysis

```bash
sigc run strategy.sig --walk-forward --report detailed
```

### Output

```
Walk-Forward Analysis Report
============================

Configuration:
  Train Period: 5 years
  Test Period: 2 years
  Step: 2 years
  Optimization: maximize Sharpe

Results by Window:
                    In-Sample              Out-of-Sample
Window | Period     | Params     | Sharpe | Period     | Sharpe
-------+------------+------------+--------+------------+-------
1      | 2005-2009  | L=60, T=0.2| 1.25   | 2010-2011  | 0.95
2      | 2007-2011  | L=60, T=0.2| 1.18   | 2012-2013  | 0.88
3      | 2009-2013  | L=60, T=0.2| 1.32   | 2014-2015  | 0.78
4      | 2011-2015  | L=80, T=0.2| 1.15   | 2016-2017  | 0.72
5      | 2013-2017  | L=60, T=0.2| 1.28   | 2018-2019  | 0.82
6      | 2015-2019  | L=60, T=0.2| 1.22   | 2020-2021  | 0.68
7      | 2017-2021  | L=60, T=0.2| 1.10   | 2022-2023  | 0.55

Parameter Stability:
  lookback: 60 chosen in 6/7 windows (85.7%)
  top_pct: 0.2 chosen in 7/7 windows (100%)

Combined Out-of-Sample (2010-2023):
  CAGR: 9.8%
  Sharpe Ratio: 0.77
  Max Drawdown: -24.5%
  Calmar Ratio: 0.40

Degradation Analysis:
  Avg In-Sample Sharpe: 1.21
  Avg Out-of-Sample Sharpe: 0.77
  Degradation: 36%

Conclusion: Moderate degradation, parameters stable.
            Strategy appears reasonably robust.
```

## Next Steps

- [Metrics](metrics.md) - Performance metrics details
- [Constraints](constraints.md) - Portfolio constraints
- [Cost Models](cost-models.md) - Transaction costs
