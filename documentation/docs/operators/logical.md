# Logical Operators

Boolean logic operators for combining conditions.

## Operators

### `and`

Logical AND - true when both operands are true.

```sig
result = a and b
```

```sig
signal example:
  positive_momentum = returns > 0
  low_volatility = volatility < 0.3
  both = positive_momentum and low_volatility
  emit where(both, score, 0)
```

### `or`

Logical OR - true when either operand is true.

```sig
result = a or b
```

```sig
signal example:
  oversold = rsi < 30
  strong_momentum = momentum > 2
  either = oversold or strong_momentum
  emit where(either, 1, 0)
```

### `not(x)`

Logical NOT - inverts boolean value.

```sig
result = not(x)
```

```sig
signal example:
  excluded = status == "Excluded"
  included = not(excluded)
  emit where(included, score, 0)
```

### `where(condition, true_value, false_value)`

Conditional selection - returns `true_value` where condition is true, `false_value` otherwise.

```sig
result = where(condition, true_value, false_value)
```

```sig
signal example:
  // Replace NaN with 0
  cleaned = where(is_nan(x), 0, x)

  // Conditional scoring
  score = where(trend > 0, momentum, reversal)

  // Sign function
  sign = where(x > 0, 1, where(x < 0, -1, 0))

  emit cleaned
```

## Common Patterns

### Multiple Conditions

```sig
signal multi_condition:
  cond1 = returns > 0
  cond2 = volatility < 0.3
  cond3 = volume > avg_volume

  // All conditions must be true
  all_true = cond1 and cond2 and cond3

  // At least one condition must be true
  any_true = cond1 or cond2 or cond3

  emit where(all_true, score, 0)
```

### Nested Conditionals

```sig
signal nested:
  // Multiple categories
  category = where(score > 1, 3,
             where(score > 0, 2,
             where(score > -1, 1, 0)))
  emit category
```

### Conditional Scoring

```sig
signal conditional_score:
  uptrend = ma_fast > ma_slow
  downtrend = ma_fast < ma_slow

  // Different signal based on trend
  trend_signal = where(uptrend, momentum, where(downtrend, reversal, 0))
  emit trend_signal
```

### Filter and Score

```sig
signal filtered:
  // Define filter
  valid = not(is_nan(prices)) and volume > 0

  // Compute score
  raw_score = zscore(ret(prices, 20))

  // Apply filter
  filtered_score = where(valid, raw_score, 0)
  emit filtered_score
```

### Regime Detection

```sig
signal regime_aware:
  // Detect volatility regime
  current_vol = rolling_std(ret(prices, 1), 20)
  avg_vol = rolling_mean(current_vol, 252)
  high_vol = current_vol > avg_vol * 1.5

  // Different strategies for different regimes
  high_vol_signal = -zscore(ret(prices, 5))   // Mean reversion
  low_vol_signal = zscore(ret(prices, 60))    // Momentum

  signal = where(high_vol, high_vol_signal, low_vol_signal)
  emit signal
```

### Binary Indicator

```sig
signal indicator:
  // Golden cross indicator
  golden_cross = ema(prices, 50) > ema(prices, 200)

  // Convert to +1/-1
  indicator = where(golden_cross, 1, -1)
  emit indicator
```

### Missing Data Handling

```sig
signal handle_missing:
  // Check for missing
  has_data = not(is_nan(prices))

  // Fill or exclude
  filled = where(is_nan(prices), lag(prices, 1), prices)

  // Or set to zero
  zeroed = where(is_nan(score), 0, score)

  emit zeroed
```

### Exclusion List

```sig
signal exclude:
  // Exclude based on multiple criteria
  too_small = market_cap < 1e9
  too_illiquid = avg_volume < 100000
  excluded_sector = sector == "Utilities"

  excluded = too_small or too_illiquid or excluded_sector
  included = not(excluded)

  filtered_score = where(included, raw_score, 0)
  emit filtered_score
```

## Truth Tables

### `and`

| a | b | a and b |
|---|---|---------|
| T | T | T |
| T | F | F |
| F | T | F |
| F | F | F |

### `or`

| a | b | a or b |
|---|---|--------|
| T | T | T |
| T | F | T |
| F | T | T |
| F | F | F |

### `not`

| a | not(a) |
|---|--------|
| T | F |
| F | T |

## Short-Circuit Evaluation

`and` and `or` use short-circuit evaluation:

```sig
// If a is false, b is not evaluated
result = a and b

// If a is true, b is not evaluated
result = a or b
```

This is useful for avoiding errors:

```sig
// Safe: denominator checked before division
safe = denominator != 0 and (numerator / denominator > threshold)
```

## Precedence

From highest to lowest:

1. `not`
2. Comparison operators (`>`, `<`, `==`, etc.)
3. `and`
4. `or`

```sig
// These are equivalent:
a > b and c < d or e > f
((a > b) and (c < d)) or (e > f)

// Use parentheses for clarity
(a > b and c < d) or (e > f)
```

## Type Behavior

Logical operators work on boolean values:

```sig
// Comparison produces boolean
condition = prices > 100  // Boolean

// Logical operators combine booleans
combined = condition and another_condition  // Boolean

// where() uses boolean to select values
result = where(combined, value_a, value_b)  // Same type as value_a/b
```

## NaN Handling

- `NaN and x` → false
- `NaN or x` → depends on x
- `not(NaN)` → true (because NaN is falsy)

For explicit NaN handling:

```sig
is_valid = not(is_nan(x))
```

## Next Steps

- [Data Handling](data-handling.md) - Missing data operators
- [Comparison](comparison.md) - Create conditions
- [Time-Series](time-series.md) - Compute values
