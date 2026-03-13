# DSL Basics

Learn the sigc language syntax and structure.

## File Structure

A `.sig` file has four main blocks:

```
data:
  <data declarations>

params:
  <parameter declarations>

signal <name>:
  <computations>
  emit <output>

portfolio <name>:
  <portfolio construction>
  backtest from <start> to <end>
```

## Data Block

Declare data sources to load.

### CSV Files

```
data:
  prices: load csv from "path/to/prices.csv"
  volume: load csv from "data/volume.csv"
```

### Parquet Files

```
data:
  prices: load parquet from "data/prices.parquet"
```

### S3 Sources

```
data:
  prices: load parquet from "s3://bucket/prices.parquet"
```

### Multiple Sources

```
data:
  prices: load csv from "prices.csv"
  volume: load csv from "volume.csv"
  fundamentals: load parquet from "fundamentals.parquet"
```

## Params Block

Declare tunable parameters with default values.

```
params:
  lookback = 20
  threshold = 0.5
  decay = 0.94
```

Parameters are constants that can be optimized via GridSearch.

## Signal Block

Define computations that produce alpha signals.

### Basic Structure

```
signal <name>:
  <variable> = <expression>
  ...
  emit <variable>
```

### Variables

Assign intermediate results:

```
signal example:
  returns = ret(prices, 20)
  avg = rolling_mean(returns, 10)
  score = returns - avg
  emit score
```

### Expressions

Combine operators and variables:

```
signal complex:
  a = ret(prices, 20)
  b = rolling_std(prices, 20)
  c = a / b                    # Arithmetic
  d = zscore(c)                # Function call
  e = winsor(d, 0.01)          # With parameter
  f = e * 0.5 + lag(e, 1) * 0.5  # Combined
  emit f
```

### Multiple Signals

Define multiple signals in one file:

```
signal momentum:
  emit zscore(ret(prices, 20))

signal volatility:
  emit rolling_std(ret(prices, 1), 20)

signal combined:
  mom = zscore(ret(prices, 20))
  vol = rolling_std(ret(prices, 1), 20)
  emit mom / vol
```

## Portfolio Block

Construct portfolio weights and run backtest.

### Basic Portfolio

```
portfolio main:
  weights = rank(signal_name).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Weight Construction

```
portfolio main:
  # Long top 20%, short bottom 20%
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

The `long_short` function:
- Ranks assets cross-sectionally
- Assigns +1/n to top `top%` assets
- Assigns -1/n to bottom `bottom%` assets
- Creates dollar-neutral weights

## Expressions

### Literals

```
x = 42          # Integer
y = 3.14        # Float
z = -0.5        # Negative
```

### Arithmetic

```
a + b           # Addition
a - b           # Subtraction
a * b           # Multiplication
a / b           # Division
```

### Function Calls

```
abs(x)                    # Single argument
ret(prices, 20)           # With period
winsor(score, 0.01)       # With threshold
rolling_corr(a, b, 60)    # Multiple arguments
```

### Chaining

Operations can reference previous variables:

```
signal chain:
  step1 = ret(prices, 20)
  step2 = zscore(step1)
  step3 = winsor(step2, 0.01)
  emit step3
```

### Inline Expressions

Combine operations inline:

```
signal inline:
  emit winsor(zscore(ret(prices, 20)), 0.01)
```

## Comments

Use `#` for comments:

```
# This is a comment
signal example:
  # Compute 20-day return
  r = ret(prices, 20)

  # Z-score normalize
  z = zscore(r)  # inline comment

  emit z
```

## Best Practices

### Naming

```
# Good - descriptive names
signal momentum_20d:
  returns_20d = ret(prices, 20)
  score_normalized = zscore(returns_20d)
  emit score_normalized

# Bad - unclear names
signal s:
  x = ret(prices, 20)
  y = zscore(x)
  emit y
```

### Organization

```
# Step-by-step for readability
signal well_organized:
  # Step 1: Raw signal
  raw_momentum = ret(prices, lookback)

  # Step 2: Normalize
  normalized = zscore(raw_momentum)

  # Step 3: Clean outliers
  cleaned = winsor(normalized, 0.01)

  emit cleaned
```

### Parameters

Use params for values you might tune:

```
params:
  lookback = 20       # Will optimize this
  winsor_pct = 0.01   # Might adjust

signal tunable:
  r = ret(prices, lookback)
  z = zscore(r)
  emit winsor(z, winsor_pct)
```

## Complete Example

```
# Momentum strategy with volatility adjustment
# Author: Quant Team
# Date: 2024-01-01

data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"

params:
  mom_lookback = 20
  vol_lookback = 60
  skip_days = 5
  winsor_pct = 0.01
  long_pct = 0.2
  short_pct = 0.2

signal vol_adj_momentum:
  # Compute momentum avoiding short-term reversal
  total_return = ret(prices, mom_lookback)
  short_return = ret(prices, skip_days)
  raw_momentum = total_return - short_return

  # Volatility adjustment
  daily_ret = ret(prices, 1)
  volatility = rolling_std(daily_ret, vol_lookback)
  vol_adjusted = raw_momentum / volatility

  # Normalize and clean
  normalized = zscore(vol_adjusted)
  cleaned = winsor(normalized, winsor_pct)

  emit cleaned

portfolio main:
  weights = rank(vol_adj_momentum).long_short(top=long_pct, bottom=short_pct)
  backtest from 2024-01-01 to 2024-12-31
```

## Next Steps

- [Operators Reference](../reference/operators-table.md) - All available functions
- [Momentum Tutorial](../tutorials/momentum-strategy.md) - Build a real strategy
- [Data Loading Guide](data-loading.md) - Connect data sources
