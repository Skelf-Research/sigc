# Tutorial: Building a Momentum Strategy

Learn to build, test, and refine a classic momentum factor from scratch.

## What You'll Learn

- Computing momentum signals
- Cross-sectional ranking
- Portfolio construction
- Parameter optimization
- Analyzing results

## Prerequisites

- Completed [Quickstart](../getting-started/quickstart.md)
- Sample data in `docs/examples/data/sample_prices.csv`

## Step 1: Understanding Momentum

Momentum is the tendency for recent winners to continue winning. We'll compute:
- 20-day trailing return (signal)
- Skip the most recent 5 days (avoid reversal)

```
momentum = ret(20) - ret(5)
```

## Step 2: Basic Signal

Create `momentum_v1.sig`:

```
data:
  prices: load csv from "docs/examples/data/sample_prices.csv"

params:
  lookback = 20
  skip = 5

signal momentum:
  # Total return over lookback period
  total_ret = ret(prices, lookback)

  # Short-term return to skip
  short_ret = ret(prices, skip)

  # Momentum = total - short (skip recent reversal)
  mom = total_ret - short_ret

  emit mom

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-05-31
```

Run it:
```bash
./target/release/sigc run momentum_v1.sig
```

## Step 3: Adding Z-Score Normalization

Raw momentum values vary in scale. Z-score normalizes cross-sectionally:

Update the signal:

```
signal momentum:
  total_ret = ret(prices, lookback)
  short_ret = ret(prices, skip)
  mom = total_ret - short_ret

  # Normalize cross-sectionally
  normalized = zscore(mom)

  emit normalized
```

## Step 4: Adding Winsorization

Extreme outliers can dominate. Winsorize at 1% tails:

```
signal momentum:
  total_ret = ret(prices, lookback)
  short_ret = ret(prices, skip)
  mom = total_ret - short_ret
  normalized = zscore(mom)

  # Clip extreme values
  cleaned = winsor(normalized, 0.01)

  emit cleaned
```

## Step 5: Full Strategy

Here's the complete, production-ready signal:

```
data:
  prices: load csv from "docs/examples/data/sample_prices.csv"

params:
  lookback = 20
  skip = 5
  winsor_pct = 0.01
  long_pct = 0.2
  short_pct = 0.2

signal momentum:
  # Compute momentum avoiding short-term reversal
  total_ret = ret(prices, lookback)
  short_ret = ret(prices, skip)
  mom = total_ret - short_ret

  # Normalize and clean
  normalized = zscore(mom)
  cleaned = winsor(normalized, winsor_pct)

  emit cleaned

portfolio main:
  weights = rank(momentum).long_short(top=long_pct, bottom=short_pct)
  backtest from 2024-01-01 to 2024-05-31
```

## Step 6: Analyzing Results

Run and examine the output:

```bash
./target/release/sigc run momentum_v1.sig --output momentum_results.json
```

Key metrics to evaluate:

| Metric | Good Range | What It Means |
|--------|------------|---------------|
| Sharpe | > 1.0 | Risk-adjusted return |
| Max Drawdown | < 15% | Worst peak-to-trough |
| Turnover | < 500% | Annual trading volume |

## Step 7: Parameter Optimization

Find the best lookback period using GridSearch:

```rust
use sig_runtime::{GridSearch, Runtime};

let mut grid = GridSearch::new();
grid.add_range("lookback", 10.0, 60.0, 10.0);
grid.add_range("skip", 1.0, 10.0, 3.0);

let mut runtime = Runtime::new();
let results = grid.optimize(&ir, &mut runtime, "sharpe")?;

println!("Best params: {:?}", results[0].parameters);
println!("Best Sharpe: {:.2}", results[0].sharpe_ratio);
```

## Step 8: Walk-Forward Validation

Test for overfitting with out-of-sample validation:

```rust
use sig_runtime::{WalkForward, WalkForwardConfig};

// 6 months total, 3 month train, 1 month test
let config = WalkForwardConfig::new(126, 63, 21);
let mut wf = WalkForward::new(config);
wf.add_range("lookback", 10.0, 40.0, 10.0);

let result = wf.run(&ir, &mut runtime)?;
println!("Efficiency ratio: {:.1}%", result.efficiency_ratio * 100.0);
```

An efficiency ratio > 50% suggests the strategy isn't overfit.

## Step 9: Adding Transaction Costs

Real trading has costs. Add realistic estimates:

```rust
use sig_runtime::{CostModel, ImpactModel};

let cost_model = CostModel::institutional()
    .with_commission(0.5)  // 0.5 bps
    .with_slippage(1.0)    // 1 bp
    .with_impact(ImpactModel::SquareRoot { coefficient: 0.05 });
```

## Common Improvements

### Volatility Adjustment

Scale by inverse volatility:

```
signal vol_adj_momentum:
  mom = ret(prices, lookback) - ret(prices, skip)
  vol = rolling_std(ret(prices, 1), 20)
  adjusted = mom / vol
  emit zscore(adjusted)
```

### Sector Neutralization

Remove sector biases:

```
signal neutral_momentum:
  mom = ret(prices, lookback) - ret(prices, skip)
  neutral = neutralize(mom, sector)
  emit zscore(neutral)
```

### Combining Lookbacks

Blend multiple horizons:

```
signal multi_momentum:
  mom_short = zscore(ret(prices, 10))
  mom_medium = zscore(ret(prices, 20))
  mom_long = zscore(ret(prices, 60))

  combined = 0.2 * mom_short + 0.5 * mom_medium + 0.3 * mom_long
  emit combined
```

## Summary

You've learned to:
1. Compute momentum with reversal skip
2. Normalize with z-score
3. Clean with winsorization
4. Construct long/short portfolios
5. Optimize parameters
6. Validate out-of-sample

## Next Steps

- [Mean Reversion Tutorial](mean-reversion.md) - Opposite strategy
- [Multi-Factor Tutorial](multi-factor.md) - Combine signals
- [Cost Models Guide](../advanced/cost-models.md) - Realistic simulation
