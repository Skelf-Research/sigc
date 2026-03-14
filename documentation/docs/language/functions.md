# Custom Functions

Custom functions let you define reusable single-expression computations.

## Syntax

```sig
fn <name>(<param1>, <param2>=<default>, ...):
  <expression>
```

## Basic Usage

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

signal example:
  vol = volatility(prices)        // Uses default window=20
  vol60 = volatility(prices, 60)  // Override window
  emit vol
```

## Function Parameters

### Required Parameters

```sig
fn returns(x, lookback):
  ret(x, lookback)

signal example:
  r = returns(prices, 20)  // Must provide both
  emit r
```

### Default Parameters

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

signal example:
  vol1 = volatility(prices)       // window=20
  vol2 = volatility(prices, 60)   // window=60
  emit vol1
```

### Multiple Defaults

```sig
fn momentum_score(x, lookback=20, skip=5):
  ret(x, lookback) - ret(x, skip)

signal example:
  m1 = momentum_score(prices)           // lookback=20, skip=5
  m2 = momentum_score(prices, 60)       // lookback=60, skip=5
  m3 = momentum_score(prices, 60, 10)   // lookback=60, skip=10
  emit m1
```

## Function Body

The body is a single expression (no variables):

```sig
// CORRECT: Single expression
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

// ERROR: Multiple statements not allowed in fn
fn bad(x, window=20):
  r = ret(x, 1)           // Not allowed
  rolling_std(r, window)
```

For multiple statements, use [macros](macros.md) instead.

## Calling Functions

### Positional Arguments

```sig
fn example(a, b, c=10):
  a + b * c

signal test:
  x = example(1, 2)      // a=1, b=2, c=10
  y = example(1, 2, 3)   // a=1, b=2, c=3
  emit x
```

### Named Arguments

Not yet supported. Use positional arguments.

## Common Function Patterns

### Volatility

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

fn annualized_vol(x, window=252):
  rolling_std(ret(x, 1), window) * sqrt(252)
```

### Sharpe Ratio

```sig
fn sharpe(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window)

fn annualized_sharpe(returns, window=252):
  sharpe(returns, window) * sqrt(252)
```

### Momentum

```sig
fn momentum(x, lookback=20):
  ret(x, lookback)

fn momentum_skip(x, lookback=252, skip=21):
  ret(x, lookback) - ret(x, skip)
```

### Moving Averages

```sig
fn sma(x, window=20):
  rolling_mean(x, window)

fn ema_crossover(x, fast=10, slow=50):
  ema(x, fast) - ema(x, slow)
```

### Normalization

```sig
fn normalize(x):
  zscore(x)

fn normalize_winsor(x, p=0.01):
  winsor(zscore(x), p)
```

### Technical

```sig
fn rsi_signal(x, period=14):
  rsi(x, period) - 50

fn macd_signal(x, fast=12, slow=26, signal=9):
  macd(x, fast, slow, signal)

fn bollinger_position(x, window=20, num_std=2):
  ma = rolling_mean(x, window)
  std = rolling_std(x, window)
  (x - ma) / (num_std * std)
```

## Composing Functions

Functions can call other functions:

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

fn vol_adjusted_returns(x, ret_window=20, vol_window=60):
  ret(x, ret_window) / volatility(x, vol_window)

fn normalized_vol_adj(x, ret_window=20, vol_window=60):
  zscore(vol_adjusted_returns(x, ret_window, vol_window))

signal example:
  emit normalized_vol_adj(prices)
```

## Using Functions in Signals

```sig
fn vol(x, window=60):
  rolling_std(ret(x, 1), window)

fn momentum(x, lookback=20):
  ret(x, lookback)

signal vol_adjusted_momentum:
  m = momentum(prices, 60)
  v = vol(prices, 252)
  emit zscore(m / v)
```

## Using Functions with Parameters

```sig
params:
  lookback = 20
  vol_window = 60

fn momentum(x, lb):
  ret(x, lb)

fn vol(x, window):
  rolling_std(ret(x, 1), window)

signal example:
  m = momentum(prices, lookback)    // Uses param
  v = vol(prices, vol_window)       // Uses param
  emit zscore(m / v)
```

## Functions vs Macros

| Feature | Functions (`fn`) | Macros (`macro`) |
|---------|------------------|------------------|
| Body | Single expression | Multiple statements |
| Variables | Not allowed | `let` statements |
| Output | Return value | `emit` statement |
| Type params | No | Yes (`expr`, `number`) |
| Best for | Simple computations | Complex patterns |

Use functions for:

- Single-expression calculations
- Commonly used operators
- Simple transformations

Use macros for:

- Multi-step computations
- Reusable signal patterns
- Complex logic with intermediates

## Example: Building a Signal Library

```sig
// ============================================
// Signal Library: Common Functions
// ============================================

// --- Returns ---
fn ret_simple(x, n):
  ret(x, n)

fn ret_log(x, n):
  log(x / lag(x, n))

// --- Volatility ---
fn vol(x, window=20):
  rolling_std(ret(x, 1), window)

fn vol_annualized(x, window=252):
  vol(x, window) * sqrt(252)

// --- Momentum ---
fn mom(x, lookback=20):
  ret(x, lookback)

fn mom_skip(x, lookback=252, skip=21):
  ret(x, lookback) - ret(x, skip)

// --- Technical ---
fn rsi_centered(x, period=14):
  rsi(x, period) - 50

fn bollinger_zscore(x, window=20):
  (x - rolling_mean(x, window)) / rolling_std(x, window)

// --- Normalization ---
fn normalize(x):
  zscore(x)

fn clean(x, p=0.01):
  winsor(zscore(x), p)

// ============================================
// Strategy
// ============================================

data:
  prices: load csv from "data/prices.csv"

signal my_signal:
  m = mom_skip(prices, 252, 21)
  v = vol(prices, 60)
  raw = m / v
  emit clean(raw)

portfolio main:
  weights = rank(my_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Best Practices

### 1. Use Descriptive Names

```sig
// Good
fn annualized_volatility(x, window=252):
  rolling_std(ret(x, 1), window) * sqrt(252)

// Avoid
fn av(x, w=252):
  rolling_std(ret(x, 1), w) * sqrt(252)
```

### 2. Provide Sensible Defaults

```sig
// Common window sizes as defaults
fn volatility(x, window=20):   // 20-day is standard
  rolling_std(ret(x, 1), window)

fn momentum(x, lookback=60):   // ~3 months
  ret(x, lookback)
```

### 3. Keep Functions Focused

```sig
// Good: Single purpose
fn volatility(x, window):
  rolling_std(ret(x, 1), window)

// Avoid: Too much in one function
fn everything(x, ret_window, vol_window, winsor_pct):
  winsor(zscore(ret(x, ret_window) / rolling_std(ret(x, 1), vol_window)), winsor_pct)
```

### 4. Document Complex Functions

```sig
// Sharpe ratio: mean return / volatility, annualized
fn sharpe_annualized(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window) * sqrt(252)
```

## Next Steps

- [Macros](macros.md) - Multi-statement reusable patterns
- [Operators](../operators/index.md) - Available operators
- [Signal Section](signal-section.md) - Using functions in signals
