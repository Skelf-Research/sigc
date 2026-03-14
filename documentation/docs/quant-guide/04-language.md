# Chapter 4: The sigc Language

Master the sigc Domain-Specific Language (DSL).

## Language Overview

sigc uses a purpose-built DSL for trading strategies:

```sig
data:
  source = "prices.parquet"
  format = parquet

signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Program Structure

Every sigc program has three main sections:

```sig
// 1. DATA: Where to get data
data:
  ...

// 2. SIGNALS: How to compute scores
signal name:
  ...

// 3. PORTFOLIO: How to construct positions
portfolio name:
  ...
```

## Data Section

### Basic Data Loading

```sig
data:
  source = "prices.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric as volume
```

### Parquet (Recommended)

```sig
data:
  source = "prices.parquet"
  format = parquet
```

### Multiple Data Sources

```sig
data:
  prices = "prices.parquet"
  fundamentals = "fundamentals.parquet"
  factors = "factors.parquet"
```

### S3 Data

```sig
data:
  source = "s3://mybucket/data/prices.parquet"
  format = parquet
```

## Signal Section

### Basic Signal

```sig
signal momentum:
  ret_60 = ret(prices, 60)
  z = zscore(ret_60)
  emit z
```

### Signal with Intermediate Calculations

```sig
signal complex:
  // Step 1: Calculate returns
  daily_ret = ret(prices, 1)

  // Step 2: Calculate volatility
  vol = rolling_std(daily_ret, 60) * sqrt(252)

  // Step 3: Risk-adjusted momentum
  momentum = ret(prices, 60)
  sharpe_signal = momentum / vol

  // Step 4: Emit final signal
  emit zscore(sharpe_signal)
```

### Using Other Signals

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal combined:
  // Reference other signals
  emit 0.5 * momentum + 0.5 * value
```

## Variables and Expressions

### Arithmetic

```sig
signal math_examples:
  // Basic arithmetic
  sum = a + b
  diff = a - b
  prod = a * b
  quot = a / b

  // With constants
  scaled = prices * 100
  shifted = prices + 10

  // Compound expressions
  result = (a + b) / (c - d)

  emit result
```

### Comparisons

```sig
signal compare_examples:
  // Comparisons (return boolean)
  above_100 = prices > 100
  below_ma = prices < rolling_mean(prices, 20)
  equals = a == b
  not_equals = a != b

  emit above_100
```

### Conditionals

```sig
signal conditional_examples:
  // where(condition, true_value, false_value)
  signal = where(prices > ma, 1, -1)

  // Nested
  signal2 = where(a > 0, 1,
            where(a < 0, -1, 0))

  emit signal
```

### Logical Operations

```sig
signal logical_examples:
  // and, or, not
  both = condition1 and condition2
  either = condition1 or condition2
  neither = not(condition1)

  // Combine
  signal = where(bull and low_vol, momentum, defensive)

  emit signal
```

## Built-in Functions

### Time-Series Functions

```sig
signal ts_examples:
  // Returns
  ret_1d = ret(prices, 1)
  ret_60d = ret(prices, 60)

  // Lags
  prev_price = lag(prices, 1)
  prev_week = lag(prices, 5)

  // Differences
  change = diff(prices, 1)

  // Rolling statistics
  ma_20 = rolling_mean(prices, 20)
  std_60 = rolling_std(prices, 60)
  sum_5 = rolling_sum(volume, 5)
  max_252 = rolling_max(prices, 252)
  min_20 = rolling_min(prices, 20)

  emit ma_20
```

### Cross-Sectional Functions

```sig
signal cs_examples:
  // Standardization
  z = zscore(momentum)        // Mean=0, Std=1

  // Ranking
  r = rank(momentum)          // 1 to N

  // Neutralization
  neutral = neutralize(momentum, by=sectors)

  // Aggregation
  avg = mean(returns)         // Average across assets
  total = sum(weights)
  num = count(prices)

  emit z
```

### Mathematical Functions

```sig
signal math_functions:
  // Basic math
  absolute = abs(returns)
  square_root = sqrt(variance)
  natural_log = ln(prices)
  exponential = exp(log_returns)
  power = pow(base, 2)

  // Trigonometric (rare in finance)
  sine = sin(x)
  cosine = cos(x)

  // Rounding
  rounded = round(value, 2)
  floored = floor(value)
  ceiling = ceil(value)

  emit absolute
```

### Statistical Functions

```sig
signal stat_functions:
  // Quantiles
  median_val = median(prices)
  q75 = quantile(prices, 0.75)

  // Correlation and covariance
  corr = rolling_corr(ret_a, ret_b, 60)
  cov = rolling_cov(ret_a, ret_b, 60)
  var = rolling_var(returns, 60)

  // Ranking over time
  ts_pct = ts_rank(prices, 252) / 252

  emit median_val
```

## Portfolio Section

### Basic Portfolio

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

### With Constraints

```sig
portfolio constrained:
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20
    max_position = 0.03

  backtest from 2020-01-01 to 2024-12-31
```

### With Costs

```sig
portfolio realistic:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(10)
  backtest from 2020-01-01 to 2024-12-31
```

### Rebalancing Frequency

```sig
portfolio monthly:
  weights = ...
  backtest rebal=21 from 2020-01-01 to 2024-12-31  // Monthly

portfolio weekly:
  weights = ...
  backtest rebal=5 from 2020-01-01 to 2024-12-31  // Weekly
```

## Weight Construction

### Long-Short

```sig
// Equal-weighted long/short
weights = rank(signal).long_short(top=0.2, bottom=0.2)

// With position cap
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.03)
```

### Long-Only

```sig
// Top 20% only
weights = rank(signal).long_only(top=0.2)

// With cap
weights = rank(signal).long_only(top=0.2, cap=0.05)
```

### Signal-Weighted

```sig
// Weights proportional to signal
weights = signal.normalize()

// Scaled to target exposure
weights = signal.normalize().scale(gross=2.0)
```

## Parameters

### Defining Parameters

```sig
params:
  lookback: 60
  top_pct: 0.2
  cap: 0.03

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct, cap=cap)
```

### Parameter Ranges (for Optimization)

```sig
params:
  lookback: range(20, 120, 20)   // 20, 40, 60, 80, 100, 120
  top_pct: range(0.1, 0.3, 0.05) // 0.1, 0.15, 0.2, 0.25, 0.3
  rebal: [5, 10, 21]             // Explicit list
```

## Comments

```sig
// Single line comment

/*
  Multi-line
  comment
*/

signal momentum:
  // Explain your logic
  ret_60 = ret(prices, 60)  // 60-day return
  emit zscore(ret_60)
```

## Complete Example

```sig
/*
  Multi-Factor Strategy
  Combines momentum, value, and quality factors
*/

// Data configuration
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Parameters
params:
  mom_lookback: 60
  value_weight: 0.3
  quality_weight: 0.3
  top_bottom: 0.2

// Individual factors
signal momentum:
  // 60-day momentum, sector-neutralized
  raw = ret(prices, mom_lookback)
  emit neutralize(zscore(raw), by=sectors)

signal value:
  // Book-to-market ratio
  btm = book_value / market_cap
  emit neutralize(zscore(btm), by=sectors)

signal quality:
  // Return on equity
  roe = net_income / equity
  emit neutralize(zscore(roe), by=sectors)

// Combined signal
signal multi_factor:
  mom_weight = 1 - value_weight - quality_weight
  combined = mom_weight * momentum + value_weight * value + quality_weight * quality
  emit combined

// Portfolio construction
portfolio main:
  weights = rank(multi_factor).long_short(
    top = top_bottom,
    bottom = top_bottom,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure: [-0.1, 0.1]
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Best Practices

### 1. Use Meaningful Names

```sig
// Good
signal momentum_12_1:
signal value_btm:

// Bad
signal s1:
signal x:
```

### 2. Break Down Complex Logic

```sig
// Good
signal complex:
  step1 = calculation1
  step2 = calculation2(step1)
  step3 = calculation3(step2)
  emit step3

// Bad
signal complex:
  emit calculation3(calculation2(calculation1))
```

### 3. Comment Your Intent

```sig
signal adaptive:
  // Use momentum in bull markets, mean reversion in bear
  regime = ma_50 > ma_200
  emit where(regime, momentum, mean_reversion)
```

## Exercises

1. Write a signal that computes RSI (Relative Strength Index)
2. Create a combined signal with momentum, value, and size factors
3. Build a portfolio with sector constraints
4. Parameterize your strategy for optimization

## Next Chapter

Continue to [Chapter 5: Backtesting](05-backtesting.md) to learn proper strategy validation.
