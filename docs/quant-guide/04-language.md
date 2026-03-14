# Chapter 4: The sigc Language

This chapter provides a complete reference for the sigc domain-specific language (DSL).

## Language Overview

sigc uses a declarative DSL designed for expressing quantitative signals. The language prioritizes:
- Clarity over brevity
- Safety over flexibility
- Performance through compilation

## Basic Syntax

### Comments

```sig
// Single line comment

/*
   Multi-line
   comment
*/
```

### Variables

Variables are immutable. Once assigned, they cannot be changed.

```sig
data prices = load("prices.csv")      // Data variable
signal momentum = prices / lag(prices, 20) - 1  // Signal variable
```

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `data` | Input time series from external source | `data prices = load("file.csv")` |
| `signal` | Derived time series from computation | `signal ret = prices / lag(prices, 1) - 1` |
| `param` | Constant parameter | `param window = 20` |

## Data Loading

### From Files

```sig
// CSV file
data prices = load("data/prices.csv")

// Parquet file (efficient for large data)
data prices = load("data/prices.parquet")

// With explicit format
data prices = load("data/prices.csv", format="csv")
```

### From Databases

```sig
// PostgreSQL
data prices = query("SELECT * FROM prices WHERE date > '2020-01-01'", source="postgres")

// Snowflake
data prices = query("SELECT * FROM prices", source="snowflake")
```

### From Cloud Storage

```sig
// S3
data prices = load("s3://bucket/data/prices.parquet")

// GCS
data prices = load("gs://bucket/data/prices.parquet")
```

## Arithmetic Operations

### Basic Math

```sig
signal a = prices + 10        // Addition
signal b = prices - 10        // Subtraction
signal c = prices * 2         // Multiplication
signal d = prices / 100       // Division
signal e = prices ** 2        // Power
signal f = -prices            // Negation
```

### Element-wise Operations

Operations are automatically applied element-wise across time series:

```sig
signal sum = prices1 + prices2      // Adds each time point
signal ratio = prices1 / prices2    // Divides each time point
```

### Mathematical Functions

```sig
signal a = abs(returns)         // Absolute value
signal b = sqrt(variance)       // Square root
signal c = log(prices)          // Natural log
signal d = exp(log_returns)     // Exponential
signal e = pow(returns, 2)      // Power
```

## Time Series Functions

### Lag and Lead

```sig
signal prev = lag(prices, 1)      // Previous value
signal prev_week = lag(prices, 5) // 5 periods ago
signal next = lead(prices, 1)     // Next value (for analysis only!)
```

**Warning**: Never use `lead()` in actual signals - it causes look-ahead bias!

### Rolling Windows

```sig
// Moving averages
signal sma_20 = sma(prices, 20)     // Simple moving average
signal ema_20 = ema(prices, 20)     // Exponential moving average

// Rolling statistics
signal std_20 = std(returns, 20)    // Rolling standard deviation
signal var_20 = var(returns, 20)    // Rolling variance
signal sum_20 = sum(returns, 20)    // Rolling sum
signal min_20 = min(prices, 20)     // Rolling minimum
signal max_20 = max(prices, 20)     // Rolling maximum
```

### Cumulative Functions

```sig
signal cumret = cumsum(log_returns)   // Cumulative sum
signal cumprod = cumprod(1 + returns) // Cumulative product
signal cummax = cummax(equity)        // Running maximum
signal cummin = cummin(drawdown)      // Running minimum
```

### Differences

```sig
signal change = diff(prices, 1)      // prices - lag(prices, 1)
signal pct_change = pct_change(prices, 1)  // (prices - lag) / lag
```

## Statistical Functions

### Descriptive Statistics

```sig
signal mean_val = mean(returns, 60)     // Rolling mean
signal median_val = median(returns, 60) // Rolling median
signal skew_val = skew(returns, 60)     // Rolling skewness
signal kurt_val = kurt(returns, 60)     // Rolling kurtosis
```

### Standardization

```sig
// Z-score: (x - mean) / std
signal zscore = zscore(returns, 60)

// Rank (percentile)
signal rank = rank(returns)             // Cross-sectional rank
signal ts_rank = ts_rank(returns, 20)   // Time series rank
```

### Correlation and Regression

```sig
// Rolling correlation
signal corr = corr(returns1, returns2, 60)

// Rolling beta (regression coefficient)
signal beta = beta(stock_returns, market_returns, 60)

// Rolling alpha
signal alpha = alpha(stock_returns, market_returns, 60)
```

## Conditional Logic

### If-Else

```sig
// Simple condition
signal capped = if(returns > 0.1, 0.1, returns)

// Nested conditions
signal regime = if(vol > high_thresh, "high",
                   if(vol > low_thresh, "medium", "low"))
```

### Comparisons

```sig
signal is_positive = returns > 0        // Boolean (1 or 0)
signal is_above_ma = prices > sma_50
signal in_range = (prices > lower) & (prices < upper)
```

### Boolean Operations

```sig
signal both = cond1 & cond2       // AND
signal either = cond1 | cond2     // OR
signal not_cond = !condition      // NOT
```

## Cross-Sectional Operations

Operations across assets at each time point.

```sig
// Cross-sectional rank
signal cs_rank = cs_rank(momentum)      // Rank across assets

// Cross-sectional z-score
signal cs_zscore = cs_zscore(momentum)  // Standardize across assets

// Cross-sectional demean
signal demeaned = momentum - cs_mean(momentum)

// Industry-neutral
signal ind_neutral = momentum - group_mean(momentum, industry)
```

## Custom Functions

Define reusable computations:

```sig
// Define function
fn sharpe_ratio(returns, window) {
    mean(returns, window) / std(returns, window) * sqrt(252)
}

// Use function
signal sr = sharpe_ratio(returns, 60)
```

## Parameters

Parameters allow external configuration:

```sig
// Define with defaults
param lookback = 20
param threshold = 0.05

// Use in signals
signal momentum = prices / lag(prices, lookback) - 1
signal filtered = if(abs(momentum) > threshold, momentum, 0)

output filtered
```

Override from command line:
```bash
sigc run strategy.sig --param lookback=60 --param threshold=0.10
```

## Output

Specify which signals to output:

```sig
// Single output
output momentum

// Multiple outputs
output momentum, value, quality

// Named outputs
output {
    main: combined_signal,
    debug: raw_momentum
}
```

## Complete Example

```sig
// momentum_quality.sig - Momentum-quality combination strategy

// Parameters
param mom_window = 60
param vol_window = 20
param quality_weight = 0.5

// Load data
data prices = load("prices.csv")
data earnings = load("earnings.csv")
data book_value = load("book.csv")

// Calculate returns and volatility
signal returns = pct_change(prices, 1)
signal volatility = std(returns, vol_window)

// Momentum signal
signal raw_momentum = prices / lag(prices, mom_window) - 1
signal momentum = zscore(raw_momentum, 252)

// Quality signal (ROE)
signal roe = earnings / book_value
signal quality = zscore(roe, 252)

// Combine signals
signal combined = (1 - quality_weight) * momentum + quality_weight * quality

// Volatility-adjust
signal final = combined / volatility

// Cross-sectional standardize
signal output_signal = cs_zscore(final)

output output_signal
```

## Best Practices

### Naming Conventions

```sig
// Good: descriptive names
signal momentum_20d = prices / lag(prices, 20) - 1
signal vol_adjusted_signal = signal / volatility

// Bad: cryptic names
signal m = prices / lag(prices, 20) - 1
signal x = signal / volatility
```

### Signal Pipeline

Structure code as a clear pipeline:

```sig
// 1. Load data
data prices = load("prices.csv")

// 2. Calculate intermediate values
signal returns = pct_change(prices, 1)
signal vol = std(returns, 20)

// 3. Build raw signals
signal momentum = prices / lag(prices, 60) - 1

// 4. Transform and normalize
signal zscore_mom = zscore(momentum, 252)

// 5. Final output
output zscore_mom
```

### Avoid Look-Ahead Bias

```sig
// WRONG: Uses future data
signal bad = lead(prices, 1) / prices - 1

// CORRECT: Uses only past data
signal good = prices / lag(prices, 1) - 1
```

### Handle Missing Data

```sig
// Fill forward
signal filled = ffill(prices)

// Fill with value
signal filled = fill(prices, 0)

// Drop NaN
signal clean = dropna(prices)
```

## Debugging

### Inspect Intermediate Values

```sig
// Output intermediate signals for debugging
output {
    final: output_signal,
    raw_mom: raw_momentum,
    vol: volatility
}
```

### Use Verbose Mode

```bash
sigc run strategy.sig --verbose
```

### Check Data Loading

```bash
sigc inspect data/prices.csv
```

## Performance Tips

### Use Appropriate Window Sizes

Larger windows = more computation:
```sig
// Fast
signal fast_ma = sma(prices, 20)

// Slow
signal slow_ma = sma(prices, 500)
```

### Avoid Redundant Calculations

```sig
// Bad: calculates sma twice
signal upper = sma(prices, 20) + 2 * std(prices, 20)
signal lower = sma(prices, 20) - 2 * std(prices, 20)

// Good: calculate once
signal ma = sma(prices, 20)
signal sd = std(prices, 20)
signal upper = ma + 2 * sd
signal lower = ma - 2 * sd
```

### Use SIMD-Optimized Functions

sigc automatically uses optimized kernels for large datasets:
- `sma`, `ema`, `std` - SIMD accelerated
- Rolling operations on large windows

## Key Takeaways

1. **Declarative style**: Describe what you want, not how to compute it
2. **Immutable variables**: Once set, values don't change
3. **Automatic alignment**: Time series are aligned automatically
4. **Type safety**: Compiler catches errors before runtime
5. **Performance**: Compiled for fast execution

## Next Chapter

[Chapter 5: Backtesting Methodology](05-backtesting.md) - Validate your signals properly.
