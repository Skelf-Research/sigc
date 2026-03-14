# Arithmetic Operators

Mathematical operations for signal computation.

## Binary Operators

### Addition (`+`)

```sig
result = a + b
```

Element-wise addition.

```sig
signal example:
  adjusted = prices + 1
  combined = signal_a + signal_b
  emit combined
```

### Subtraction (`-`)

```sig
result = a - b
```

Element-wise subtraction.

```sig
signal example:
  change = prices - lag(prices, 1)
  spread = signal_a - signal_b
  emit spread
```

### Multiplication (`*`)

```sig
result = a * b
```

Element-wise multiplication.

```sig
signal example:
  scaled = returns * 252        // Annualize
  weighted = signal * weight
  emit scaled
```

### Division (`/`)

```sig
result = a / b
```

Element-wise division.

```sig
signal example:
  ratio = prices / lag(prices, 20)
  vol_adj = returns / volatility
  emit vol_adj
```

## Unary Operators

### Negation (`-`)

```sig
result = -x
```

Negate values.

```sig
signal example:
  inverted = -returns
  contrarian = -zscore(returns)  // Buy losers, sell winners
  emit contrarian
```

## Functions

### `abs(x)`

Absolute value.

```sig
abs(x: Numeric) -> Numeric
```

```sig
signal example:
  absolute_return = abs(returns)
  distance = abs(prices - moving_avg)
  emit distance
```

### `sign(x)`

Sign of value (-1, 0, or 1).

```sig
sign(x: Numeric) -> Numeric
```

```sig
signal example:
  direction = sign(returns)
  // direction is -1 for negative, 0 for zero, 1 for positive
  emit direction
```

### `log(x)`

Natural logarithm.

```sig
log(x: Numeric) -> Numeric
```

```sig
signal example:
  log_prices = log(prices)
  log_return = log(prices / lag(prices, 1))
  log_market_cap = log(market_cap)
  emit log_return
```

!!! warning
    `log(x)` is undefined for x <= 0. Ensure positive values.

### `exp(x)`

Exponential function (e^x).

```sig
exp(x: Numeric) -> Numeric
```

```sig
signal example:
  growth = exp(log_return)
  cumulative = exp(cumsum(log_returns))
  emit growth
```

### `pow(x, n)`

Raise to power.

```sig
pow(x: Numeric, n: Numeric) -> Numeric
```

```sig
signal example:
  squared = pow(returns, 2)
  variance = rolling_mean(squared, 20)
  cubed = pow(x, 3)
  emit variance
```

### `sqrt(x)`

Square root.

```sig
sqrt(x: Numeric) -> Numeric
```

```sig
signal example:
  std_dev = sqrt(variance)
  annualized_vol = sqrt(252) * daily_vol
  emit std_dev
```

!!! warning
    `sqrt(x)` is undefined for x < 0. Ensure non-negative values.

### `floor(x)`

Round down to nearest integer.

```sig
floor(x: Numeric) -> Numeric
```

```sig
signal example:
  buckets = floor(score * 10)  // 0-9 buckets
  emit buckets
```

### `ceil(x)`

Round up to nearest integer.

```sig
ceil(x: Numeric) -> Numeric
```

```sig
signal example:
  min_position = ceil(score)
  emit min_position
```

### `round(x)`

Round to nearest integer.

```sig
round(x: Numeric) -> Numeric
```

```sig
signal example:
  discrete = round(score)
  emit discrete
```

### `min(a, b)`

Element-wise minimum.

```sig
min(a: Numeric, b: Numeric) -> Numeric
```

```sig
signal example:
  capped = min(score, 2)        // Cap at 2
  lower = min(signal_a, signal_b)
  emit capped
```

### `max(a, b)`

Element-wise maximum.

```sig
max(a: Numeric, b: Numeric) -> Numeric
```

```sig
signal example:
  floored = max(score, -2)      // Floor at -2
  higher = max(signal_a, signal_b)
  emit floored
```

### `clip(x, lo, hi)`

Clamp values to range [lo, hi].

```sig
clip(x: Numeric, lo: Numeric, hi: Numeric) -> Numeric
```

```sig
signal example:
  bounded = clip(score, -3, 3)  // Clip to [-3, 3]
  weights = clip(raw_weights, -0.1, 0.1)  // Max 10% position
  emit bounded
```

Equivalent to:

```sig
clip(x, lo, hi) = max(min(x, hi), lo)
```

## Common Patterns

### Log Returns

```sig
signal log_returns:
  log_ret = log(prices / lag(prices, 1))
  emit log_ret
```

### Volatility Scaling

```sig
signal vol_scaled:
  target_vol = 0.15  // 15% annualized
  current_vol = sqrt(252) * rolling_std(ret(prices, 1), 20)
  scale_factor = target_vol / current_vol
  emit returns * scale_factor
```

### Distance from Mean

```sig
signal distance:
  ma = rolling_mean(prices, 20)
  dist = abs(prices - ma)
  normalized_dist = dist / ma
  emit normalized_dist
```

### Bounded Scores

```sig
signal bounded:
  raw = zscore(returns)
  bounded = clip(raw, -3, 3)  // Clip extremes
  emit bounded
```

### Annualization

```sig
signal annualized:
  daily_ret = ret(prices, 1)
  annual_ret = daily_ret * 252
  daily_vol = rolling_std(daily_ret, 20)
  annual_vol = daily_vol * sqrt(252)
  sharpe = annual_ret / annual_vol
  emit sharpe
```

## Type Behavior

### Scalar Operations

```sig
x + 1        // Adds 1 to every element
x * 2        // Multiplies every element by 2
x / 100      // Divides every element by 100
```

### Element-wise Operations

```sig
a + b        // Adds corresponding elements
a * b        // Multiplies corresponding elements
```

### Broadcasting

Scalars broadcast to match array dimensions:

```sig
prices + 1           // 1 broadcasts to all elements
returns * 252        // 252 broadcasts
```

## Precedence

From highest to lowest:

1. Function calls: `abs(x)`, `sqrt(x)`
2. Unary negation: `-x`
3. Multiplication/Division: `*`, `/`
4. Addition/Subtraction: `+`, `-`

Use parentheses to clarify:

```sig
a + b * c      // = a + (b * c)
(a + b) * c    // = (a + b) * c
```

## Next Steps

- [Comparison Operators](comparison.md)
- [Time-Series Operators](time-series.md)
- [Cross-Sectional Operators](cross-sectional.md)
