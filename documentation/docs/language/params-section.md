# Params Section

The `params:` section defines tunable parameters for your strategy.

## Syntax

```sig
params:
  <name> = <default_value>
```

## Basic Usage

```sig
params:
  lookback = 20
  threshold = 0.5
  top_pct = 0.2

signal example:
  returns = ret(prices, lookback)  // Uses lookback parameter
  score = zscore(returns)
  filtered = where(score > threshold, score, 0)
  emit filtered

portfolio main:
  weights = rank(example).long_short(top=top_pct, bottom=top_pct)
  backtest from 2024-01-01 to 2024-12-31
```

## Parameter Types

Parameters can be integers or floats:

```sig
params:
  // Integers
  lookback = 20
  skip_days = 5
  rebalance_freq = 21

  // Floats
  threshold = 0.5
  decay_rate = 0.94
  winsor_pct = 0.01
  commission_bps = 5.0
```

## Using Parameters

### In Expressions

```sig
params:
  lookback = 20

signal example:
  returns = ret(prices, lookback)
  emit zscore(returns)
```

### In Function Calls

```sig
params:
  vol_window = 60
  winsor_pct = 0.01

signal example:
  vol = rolling_std(ret(prices, 1), vol_window)
  score = zscore(ret(prices, 20) / vol)
  emit winsor(score, p=winsor_pct)
```

### In Portfolio Construction

```sig
params:
  top_pct = 0.2
  bottom_pct = 0.2
  max_weight = 0.05

portfolio main:
  weights = rank(signal).long_short(
    top=top_pct,
    bottom=bottom_pct,
    cap=max_weight
  )
  backtest from 2024-01-01 to 2024-12-31
```

### In Functions and Macros

```sig
params:
  default_vol_window = 60

fn volatility(x, window=default_vol_window):
  rolling_std(ret(x, 1), window)
```

!!! note
    Parameters used as function defaults must be defined before the function.

## Parameter Naming

### Conventions

Use descriptive, snake_case names:

```sig
// Good
params:
  momentum_lookback = 60
  vol_estimation_window = 252
  winsorization_percentile = 0.01
  position_cap = 0.05

// Avoid
params:
  n = 60      // Too short
  window1 = 252  // Not descriptive
  p = 0.01    // Unclear
```

### Grouped Parameters

Organize related parameters together:

```sig
params:
  // Momentum parameters
  mom_lookback = 60
  mom_skip = 5

  // Volatility parameters
  vol_window = 252
  vol_target = 0.15

  // Portfolio parameters
  top_pct = 0.2
  bottom_pct = 0.2
  max_weight = 0.05

  // Cost parameters
  commission_bps = 5.0
  impact_coef = 0.1
```

## Parameter Optimization

Parameters can be optimized using grid search in Python:

```python
import pysigc

template = """
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = {lookback}

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
"""

# Grid search
results = []
for lookback in [10, 20, 30, 40, 60]:
    source = template.format(lookback=lookback)
    result = pysigc.backtest(source)
    results.append({
        'lookback': lookback,
        'sharpe': result.sharpe_ratio,
        'return': result.total_return
    })

# Find best
best = max(results, key=lambda x: x['sharpe'])
print(f"Best lookback: {best['lookback']}, Sharpe: {best['sharpe']:.2f}")
```

## Walk-Forward Optimization

For robust parameter selection:

```rust
use sig_runtime::{WalkForward, WalkForwardConfig};

let config = WalkForwardConfig::new(252, 126, 21);
let mut wf = WalkForward::new(config);

// Define parameter ranges
wf.add_range("lookback", 10.0, 60.0, 10.0);
wf.add_range("vol_window", 20.0, 120.0, 20.0);

let result = wf.run(&ir, &mut runtime)?;
println!("Best params: {:?}", result.best_params);
println!("Efficiency: {:.1}%", result.efficiency_ratio * 100.0);
```

## Best Practices

### 1. Use Meaningful Defaults

Choose defaults that work across market conditions:

```sig
params:
  lookback = 20       // Reasonable for many strategies
  vol_window = 60     // Standard volatility estimation
  winsor_pct = 0.01   // Common outlier threshold
```

### 2. Document Parameter Purpose

Add comments explaining each parameter:

```sig
params:
  lookback = 20       // Days to compute returns
  skip_days = 5       // Days to skip for reversal
  vol_window = 60     // Window for volatility estimation
  winsor_pct = 0.01   // Percentile for winsorization
```

### 3. Avoid Over-Parameterization

Too many parameters leads to overfitting:

```sig
// Too many parameters - suspicious
params:
  lookback = 17
  skip = 3
  vol_window = 47
  winsor_pct = 0.023
  decay = 0.937
  threshold = 0.178
```

### 4. Use Round Numbers

Prefer simple, interpretable values:

```sig
// Good
params:
  lookback = 20
  vol_window = 60

// Suspicious (likely overfit)
params:
  lookback = 17
  vol_window = 53
```

### 5. Test Sensitivity

Parameters should be robust to small changes:

```bash
# Test multiple lookback values
sigc run strategy_10d.sig
sigc run strategy_20d.sig
sigc run strategy_30d.sig

# Results should be similar
```

## Examples

### Momentum Strategy

```sig
params:
  // Return calculation
  lookback = 252        // 1-year lookback
  skip_days = 21        // Skip recent month

  // Volatility
  vol_window = 60       // Vol estimation window

  // Cleaning
  winsor_pct = 0.01     // Clip at 1st/99th percentile

  // Portfolio
  top_pct = 0.2         // Long top 20%
  bottom_pct = 0.2      // Short bottom 20%
  max_weight = 0.05     // Max 5% per position

signal momentum:
  // 12-1 month momentum, skip recent
  total = ret(prices, lookback)
  recent = ret(prices, skip_days)
  mom = total - recent

  // Volatility adjustment
  vol = rolling_std(ret(prices, 1), vol_window)
  vol_adj = mom / vol

  // Clean and normalize
  emit winsor(zscore(vol_adj), winsor_pct)

portfolio main:
  weights = rank(momentum).long_short(
    top=top_pct,
    bottom=bottom_pct,
    cap=max_weight
  )
  backtest from 2020-01-01 to 2024-12-31
```

### Multi-Factor Strategy

```sig
params:
  // Factor lookbacks
  mom_lookback = 60
  rev_lookback = 5
  vol_lookback = 20

  // Factor weights
  mom_weight = 0.5
  rev_weight = 0.3
  vol_weight = 0.2

  // Portfolio
  top_pct = 0.2
  bottom_pct = 0.2

signal momentum:
  emit zscore(ret(prices, mom_lookback))

signal reversal:
  emit -zscore(ret(prices, rev_lookback))

signal low_vol:
  vol = rolling_std(ret(prices, 1), vol_lookback)
  emit -zscore(vol)

signal combined:
  emit mom_weight * momentum + rev_weight * reversal + vol_weight * low_vol

portfolio main:
  weights = rank(combined).long_short(top=top_pct, bottom=bottom_pct)
  backtest from 2020-01-01 to 2024-12-31
```

## Next Steps

- [Signal Section](signal-section.md) - Computing signals
- [Walk-Forward](../backtesting/walk-forward.md) - Parameter optimization
- [Tutorials](../tutorials/index.md) - Complete examples
