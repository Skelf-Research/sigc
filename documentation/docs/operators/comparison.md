# Comparison Operators

Relational operators that produce boolean results.

## Operators

### Greater Than (`>`)

```sig
result = a > b
```

True where `a` is greater than `b`.

```sig
signal example:
  above_ma = prices > rolling_mean(prices, 20)
  positive = returns > 0
  emit above_ma
```

### Less Than (`<`)

```sig
result = a < b
```

True where `a` is less than `b`.

```sig
signal example:
  below_ma = prices < rolling_mean(prices, 20)
  negative = returns < 0
  oversold = rsi < 30
  emit below_ma
```

### Greater or Equal (`>=`)

```sig
result = a >= b
```

True where `a` is greater than or equal to `b`.

```sig
signal example:
  at_or_above = prices >= threshold
  included = score >= cutoff
  emit at_or_above
```

### Less or Equal (`<=`)

```sig
result = a <= b
```

True where `a` is less than or equal to `b`.

```sig
signal example:
  at_or_below = prices <= threshold
  low_vol = volatility <= 0.2
  emit at_or_below
```

### Equal (`==`)

```sig
result = a == b
```

True where `a` equals `b`.

```sig
signal example:
  is_tech = sector == "Technology"
  at_target = price == target_price
  emit is_tech
```

### Not Equal (`!=`)

```sig
result = a != b
```

True where `a` does not equal `b`.

```sig
signal example:
  not_excluded = status != "Excluded"
  active = volume != 0
  emit not_excluded
```

## Using Comparisons

### Filtering

```sig
signal filtered:
  raw_score = zscore(ret(prices, 20))
  // Only keep positive scores
  filtered = where(raw_score > 0, raw_score, 0)
  emit filtered
```

### Conditional Logic

```sig
signal conditional:
  ma_20 = rolling_mean(prices, 20)
  ma_50 = rolling_mean(prices, 50)

  // Trend direction
  uptrend = ma_20 > ma_50
  downtrend = ma_20 < ma_50

  // Signal based on trend
  signal = where(uptrend, 1, where(downtrend, -1, 0))
  emit signal
```

### Multiple Conditions

```sig
signal multi_condition:
  positive_momentum = returns > 0
  low_volatility = volatility < 0.3
  not_excluded = status != "Excluded"

  // Combine conditions
  valid = positive_momentum and low_volatility and not_excluded
  score = where(valid, raw_signal, 0)
  emit score
```

### Range Checks

```sig
signal in_range:
  // Check if score is in [-2, 2]
  in_range = score >= -2 and score <= 2
  bounded = where(in_range, score, 0)
  emit bounded
```

## Common Patterns

### Trend Detection

```sig
signal trend:
  fast_ma = ema(prices, 10)
  slow_ma = ema(prices, 50)

  uptrend = fast_ma > slow_ma
  downtrend = fast_ma < slow_ma

  // +1 for uptrend, -1 for downtrend
  trend_signal = where(uptrend, 1, where(downtrend, -1, 0))
  emit trend_signal
```

### Breakout Detection

```sig
signal breakout:
  high_20 = rolling_max(prices, 20)
  low_20 = rolling_min(prices, 20)

  breakout_up = prices > high_20
  breakout_down = prices < low_20

  signal = where(breakout_up, 1, where(breakout_down, -1, 0))
  emit signal
```

### RSI Signals

```sig
signal rsi_signal:
  rsi_val = rsi(prices, 14)

  oversold = rsi_val < 30
  overbought = rsi_val > 70

  // Contrarian: buy oversold, sell overbought
  signal = where(oversold, 1, where(overbought, -1, 0))
  emit signal
```

### Volume Filter

```sig
signal volume_filtered:
  avg_volume = rolling_mean(volume, 20)
  sufficient_volume = volume > avg_volume * 0.5

  raw_signal = zscore(ret(prices, 20))
  filtered = where(sufficient_volume, raw_signal, 0)
  emit filtered
```

### Quality Filter

```sig
signal quality_filter:
  // Only include stocks with positive ROE
  quality_filter = roe > 0

  momentum = zscore(ret(prices, 60))
  filtered = where(quality_filter, momentum, 0)
  emit filtered
```

## Comparison with Scalar

```sig
signal scalar_comparison:
  // Compare with constant
  above_zero = returns > 0
  below_threshold = volatility < 0.3
  equals_one = bucket == 1
  emit above_zero
```

## Comparison with Series

```sig
signal series_comparison:
  // Compare two series element-wise
  outperforms = stock_return > market_return
  above_ma = prices > rolling_mean(prices, 20)
  emit outperforms
```

## Chaining Comparisons

Use `and`/`or` to chain comparisons:

```sig
signal chained:
  // NOT: a < x < b (doesn't work)
  // USE: x > a and x < b
  in_range = score > -2 and score < 2
  emit where(in_range, score, 0)
```

## NaN Handling

Comparisons with NaN return false:

```sig
signal nan_handling:
  // NaN > 0 is false
  // NaN == NaN is false

  // Check for NaN explicitly
  is_valid = not(is_nan(x))
  valid_positive = is_valid and x > 0
  emit valid_positive
```

## Type Coercion

Comparison results are boolean (true/false):

```sig
signal bool_result:
  condition = prices > 100  // Boolean
  // Use in where() or logical operations
  result = where(condition, 1, 0)
  emit result
```

## Precedence

Comparison operators have lower precedence than arithmetic:

```sig
a + b > c    // = (a + b) > c
a > b + c    // = a > (b + c)
```

But higher than logical operators:

```sig
a > b and c < d    // = (a > b) and (c < d)
```

## Next Steps

- [Logical Operators](logical.md) - Combine conditions
- [Data Handling](data-handling.md) - Handle missing data
- [Time-Series](time-series.md) - Compute values to compare
