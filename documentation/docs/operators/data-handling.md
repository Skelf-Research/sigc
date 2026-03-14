# Data Handling Operators

Operators for missing data and cumulative computations.

## Missing Data

### `is_nan(x)`

Check for NaN (Not a Number) values.

```sig
is_nan(x: Numeric) -> Boolean
```

```sig
signal example:
  missing = is_nan(prices)
  valid = not(is_nan(prices))
  emit where(valid, prices, 0)
```

### `fill_nan(x, value)`

Replace NaN values with a constant.

```sig
fill_nan(x: Numeric, value: Scalar) -> Numeric
```

```sig
signal example:
  // Fill missing prices with 0
  filled = fill_nan(prices, 0)

  // Fill missing returns with 0
  returns = ret(prices, 20)
  safe_returns = fill_nan(returns, 0)

  emit safe_returns
```

### `coalesce(a, b)`

Return first non-NaN value.

```sig
coalesce(a: Numeric, b: Numeric) -> Numeric
```

```sig
signal example:
  // Use primary source, fallback to secondary
  price = coalesce(primary_price, secondary_price)

  // Chain multiple fallbacks
  value = coalesce(estimate_a, coalesce(estimate_b, default_value))

  emit price
```

## Cumulative Operations

### `cumsum(x)`

Cumulative sum.

```sig
cumsum(x: Numeric) -> Numeric
```

```sig
signal example:
  // Cumulative returns
  cum_returns = cumsum(daily_returns)

  // Running total
  total_volume = cumsum(volume)

  emit cum_returns
```

### `cumprod(x)`

Cumulative product.

```sig
cumprod(x: Numeric) -> Numeric
```

```sig
signal example:
  // Cumulative wealth (starting at 1)
  growth_factors = 1 + daily_returns
  wealth = cumprod(growth_factors)

  emit wealth
```

### `cummax(x)`

Cumulative maximum (running max).

```sig
cummax(x: Numeric) -> Numeric
```

```sig
signal example:
  // Track all-time high
  all_time_high = cummax(prices)

  // Drawdown from peak
  drawdown = (prices - all_time_high) / all_time_high

  emit drawdown
```

### `cummin(x)`

Cumulative minimum (running min).

```sig
cummin(x: Numeric) -> Numeric
```

```sig
signal example:
  // Track all-time low
  all_time_low = cummin(prices)

  // Distance from trough
  recovery = (prices - all_time_low) / all_time_low

  emit recovery
```

## Common Patterns

### Handle Missing Returns

```sig
signal safe_returns:
  raw_returns = ret(prices, 20)
  // Fill missing returns with 0 (no change)
  safe = fill_nan(raw_returns, 0)
  emit zscore(safe)
```

### Forward Fill (with Lag)

```sig
signal forward_fill:
  // Use previous value if current is missing
  filled = where(is_nan(prices), lag(prices, 1), prices)
  emit filled
```

### Multiple Data Sources

```sig
signal multi_source:
  // Primary source preferred, fallback to secondary
  combined = coalesce(primary_data, secondary_data)

  // Triple fallback
  best_estimate = coalesce(estimate_a, coalesce(estimate_b, estimate_c))

  emit combined
```

### Drawdown Calculation

```sig
signal drawdown:
  // Peak wealth
  cum_return = cumsum(ret(prices, 1))
  peak = cummax(cum_return)

  // Current drawdown
  dd = cum_return - peak

  // Max drawdown (most negative)
  max_dd = cummin(dd)

  emit dd
```

### Wealth Index

```sig
signal wealth:
  // Daily returns
  daily_ret = ret(prices, 1)

  // Growth factor (1 + return)
  growth = 1 + daily_ret

  // Cumulative wealth (assumes $1 start)
  wealth = cumprod(growth)

  emit wealth
```

### High Water Mark

```sig
signal hwm:
  // Cumulative performance
  performance = cumsum(returns)

  // High water mark
  hwm = cummax(performance)

  // Distance from HWM
  below_hwm = performance - hwm

  emit below_hwm
```

### Data Quality Score

```sig
signal quality:
  // Count non-missing values
  has_price = where(is_nan(prices), 0, 1)
  has_volume = where(is_nan(volume), 0, 1)

  // Quality score
  quality = (has_price + has_volume) / 2

  emit quality
```

### Safe Division

```sig
signal safe_div:
  // Avoid division by zero
  ratio = where(denominator != 0, numerator / denominator, 0)

  // Or use coalesce
  safe_ratio = coalesce(numerator / denominator, 0)

  emit ratio
```

### Cumulative Indicator

```sig
signal trend_days:
  // Count consecutive up days
  up = where(ret(prices, 1) > 0, 1, 0)
  cum_up = cumsum(up)

  emit cum_up
```

## NaN Propagation

Most operators propagate NaN:

```sig
// If any input is NaN, output is NaN
NaN + 1 = NaN
zscore([1, NaN, 3]) = [z1, NaN, z3]
rolling_mean([1, NaN, 3], 2) = [NaN, NaN, NaN]  // Window contains NaN
```

Handle NaN explicitly:

```sig
signal robust:
  // Remove NaN before computation
  clean = fill_nan(raw_data, 0)
  result = zscore(clean)
  emit result
```

## Type Behavior

| Operator | Input | Output |
|----------|-------|--------|
| `is_nan` | Numeric | Boolean |
| `fill_nan` | Numeric, Scalar | Numeric |
| `coalesce` | Numeric, Numeric | Numeric |
| `cumsum` | Numeric | Numeric |
| `cumprod` | Numeric | Numeric |
| `cummax` | Numeric | Numeric |
| `cummin` | Numeric | Numeric |

## Best Practices

### 1. Handle Missing Data Early

```sig
signal clean_first:
  // Clean data at the start
  clean_prices = fill_nan(prices, 0)

  // Then compute
  returns = ret(clean_prices, 20)
  emit zscore(returns)
```

### 2. Use Coalesce for Fallbacks

```sig
signal with_fallback:
  // Prefer primary, use secondary if missing
  price = coalesce(bloomberg_price, yahoo_price)
  emit zscore(ret(price, 20))
```

### 3. Check for NaN in Conditions

```sig
signal safe_condition:
  has_data = not(is_nan(x))
  valid_and_positive = has_data and x > 0
  emit where(valid_and_positive, x, 0)
```

### 4. Document Missing Data Handling

```sig
// Missing prices are forward-filled
// Missing returns are set to 0
// Missing fundamentals are excluded
signal documented:
  ...
```

## Next Steps

- [Time-Series](time-series.md) - Rolling computations
- [Cross-Sectional](cross-sectional.md) - Normalization
- [Logical](logical.md) - Conditional logic
