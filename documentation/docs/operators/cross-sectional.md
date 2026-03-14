# Cross-Sectional Operators

Operators that compute values across assets at each point in time.

## Normalization

### `zscore(x)`

Cross-sectional z-score standardization.

```sig
zscore(x: Panel) -> Panel
```

For each date: subtract mean, divide by standard deviation.

```sig
signal example:
  returns = ret(prices, 20)
  normalized = zscore(returns)
  // Result: mean=0, std=1 at each date
  emit normalized
```

### `rank(x)`

Cross-sectional rank (0 to 1).

```sig
rank(x: Panel) -> Panel
```

```sig
signal example:
  returns = ret(prices, 20)
  ranked = rank(returns)
  // Lowest = 0, Highest = 1
  emit ranked
```

### `rank_pct(x)`

Cross-sectional percentile rank (0 to 100).

```sig
rank_pct(x: Panel) -> Panel
```

```sig
signal example:
  percentile = rank_pct(returns)
  // 50th percentile = median
  emit percentile
```

### `scale(x)`

Scale to sum to 1.

```sig
scale(x: Panel) -> Panel
```

```sig
signal example:
  // Convert to weights
  abs_scores = abs(signal)
  weights = scale(abs_scores)
  // Sum of weights = 1
  emit weights
```

### `demean(x)`

Remove cross-sectional mean.

```sig
demean(x: Panel) -> Panel
```

```sig
signal example:
  // Center around zero
  centered = demean(returns)
  // mean = 0 at each date
  emit centered
```

## Outlier Handling

### `winsor(x, p)`

Winsorize at percentile p (clip at p-th and (1-p)-th percentile).

```sig
winsor(x: Panel, p: Scalar) -> Panel
```

```sig
signal example:
  normalized = zscore(returns)
  // Clip at 1st and 99th percentile
  cleaned = winsor(normalized, p=0.01)
  emit cleaned
```

### `clip(x, lo, hi)`

Clip values to range [lo, hi].

```sig
clip(x: Panel, lo: Scalar, hi: Scalar) -> Panel
```

```sig
signal example:
  // Clip scores to [-3, 3]
  bounded = clip(signal, -3, 3)
  emit bounded
```

## Group Operations

### `neutralize(x, by)`

Group neutralization (demean within groups).

```sig
neutralize(x: Panel, by: Panel) -> Panel
```

```sig
signal example:
  // Remove sector bias
  sector_neutral = neutralize(returns, by=sectors)
  emit sector_neutral
```

Each sector will have mean=0.

## Quantile Operations

### `quantile(x, q)`

Cross-sectional q-th quantile value.

```sig
quantile(x: Panel, q: Scalar) -> Panel
```

```sig
signal example:
  median = quantile(returns, 0.5)
  q75 = quantile(returns, 0.75)
  emit median
```

### `bucket(x, n)`

Assign to n buckets (1 to n).

```sig
bucket(x: Panel, n: Scalar) -> Panel
```

```sig
signal example:
  // Quintiles
  quintile = bucket(returns, 5)
  // Returns 1, 2, 3, 4, or 5
  emit quintile
```

### `median(x)`

Cross-sectional median.

```sig
median(x: Panel) -> Panel
```

```sig
signal example:
  med = median(returns)
  // Compare to median
  above_median = where(returns > med, 1, 0)
  emit above_median
```

### `mad(x)`

Median Absolute Deviation.

```sig
mad(x: Panel) -> Panel
```

```sig
signal example:
  // Robust measure of dispersion
  deviation = mad(returns)
  // Robust z-score
  robust_z = (returns - median(returns)) / deviation
  emit robust_z
```

## Common Patterns

### Standard Normalization

```sig
signal normalized:
  raw = ret(prices, 20)
  z = zscore(raw)
  emit z
```

### Robust Normalization

```sig
signal robust:
  raw = ret(prices, 20)
  z = zscore(raw)
  cleaned = winsor(z, p=0.01)
  emit cleaned
```

### Sector Neutralization

```sig
signal sector_neutral:
  raw = zscore(ret(prices, 60))
  neutral = neutralize(raw, by=sectors)
  emit neutral
```

### Rank-Based Signal

```sig
signal ranked:
  // Use ranks instead of raw values
  returns = ret(prices, 60)
  ranked = rank(returns)
  // Rank is robust to outliers
  emit ranked
```

### Quintile Spread

```sig
signal quintile_spread:
  returns = ret(prices, 60)
  q = bucket(returns, 5)
  // Long Q5 (top), short Q1 (bottom)
  signal = where(q == 5, 1, where(q == 1, -1, 0))
  emit signal
```

### Multi-Step Normalization

```sig
signal multi_step:
  // Step 1: Compute raw signal
  raw = ret(prices, 60)

  // Step 2: Z-score normalize
  z = zscore(raw)

  // Step 3: Winsorize outliers
  cleaned = winsor(z, p=0.01)

  // Step 4: Sector neutralize
  neutral = neutralize(cleaned, by=sectors)

  // Step 5: Final z-score
  final = zscore(neutral)

  emit final
```

### Comparison to Median

```sig
signal above_median:
  returns = ret(prices, 20)
  med = median(returns)
  above = where(returns > med, 1, -1)
  emit above
```

### Robust Z-Score

```sig
signal robust_zscore:
  // Use MAD instead of std for robustness
  x = ret(prices, 20)
  med = median(x)
  deviation = mad(x)
  robust_z = (x - med) / deviation
  emit robust_z
```

### Market Neutralization

```sig
signal market_neutral:
  returns = ret(prices, 20)
  // Simple market neutralization
  market_neutral = demean(returns)
  emit zscore(market_neutral)
```

### Industry-Relative

```sig
signal industry_relative:
  returns = ret(prices, 60)
  // Relative to industry peers
  industry_neutral = neutralize(returns, by=industry)
  emit zscore(industry_neutral)
```

## Cross-Sectional vs Time-Series

| Aspect | Cross-Sectional | Time-Series |
|--------|-----------------|-------------|
| Direction | Across assets | Over time |
| Example | zscore(x) | rolling_mean(x, 20) |
| At date t | Uses all assets | Uses one asset |
| Output | Ranks among peers | Historical pattern |

## Type Behavior

All cross-sectional operators:

- Input: Panel (dates × assets)
- Output: Panel (same shape)
- Operate on each date independently

## Best Practices

### 1. Always Normalize

```sig
// Raw returns vary in scale
raw = ret(prices, 20)
// Normalized for comparability
z = zscore(raw)
```

### 2. Handle Outliers

```sig
z = zscore(raw)
cleaned = winsor(z, p=0.01)  // Clip extremes
```

### 3. Consider Sector Effects

```sig
// If signal correlates with sectors
neutral = neutralize(signal, by=sectors)
```

### 4. Use Ranks for Robustness

```sig
// Ranks are robust to outliers
ranked = rank(raw_signal)
```

## Next Steps

- [Time-Series](time-series.md) - Per-asset operators
- [Technical](technical.md) - Technical indicators
- [Portfolio](portfolio.md) - Weight construction
