# Backtesting

Simulate strategy performance on historical data.

## Overview

Backtesting in sigc:

1. **Loads historical data** from your data section
2. **Computes signals** at each time point
3. **Constructs portfolios** based on weights
4. **Simulates trading** with realistic costs
5. **Reports metrics** for analysis

## Quick Example

```sig
data:
  source = "prices.parquet"
  format = parquet

signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

Run the backtest:

```bash
sigc run strategy.sig
```

## Backtest Options

### Date Range

```sig
portfolio main:
  weights = ...
  backtest from 2020-01-01 to 2024-12-31
```

### Rebalancing Frequency

```sig
portfolio main:
  weights = ...
  // Rebalance every 21 trading days (~monthly)
  backtest rebal=21 from 2020-01-01 to 2024-12-31
```

Common frequencies:

| Setting | Frequency |
|---------|-----------|
| `rebal=1` | Daily |
| `rebal=5` | Weekly |
| `rebal=21` | Monthly |
| `rebal=63` | Quarterly |
| `rebal=252` | Annually |

### Benchmark

```sig
portfolio main:
  weights = ...
  backtest rebal=21 benchmark=SPY from 2020-01-01 to 2024-12-31
```

### Transaction Costs

```sig
portfolio main:
  weights = ...
  costs = tc.bps(5)  # 5 basis points
  backtest from 2020-01-01 to 2024-12-31
```

See [Cost Models](cost-models.md) for details.

## Backtest Output

### Console Output

```
Backtest Results: momentum_strategy
===================================
Period: 2020-01-01 to 2024-12-31
Rebalancing: 21 days
Benchmark: SPY

Performance Metrics:
  Total Return:     85.2%
  Annualized Return: 13.1%
  Volatility:       15.2%
  Sharpe Ratio:     0.86
  Max Drawdown:    -18.5%
  Calmar Ratio:     0.71

vs Benchmark (SPY):
  Alpha:            4.2%
  Beta:             0.72
  Information Ratio: 0.45
  Tracking Error:   9.3%

Risk Metrics:
  VaR (95%):       -2.1%
  CVaR (95%):      -3.2%
  Skewness:        -0.15
  Kurtosis:         3.8
```

### Detailed Report

```bash
sigc run strategy.sig --report detailed
```

Generates HTML report with:

- Cumulative return chart
- Drawdown chart
- Monthly returns heatmap
- Position concentration over time
- Turnover analysis
- Factor exposures

### Export Results

```bash
# CSV export
sigc run strategy.sig --output results.csv

# JSON export
sigc run strategy.sig --output results.json

# Parquet export
sigc run strategy.sig --output results.parquet
```

## Backtest Process

```
┌───────────────────────────────────────────────────────────┐
│                    For each rebalance date:               │
│                                                           │
│  1. Load data up to current date (no look-ahead)          │
│  2. Compute all signals                                   │
│  3. Calculate target weights                              │
│  4. Apply constraints (if any)                            │
│  5. Calculate trades needed                               │
│  6. Apply transaction costs                               │
│  7. Execute trades (update positions)                     │
│  8. Mark-to-market portfolio                              │
│  9. Record metrics                                        │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## Key Concepts

### Point-in-Time

sigc only uses data available at each historical date:

```sig
signal no_lookahead:
  // At 2020-01-15, only data up to 2020-01-15 is visible
  emit zscore(ret(prices, 60))
```

### Rebalancing

Trades only occur on rebalance dates:

```
Rebal=21 (monthly):
Day 1:  Compute weights → Trade
Day 2-20: Hold positions
Day 21: Compute weights → Trade
Day 22-41: Hold positions
...
```

### Transaction Costs

Applied when positions change:

```
Old Position: +5% AAPL
New Position: +3% AAPL
Trade: Sell 2% of portfolio
Cost: 2% × cost_rate
```

## Multiple Portfolios

Test variations:

```sig
signal momentum:
  emit zscore(ret(prices, 60))

portfolio monthly:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest rebal=21 from 2020-01-01 to 2024-12-31

portfolio weekly:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest rebal=5 from 2020-01-01 to 2024-12-31

portfolio capped:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.03)
  backtest rebal=21 from 2020-01-01 to 2024-12-31
```

## Performance Metrics

Key metrics computed automatically:

| Metric | Description |
|--------|-------------|
| Total Return | Cumulative return over period |
| CAGR | Compound annual growth rate |
| Volatility | Annualized standard deviation |
| Sharpe Ratio | Risk-adjusted return |
| Max Drawdown | Largest peak-to-trough decline |
| Calmar Ratio | CAGR / Max Drawdown |

See [Metrics](metrics.md) for complete list.

## Best Practices

### 1. Use Realistic Costs

```sig
// Include trading costs
costs = tc.bps(10) + slippage.model("square-root", coef=0.1)
```

### 2. Match Rebalancing to Signal

```sig
// Fast signal (mean reversion)
backtest rebal=5 ...  // Weekly

// Slow signal (value, momentum)
backtest rebal=21 ...  // Monthly
```

### 3. Test Multiple Periods

```sig
// Out-of-sample testing
portfolio in_sample:
  backtest from 2010-01-01 to 2019-12-31

portfolio out_of_sample:
  backtest from 2020-01-01 to 2024-12-31
```

### 4. Include Benchmark

```sig
// Compare to market
backtest benchmark=SPY from ...
```

### 5. Check Turnover

High turnover erodes returns:

```bash
sigc run strategy.sig --report turnover
```

## Common Pitfalls

### Look-Ahead Bias

Using future information:

```sig
// BAD: Using same-day close to trade at close
signal bad:
  emit zscore(prices)  // Can't use today's price to trade today

// GOOD: Use previous day
signal good:
  emit zscore(lag(prices, 1))
```

### Survivorship Bias

Only testing on current stocks:

```sig
// Use point-in-time data that includes delisted stocks
data:
  source = "pit_prices.parquet"  // Point-in-time
```

### Overfitting

Testing too many variations:

```
Problem: Testing 1000 parameter combinations
         Finding "best" parameters
         Parameters don't work out-of-sample
```

Solution: Use walk-forward validation. See [Walk-Forward](walk-forward.md).

## Documentation Index

- [Metrics](metrics.md) - Performance and risk metrics
- [Cost Models](cost-models.md) - Transaction cost modeling
- [Walk-Forward](walk-forward.md) - Out-of-sample testing
- [Constraints](constraints.md) - Position and risk constraints
- [Benchmark Analysis](benchmark-analysis.md) - Benchmark comparison
- [Attribution](attribution.md) - Return attribution

## Next Steps

- [Metrics](metrics.md) - Understanding performance metrics
- [Cost Models](cost-models.md) - Modeling trading costs
- [Walk-Forward](walk-forward.md) - Proper validation
