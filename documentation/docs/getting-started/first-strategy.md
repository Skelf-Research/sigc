# Your First Strategy

This tutorial walks you through building a complete momentum strategy from scratch. You'll learn the key concepts and best practices along the way.

!!! tip "Time Required"
    About 15-20 minutes to complete.

## Overview

We'll build a **volatility-adjusted momentum strategy** that:

1. Computes 20-day returns for each asset
2. Adjusts for volatility (so high-vol stocks don't dominate)
3. Normalizes scores cross-sectionally
4. Goes long top performers, short bottom performers
5. Includes transaction cost modeling

## Prerequisites

- [sigc installed](installation.md)
- Sample price data (we'll create it)

## Step 1: Set Up Your Project

Create a project directory:

```bash
mkdir momentum-strategy && cd momentum-strategy
mkdir data
```

### Create Price Data

Create `data/prices.csv` with at least 60 days of data for 10 stocks:

```bash
# Download sample data from the docs
curl -o data/prices.csv https://docs.skelfresearch.com/sigc/assets/sample-data/prices.csv
```

Or create your own CSV with columns: `date,AAPL,MSFT,GOOGL,...`

## Step 2: Basic Signal Structure

Create `strategy.sig` with the basic structure:

```sig
// Volatility-Adjusted Momentum Strategy
// Author: Your Name
// Date: 2024-01-01

data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal momentum:
  returns = ret(prices, lookback)
  emit returns

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-02 to 2024-02-29
```

Test it:

```bash
sigc compile strategy.sig
sigc run strategy.sig
```

!!! note "Understanding the Output"
    At this point, you should see backtest results. Don't worry if the Sharpe ratio is low - we'll improve it.

## Step 3: Add Volatility Adjustment

Raw momentum favors high-volatility stocks. Let's normalize by volatility:

```sig
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20
  vol_lookback = 60

signal momentum:
  // Compute returns
  returns = ret(prices, lookback)

  // Compute volatility
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)

  // Volatility-adjusted returns
  vol_adj = returns / volatility

  emit vol_adj

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-02 to 2024-02-29
```

Run again and compare:

```bash
sigc run strategy.sig
```

## Step 4: Cross-Sectional Normalization

Add z-score normalization to standardize scores:

```sig
signal momentum:
  // Compute returns
  returns = ret(prices, lookback)

  // Compute volatility
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)

  // Volatility-adjusted returns
  vol_adj = returns / volatility

  // Cross-sectional normalization
  normalized = zscore(vol_adj)

  emit normalized
```

The `zscore` function:

1. Subtracts the cross-sectional mean
2. Divides by the cross-sectional standard deviation
3. Results in scores with mean=0, std=1

## Step 5: Handle Outliers

Extreme values can distort portfolio weights. Add winsorization:

```sig
signal momentum:
  // Compute returns
  returns = ret(prices, lookback)

  // Compute volatility
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)

  // Volatility-adjusted returns
  vol_adj = returns / volatility

  // Cross-sectional normalization
  normalized = zscore(vol_adj)

  // Clip outliers at 1st and 99th percentile
  cleaned = winsor(normalized, p=0.01)

  emit cleaned
```

## Step 6: Skip Recent Returns

Academic research shows that very recent returns tend to reverse (short-term reversal effect). Skip the most recent days:

```sig
params:
  lookback = 20
  skip_days = 5
  vol_lookback = 60

signal momentum:
  // Total return over lookback period
  total_return = ret(prices, lookback)

  // Return in skip period (to subtract)
  recent_return = ret(prices, skip_days)

  // Momentum = total - recent (skip reversal)
  raw_momentum = total_return - recent_return

  // Volatility adjustment
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)
  vol_adj = raw_momentum / volatility

  // Normalize and clean
  normalized = zscore(vol_adj)
  cleaned = winsor(normalized, p=0.01)

  emit cleaned
```

## Step 7: Add Position Limits

Limit individual position sizes to manage concentration risk:

```sig
portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.1)
  backtest from 2024-01-02 to 2024-02-29
```

The `cap=0.1` limits any single position to 10% of the portfolio.

## Complete Strategy

Here's the full strategy:

```sig
// Volatility-Adjusted Momentum Strategy
// Computes 20-day returns, adjusts for volatility, skips recent days
// Goes long top 20%, short bottom 20% with 10% position cap

data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20
  skip_days = 5
  vol_lookback = 60
  winsor_pct = 0.01
  top_pct = 0.2
  bottom_pct = 0.2
  max_weight = 0.1

signal momentum:
  // Total return over lookback period
  total_return = ret(prices, lookback)

  // Return in skip period (to subtract)
  recent_return = ret(prices, skip_days)

  // Momentum = total - recent (skip reversal)
  raw_momentum = total_return - recent_return

  // Volatility adjustment
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)
  vol_adj = raw_momentum / volatility

  // Normalize and clean
  normalized = zscore(vol_adj)
  cleaned = winsor(normalized, winsor_pct)

  emit cleaned

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=bottom_pct, cap=max_weight)
  backtest from 2024-01-02 to 2024-02-29
```

## Step 8: Run and Analyze

```bash
# Run backtest
sigc run strategy.sig

# Export detailed results
sigc run strategy.sig --output results.json

# View IR structure
sigc explain strategy.sig
```

### Interpret Results

```
=== Backtest Results ===
Total Return:         12.45%
Annualized Return:    98.76%
Sharpe Ratio:          2.15
Max Drawdown:          4.32%
Turnover:            280.00%
```

| Metric | What It Means | Good Range |
|--------|---------------|------------|
| Sharpe Ratio | Risk-adjusted return | > 1.0 |
| Max Drawdown | Worst decline | < 15% |
| Turnover | Trading frequency | < 500% |

## Step 9: Parameter Sensitivity

Test different parameter values:

=== "Lookback = 10"

    ```sig
    params:
      lookback = 10
      # ... rest same
    ```

=== "Lookback = 20"

    ```sig
    params:
      lookback = 20
      # ... rest same
    ```

=== "Lookback = 40"

    ```sig
    params:
      lookback = 40
      # ... rest same
    ```

Compare results:

```bash
sigc diff strategy_10d.sig strategy_20d.sig
```

## What You Learned

In this tutorial, you learned how to:

- [x] Structure a `.sig` file with data, params, signal, and portfolio blocks
- [x] Compute returns and volatility
- [x] Apply cross-sectional normalization with `zscore`
- [x] Handle outliers with `winsor`
- [x] Skip recent returns to avoid reversal
- [x] Set position limits with `cap`
- [x] Use parameters for easy tuning

## Best Practices

1. **Comment your code** - Explain the "why", not the "what"
2. **Use parameters** - Make tunable values configurable
3. **Normalize signals** - Use `zscore` for cross-sectional comparability
4. **Handle outliers** - Use `winsor` to clip extremes
5. **Start simple** - Add complexity gradually

## Next Steps

- [IDE Setup](ide-setup.md) - Get VS Code configured for sigc
- [DSL Basics](../language/syntax.md) - Deep dive into the language
- [Operators Reference](../operators/index.md) - Explore all 120+ operators
- [Tutorials](../tutorials/index.md) - More strategy tutorials
- [Strategy Library](../strategies/index.md) - Study complete strategies
