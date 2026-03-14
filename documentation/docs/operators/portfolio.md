# Portfolio Operators

Operators for constructing portfolio weights.

## Long-Short Portfolio

### `long_short(top, bottom, cap)`

Create dollar-neutral long-short weights.

```sig
rank(signal).long_short(top: Scalar, bottom: Scalar, cap: Scalar = None) -> Panel
```

**Parameters:**

- `top`: Fraction of assets to long (e.g., 0.2 = top 20%)
- `bottom`: Fraction of assets to short (e.g., 0.2 = bottom 20%)
- `cap`: Maximum position size (optional)

```sig
portfolio main:
  // Long top 20%, short bottom 20%
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Weight Distribution

With `long_short(top=0.2, bottom=0.2)` and 100 assets:

| Position | Count | Individual Weight | Total |
|----------|-------|-------------------|-------|
| Long | 20 | +5% | +100% |
| Neutral | 60 | 0% | 0% |
| Short | 20 | -5% | -100% |
| **Net** | | | **0%** |
| **Gross** | | | **200%** |

### With Position Cap

```sig
portfolio capped:
  // No position > 5%
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
  backtest from 2024-01-01 to 2024-12-31
```

The `cap` limits individual position sizes while maintaining the long-short structure.

### Examples

```sig
// Basic long-short
weights = rank(signal).long_short(top=0.2, bottom=0.2)

// Wider spread
weights = rank(signal).long_short(top=0.3, bottom=0.3)

// Asymmetric (more longs than shorts)
weights = rank(signal).long_short(top=0.3, bottom=0.1)

// With position cap
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
```

## Neutralization

### `neutralize(x, by)`

Remove group exposure by demeaning within groups.

```sig
neutralize(x: Panel, by: Panel) -> Panel
```

```sig
signal sector_neutral:
  raw = zscore(ret(prices, 60))
  // Demean within each sector
  neutral = neutralize(raw, by=sectors)
  emit neutral
```

After neutralization, each group (sector) has mean = 0.

### Use Cases

**Sector Neutralization:**

```sig
signal sector_neutral:
  momentum = zscore(ret(prices, 60))
  neutral = neutralize(momentum, by=sectors)
  emit neutral
```

**Industry Neutralization:**

```sig
signal industry_neutral:
  value = zscore(book_to_market)
  neutral = neutralize(value, by=industries)
  emit neutral
```

**Country Neutralization:**

```sig
signal country_neutral:
  momentum = zscore(ret(prices, 60))
  neutral = neutralize(momentum, by=countries)
  emit neutral
```

## Clipping

### `clip(x, lo, hi)`

Bound values to a range.

```sig
clip(x: Panel, lo: Scalar, hi: Scalar) -> Panel
```

```sig
signal bounded:
  raw = zscore(ret(prices, 20))
  // Clip to [-3, 3]
  bounded = clip(raw, -3, 3)
  emit bounded
```

### For Weights

```sig
portfolio constrained:
  raw_weights = some_weight_calculation
  // Limit position sizes
  bounded_weights = clip(raw_weights, -0.05, 0.05)
```

## Common Patterns

### Basic Long-Short

```sig
signal momentum:
  emit zscore(ret(prices, 60))

portfolio basic:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Sector-Neutral Long-Short

```sig
signal sector_neutral_mom:
  raw = zscore(ret(prices, 60))
  neutral = neutralize(raw, by=sectors)
  emit neutral

portfolio sector_neutral:
  weights = rank(sector_neutral_mom).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Position-Capped

```sig
signal signal:
  emit zscore(ret(prices, 60))

portfolio capped:
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.03)
  backtest from 2024-01-01 to 2024-12-31
```

### Combined Signals

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal combined:
  emit 0.5 * momentum + 0.5 * value

portfolio multi_factor:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Long-Only (Conceptual)

```sig
signal signal:
  scores = zscore(ret(prices, 60))
  // Only keep positive scores
  long_only = where(scores > 0, scores, 0)
  emit long_only

portfolio long:
  // Use rank for top selection
  weights = rank(signal).long_short(top=0.2, bottom=0)
  backtest from 2024-01-01 to 2024-12-31
```

## Why Use Rank?

`rank()` before `long_short()` provides:

1. **Robustness**: Not sensitive to outliers
2. **Uniform Distribution**: Equal weight to each position
3. **Consistency**: Same number of positions regardless of signal scale

### Without Rank (Signal-Weighted)

```sig
// Weights proportional to signal magnitude
// Sensitive to outliers!
```

### With Rank (Equal-Weighted)

```sig
// Equal weight to each position
// Robust to outliers
weights = rank(signal).long_short(top=0.2, bottom=0.2)
```

## Weight Properties

### Dollar Neutral

With equal `top` and `bottom`:

```sig
weights = rank(signal).long_short(top=0.2, bottom=0.2)
// Sum of long weights = Sum of short weights (absolute)
// Net exposure = 0
```

### Gross Exposure

```sig
// With top=0.2, bottom=0.2
// Gross = 100% (long) + 100% (short) = 200%
```

### Adjusting Exposure

To achieve specific gross exposure:

```sig
// For 150% gross (75% long, 75% short)
weights = rank(signal).long_short(top=0.2, bottom=0.2)
scaled_weights = weights * 0.75
```

## Integration with Backtest

```sig
portfolio full_example:
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
  costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest rebal=21 benchmark=SPY from 2020-01-01 to 2024-12-31
```

## Best Practices

### 1. Always Use Rank

```sig
// Good: Robust to outliers
weights = rank(signal).long_short(top=0.2, bottom=0.2)

// Risky: Sensitive to signal scale
weights = signal.long_short(top=0.2, bottom=0.2)
```

### 2. Apply Position Caps

```sig
// Prevent concentration
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
```

### 3. Consider Sector Neutrality

```sig
// If sector exposure is undesirable
neutral_signal = neutralize(raw_signal, by=sectors)
weights = rank(neutral_signal).long_short(top=0.2, bottom=0.2)
```

### 4. Match Rebalancing to Signal

```sig
// Fast signal: frequent rebalancing
backtest rebal=5 from ...

// Slow signal: infrequent rebalancing
backtest rebal=21 from ...
```

## Next Steps

- [Portfolio Section](../language/portfolio-section.md) - Full portfolio syntax
- [Backtesting](../backtesting/index.md) - Running backtests
- [Constraints](../backtesting/constraints.md) - Advanced constraints
