# Signal Section

The `signal` section defines computed signals that score assets.

## Syntax

```sig
signal <name>:
  <variable> = <expression>
  ...
  emit <expression>
```

## Basic Structure

Every signal block has:

1. A name
2. Zero or more variable assignments
3. One `emit` statement (required)

```sig
signal momentum:
  returns = ret(prices, 20)
  normalized = zscore(returns)
  emit normalized
```

## Variable Assignment

Create intermediate variables to break down complex computations:

```sig
signal step_by_step:
  // Step 1: Compute raw returns
  returns = ret(prices, 20)

  // Step 2: Adjust for volatility
  vol = rolling_std(ret(prices, 1), 60)
  vol_adj = returns / vol

  // Step 3: Normalize cross-sectionally
  normalized = zscore(vol_adj)

  // Step 4: Clean outliers
  cleaned = winsor(normalized, p=0.01)

  emit cleaned
```

### Variable Scope

Variables are scoped to their signal block:

```sig
signal signal_a:
  x = ret(prices, 20)  // x only exists in signal_a
  emit x

signal signal_b:
  x = ret(prices, 60)  // Different x, independent
  emit x
```

### Referencing Previous Variables

Later variables can reference earlier ones:

```sig
signal chained:
  step1 = ret(prices, 20)
  step2 = zscore(step1)        // Uses step1
  step3 = winsor(step2, 0.01)  // Uses step2
  emit step3
```

## The `emit` Statement

The `emit` statement specifies the signal's output:

```sig
signal example:
  computation = zscore(ret(prices, 20))
  emit computation  // This is the output
```

### Emit Must Be Last

The `emit` statement must be the last statement:

```sig
// CORRECT
signal good:
  x = ret(prices, 20)
  emit zscore(x)

// ERROR: emit not last
signal bad:
  emit zscore(x)
  x = ret(prices, 20)
```

### Inline Emit

For simple signals, emit directly:

```sig
signal concise:
  emit zscore(ret(prices, 20))
```

## Multiple Signals

Define multiple signals in one file:

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal reversal:
  emit -zscore(ret(prices, 5))

signal volatility:
  vol = rolling_std(ret(prices, 1), 20)
  emit -zscore(vol)  // Low vol is good
```

### Referencing Other Signals

Signals can reference other signals:

```sig
signal base_momentum:
  emit zscore(ret(prices, 60))

signal adjusted_momentum:
  // Uses base_momentum signal
  vol = rolling_std(ret(prices, 1), 60)
  emit base_momentum / vol
```

### Combining Signals

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal reversal:
  emit -zscore(ret(prices, 5))

signal combined:
  // Reference other signals
  mom = momentum
  rev = reversal
  emit 0.7 * mom + 0.3 * rev
```

## Using Functions

Call custom functions in signals:

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

fn sharpe_ratio(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window)

signal vol_adjusted:
  returns = ret(prices, 20)
  vol = volatility(prices, 60)
  emit zscore(returns / vol)
```

## Using Macros

Invoke macros for reusable patterns:

```sig
macro momentum_signal(px: expr, lookback: number = 20):
  let r = ret(px, lookback)
  let vol = rolling_std(ret(px, 1), 60)
  emit zscore(r / vol)

signal my_momentum:
  emit momentum_signal(prices, 30)
```

## Common Patterns

### Momentum

```sig
signal momentum:
  // 12-1 month momentum (skip recent month)
  total_return = ret(prices, 252)
  recent_return = ret(prices, 21)
  raw_mom = total_return - recent_return
  emit zscore(raw_mom)
```

### Mean Reversion

```sig
signal mean_reversion:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  z_score = (prices - ma) / std
  emit -zscore(z_score)  // Negative: fade extremes
```

### Value

```sig
signal value:
  // Book-to-market ratio
  bm = book_value / prices
  emit zscore(bm)
```

### Quality

```sig
signal quality:
  roe_z = zscore(roe)
  leverage_z = -zscore(debt_to_equity)
  emit (roe_z + leverage_z) / 2
```

### Volatility (Low Vol)

```sig
signal low_volatility:
  vol = rolling_std(ret(prices, 1), 60)
  emit -zscore(vol)  // Lower vol is better
```

### Technical (RSI)

```sig
signal rsi_signal:
  rsi_value = rsi(prices, 14)
  centered = rsi_value - 50
  emit -zscore(centered)  // Contrarian: buy oversold
```

## Signal Processing

### Normalization

Always normalize for cross-sectional comparability:

```sig
signal normalized:
  raw = ret(prices, 20)
  emit zscore(raw)  // Mean=0, Std=1
```

### Outlier Handling

Clip extreme values:

```sig
signal cleaned:
  raw = zscore(ret(prices, 20))
  emit winsor(raw, p=0.01)  // Clip at 1st/99th percentile
```

### Volatility Adjustment

Prevent high-vol stocks from dominating:

```sig
signal vol_adjusted:
  returns = ret(prices, 20)
  vol = rolling_std(ret(prices, 1), 60)
  emit zscore(returns / vol)
```

### Sector Neutralization

Remove sector bias:

```sig
signal sector_neutral:
  raw = zscore(ret(prices, 20))
  emit neutralize(raw, by=sectors)
```

### Missing Data Handling

```sig
signal with_fillna:
  raw = ret(prices, 20)
  filled = fill_nan(raw, 0)
  emit zscore(filled)
```

## Best Practices

### 1. Use Descriptive Names

```sig
// Good
signal vol_adjusted_momentum:
  ...

// Avoid
signal s1:
  ...
```

### 2. Break Down Complex Logic

```sig
// Good: Clear steps
signal clear:
  step1_returns = ret(prices, 20)
  step2_vol_adj = step1_returns / vol
  step3_normalized = zscore(step2_vol_adj)
  emit winsor(step3_normalized, 0.01)

// Avoid: One giant expression
signal unclear:
  emit winsor(zscore(ret(prices, 20) / rolling_std(ret(prices, 1), 60)), 0.01)
```

### 3. Add Comments

```sig
signal documented:
  // Compute 20-day momentum
  returns = ret(prices, 20)

  // Adjust for 60-day volatility
  vol = rolling_std(ret(prices, 1), 60)
  vol_adj = returns / vol

  // Cross-sectional standardization
  emit zscore(vol_adj)
```

### 4. Always Normalize Output

```sig
signal always_normalize:
  raw = some_computation(data)
  emit zscore(raw)  // Don't forget!
```

### 5. Handle Edge Cases

```sig
signal robust:
  raw = ret(prices, 20)

  // Handle missing data
  filled = fill_nan(raw, 0)

  // Handle outliers
  normalized = zscore(filled)
  cleaned = winsor(normalized, 0.01)

  emit cleaned
```

## Next Steps

- [Portfolio Section](portfolio-section.md) - Convert signals to portfolios
- [Operators](../operators/index.md) - Available operators
- [Macros](macros.md) - Reusable signal patterns
