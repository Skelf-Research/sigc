# Expressions

Expressions compute values from data, operators, and functions.

## Literals

### Numbers

```sig
42          // Integer
-5          // Negative integer
3.14        // Float
-0.5        // Negative float
1e-3        // Scientific: 0.001
2.5e10      // Scientific: 25000000000
```

### Strings

```sig
"data/prices.csv"
"s3://bucket/file.parquet"
```

## Identifiers

Reference data, parameters, variables, and signals:

```sig
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal example:
  returns = ret(prices, lookback)  // prices and lookback are identifiers
  emit returns                      // returns is an identifier
```

## Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `x + y` |
| `-` | Subtraction | `x - y` |
| `*` | Multiplication | `x * y` |
| `/` | Division | `x / y` |
| `-` (unary) | Negation | `-x` |

### Examples

```sig
signal arithmetic:
  a = prices + 1
  b = prices - lag(prices, 1)
  c = returns * 252
  d = returns / volatility
  e = -returns  // Negate
  emit a
```

### Mixed Operations

```sig
signal mixed:
  // Parentheses control order
  a = (prices + 1) * 2
  b = prices + 1 * 2      // Same as: prices + 2
  c = (a - b) / (a + b)
  emit c
```

## Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `>` | Greater than | `x > y` |
| `<` | Less than | `x < y` |
| `>=` | Greater or equal | `x >= y` |
| `<=` | Less or equal | `x <= y` |
| `==` | Equal | `x == y` |
| `!=` | Not equal | `x != y` |

### Examples

```sig
signal comparisons:
  above_ma = prices > rolling_mean(prices, 20)
  positive_return = returns > 0
  not_excluded = status != excluded
  in_range = score >= -2 and score <= 2
  emit above_ma
```

## Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `and` | Logical AND | `a and b` |
| `or` | Logical OR | `a or b` |
| `not` | Logical NOT | `not(a)` |

### Examples

```sig
signal logical:
  condition1 = returns > 0
  condition2 = volatility < 0.3
  combined = condition1 and condition2
  either = condition1 or condition2
  inverted = not(condition1)
  emit combined
```

### Short-Circuit Evaluation

`and` and `or` use short-circuit evaluation:

```sig
// If condition1 is false, condition2 is not evaluated
result = condition1 and condition2

// If condition1 is true, condition2 is not evaluated
result = condition1 or condition2
```

## Conditional Expression

The `where` function provides conditional logic:

```sig
where(condition, true_value, false_value)
```

### Examples

```sig
signal conditional:
  // Replace NaN with 0
  filled = where(is_nan(prices), 0, prices)

  // Sign of returns
  sign = where(returns > 0, 1, where(returns < 0, -1, 0))

  // Conditional scaling
  scaled = where(volatility > 0.3, returns / 2, returns)

  emit filled
```

### Nested `where`

```sig
signal nested:
  // Multiple conditions
  category = where(score > 1, 2,
             where(score > 0, 1,
             where(score > -1, 0, -1)))
  emit category
```

## Function Calls

Call built-in operators and custom functions:

```sig
// Single argument
normalized = zscore(returns)
absolute = abs(returns)

// Multiple arguments
returns = ret(prices, 20)
mean = rolling_mean(prices, 20)
correlation = rolling_corr(x, y, 60)

// Named arguments (where supported)
cleaned = winsor(score, p=0.01)
weights = long_short(top=0.2, bottom=0.2)
```

### Chaining

Some functions support method-style chaining:

```sig
weights = rank(signal).long_short(top=0.2, bottom=0.2)
```

## Operator Precedence

From highest to lowest precedence:

1. **Function calls**: `f(x)`, `x.method()`
2. **Unary**: `-x`, `not(x)`
3. **Multiplicative**: `*`, `/`
4. **Additive**: `+`, `-`
5. **Comparison**: `>`, `<`, `>=`, `<=`, `==`, `!=`
6. **Logical AND**: `and`
7. **Logical OR**: `or`

### Examples

```sig
// These are equivalent:
a + b * c        // a + (b * c)
a + (b * c)

// These are different:
(a + b) * c      // (a + b) * c

// Comparisons before logical:
a > b and c < d  // (a > b) and (c < d)

// Parentheses for clarity:
(a > b) and (c < d)
```

## Expression Types

Expressions have inferred types:

| Expression | Type |
|------------|------|
| `42` | Int64 |
| `3.14` | Float64 |
| `"string"` | String |
| `prices` | Panel<Float64> |
| `prices > 100` | Panel<Bool> |
| `zscore(x)` | Panel<Float64> |

### Type Coercion

Numeric types are automatically coerced:

```sig
signal coercion:
  x = 5        // Int64
  y = 3.14     // Float64
  z = x + y    // Float64 (coerced)
  emit z
```

## Common Expression Patterns

### Returns

```sig
// Simple return
ret(prices, 20)

// Log return
log(prices / lag(prices, 20))

// Skip recent
ret(prices, 60) - ret(prices, 5)
```

### Normalization

```sig
// Z-score
zscore(x)

// Rank
rank(x)

// Scale to sum to 1
scale(abs(x))
```

### Volatility Adjustment

```sig
// Divide by volatility
x / rolling_std(ret(prices, 1), 60)

// Target volatility
x * (target_vol / current_vol)
```

### Cleaning

```sig
// Winsorize
winsor(x, p=0.01)

// Fill missing
fill_nan(x, 0)

// Coalesce
coalesce(a, b)
```

### Technical

```sig
// Moving average crossover
ema(prices, 10) - ema(prices, 50)

// RSI centered
rsi(prices, 14) - 50

// Bollinger position
(prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)
```

## Complex Expressions

### Multi-Step Computation

```sig
signal complex:
  // Step by step for clarity
  raw_returns = ret(prices, 60)
  skip_returns = ret(prices, 5)
  momentum = raw_returns - skip_returns
  volatility = rolling_std(ret(prices, 1), 252)
  vol_adjusted = momentum / volatility
  normalized = zscore(vol_adjusted)
  cleaned = winsor(normalized, p=0.01)
  emit cleaned
```

### Inline (Compact)

```sig
signal compact:
  emit winsor(zscore((ret(prices, 60) - ret(prices, 5)) / rolling_std(ret(prices, 1), 252)), p=0.01)
```

### Recommended: Balance

```sig
signal balanced:
  // Group related computations
  momentum = ret(prices, 60) - ret(prices, 5)
  vol_adj = momentum / rolling_std(ret(prices, 1), 252)
  emit winsor(zscore(vol_adj), p=0.01)
```

## Best Practices

### 1. Use Parentheses for Clarity

```sig
// Clear
result = (a + b) * (c - d)

// Ambiguous
result = a + b * c - d
```

### 2. Break Down Complex Expressions

```sig
// Good: Clear steps
signal clear:
  momentum = ret(prices, 60)
  volatility = rolling_std(ret(prices, 1), 60)
  vol_adj = momentum / volatility
  emit zscore(vol_adj)

// Avoid: One giant expression
signal unclear:
  emit zscore(ret(prices, 60) / rolling_std(ret(prices, 1), 60))
```

### 3. Name Meaningful Intermediates

```sig
// Good: Descriptive names
risk_adjusted_return = returns / volatility
sector_neutral_score = neutralize(score, by=sector)

// Avoid: Generic names
x = returns / volatility
y = neutralize(score, by=sector)
```

### 4. Handle Edge Cases

```sig
signal robust:
  // Handle missing data
  filled = fill_nan(prices, 0)

  // Handle division by zero
  safe_ratio = where(denominator != 0, numerator / denominator, 0)

  // Handle outliers
  cleaned = winsor(score, p=0.01)

  emit cleaned
```

## Next Steps

- [Operators](../operators/index.md) - Available operators
- [Functions](functions.md) - Custom functions
- [Signal Section](signal-section.md) - Using expressions in signals
