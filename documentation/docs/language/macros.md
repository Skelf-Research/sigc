# Macros

Macros define reusable multi-statement patterns with typed parameters.

## Syntax

```sig
macro <name>(<param>: <type>, <param>: <type> = <default>, ...):
  let <var> = <expression>
  ...
  emit <expression>
```

## Basic Usage

```sig
macro momentum_signal(px: expr, lookback: number = 20):
  let r = ret(px, lookback)
  let normalized = zscore(r)
  emit normalized

signal my_momentum:
  emit momentum_signal(prices, 60)
```

## Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `expr` | Any expression | `px: expr` |
| `number` | Numeric value | `lookback: number` |
| `string` | String literal | `name: string` |
| `ident` | Identifier | `col: ident` |

### `expr` Type

Accepts any expression:

```sig
macro normalize(x: expr):
  emit zscore(x)

signal example:
  emit normalize(ret(prices, 20))  // Expression as argument
```

### `number` Type

Accepts numeric literals or parameters:

```sig
macro rolling(x: expr, window: number):
  emit rolling_mean(x, window)

signal example:
  emit rolling(prices, 20)
```

### Default Values

```sig
macro momentum(px: expr, lookback: number = 20, skip: number = 5):
  let total = ret(px, lookback)
  let recent = ret(px, skip)
  emit zscore(total - recent)

signal example:
  a = momentum(prices)            // lookback=20, skip=5
  b = momentum(prices, 60)        // lookback=60, skip=5
  c = momentum(prices, 60, 10)    // lookback=60, skip=10
  emit a
```

## Macro Body

### `let` Statements

Define intermediate variables:

```sig
macro vol_adjusted_momentum(px: expr, ret_window: number = 20, vol_window: number = 60):
  let returns = ret(px, ret_window)
  let vol = rolling_std(ret(px, 1), vol_window)
  let vol_adj = returns / vol
  let normalized = zscore(vol_adj)
  emit winsor(normalized, p=0.01)
```

### `emit` Statement

The final output (required):

```sig
macro example(x: expr):
  let processed = zscore(x)
  emit processed  // This is the output
```

## Built-in Macros

sigc includes 8 built-in macros:

### `momentum`

```sig
// 12-1 month momentum
macro momentum(prices: expr, lookback: number = 20):
  let r = ret(prices, lookback)
  emit zscore(r)
```

### `mean_reversion`

```sig
// Mean reversion from moving average
macro mean_reversion(prices: expr, window: number = 20):
  let ma = rolling_mean(prices, window)
  let std = rolling_std(prices, window)
  let z = (prices - ma) / std
  emit -zscore(z)
```

### `vol_adj_momentum`

```sig
// Volatility-adjusted momentum
macro vol_adj_momentum(px: expr, ret_window: number = 20, vol_window: number = 60):
  let r = ret(px, ret_window)
  let vol = rolling_std(ret(px, 1), vol_window)
  emit zscore(r / vol)
```

### `trend`

```sig
// EMA crossover trend
macro trend(prices: expr, fast: number = 10, slow: number = 50):
  let fast_ema = ema(prices, fast)
  let slow_ema = ema(prices, slow)
  emit zscore(fast_ema - slow_ema)
```

### `rsi_signal`

```sig
// RSI-based contrarian signal
macro rsi_signal(prices: expr, period: number = 14):
  let rsi_val = rsi(prices, period)
  let centered = rsi_val - 50
  emit -zscore(centered)
```

### `breakout`

```sig
// Channel breakout
macro breakout(prices: expr, window: number = 20):
  let high = rolling_max(prices, window)
  let low = rolling_min(prices, window)
  let mid = (high + low) / 2
  emit zscore((prices - mid) / (high - low))
```

### `cs_momentum`

```sig
// Cross-sectional momentum
macro cs_momentum(prices: expr, lookback: number = 252):
  let r = ret(prices, lookback)
  emit rank(r)
```

### `quality`

```sig
// Quality/low-volatility factor
macro quality(prices: expr, window: number = 60):
  let vol = rolling_std(ret(prices, 1), window)
  emit -zscore(vol)
```

## Invoking Macros

```sig
signal using_builtin:
  emit momentum(prices, 60)

signal using_multiple:
  mom = momentum(prices, 60)
  rev = mean_reversion(prices, 20)
  emit 0.7 * mom + 0.3 * rev
```

## Custom Macros

### Simple Example

```sig
macro normalize_and_clean(x: expr, winsor_pct: number = 0.01):
  let normalized = zscore(x)
  let cleaned = winsor(normalized, p=winsor_pct)
  emit cleaned

signal example:
  raw = ret(prices, 20)
  emit normalize_and_clean(raw)
```

### Complex Example

```sig
macro sector_neutral_momentum(
  px: expr,
  sectors: expr,
  lookback: number = 60,
  skip: number = 5,
  vol_window: number = 252
):
  // Compute momentum
  let total = ret(px, lookback)
  let recent = ret(px, skip)
  let raw_mom = total - recent

  // Volatility adjustment
  let vol = rolling_std(ret(px, 1), vol_window)
  let vol_adj = raw_mom / vol

  // Sector neutralization
  let neutral = neutralize(vol_adj, by=sectors)

  // Clean and normalize
  let normalized = zscore(neutral)
  let cleaned = winsor(normalized, p=0.01)

  emit cleaned

signal my_signal:
  emit sector_neutral_momentum(prices, sectors, 252, 21, 252)
```

## Macro Libraries

Build a library of reusable macros:

```sig
// ============================================
// Macro Library: Factor Construction
// ============================================

// --- Momentum Variants ---
macro time_series_momentum(px: expr, lookback: number = 252, skip: number = 21):
  let total = ret(px, lookback)
  let recent = ret(px, skip)
  emit zscore(total - recent)

macro cross_sectional_momentum(px: expr, lookback: number = 60):
  let r = ret(px, lookback)
  emit rank(r)

macro vol_adjusted_momentum(px: expr, ret_window: number = 60, vol_window: number = 252):
  let r = ret(px, ret_window)
  let vol = rolling_std(ret(px, 1), vol_window)
  emit zscore(r / vol)

// --- Mean Reversion Variants ---
macro bollinger_reversion(px: expr, window: number = 20, num_std: number = 2):
  let ma = rolling_mean(px, window)
  let std = rolling_std(px, window)
  let z = (px - ma) / (num_std * std)
  emit -zscore(z)

macro short_term_reversal(px: expr, lookback: number = 5):
  let r = ret(px, lookback)
  emit -zscore(r)

// --- Technical Patterns ---
macro trend_following(px: expr, fast: number = 10, slow: number = 50):
  let fast_ma = ema(px, fast)
  let slow_ma = ema(px, slow)
  let trend = fast_ma - slow_ma
  emit zscore(trend)

macro rsi_contrarian(px: expr, period: number = 14):
  let rsi_val = rsi(px, period)
  let centered = rsi_val - 50
  emit -zscore(centered / 25)

// --- Utility Macros ---
macro clean_signal(x: expr, winsor_pct: number = 0.01):
  let normalized = zscore(x)
  emit winsor(normalized, p=winsor_pct)

macro vol_scale(x: expr, target_vol: number = 0.15, vol_window: number = 252):
  let current_vol = rolling_std(x, vol_window) * sqrt(252)
  emit x * (target_vol / current_vol)
```

## Macros vs Functions

| Feature | Functions (`fn`) | Macros (`macro`) |
|---------|------------------|------------------|
| Body | Single expression | Multiple `let` + `emit` |
| Variables | Not allowed | `let` statements |
| Parameters | Untyped | Typed (`expr`, `number`) |
| Output | Return value | `emit` statement |
| Best for | Simple ops | Complex patterns |

### When to Use Functions

```sig
// Simple computations
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

fn sharpe(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window)
```

### When to Use Macros

```sig
// Complex multi-step patterns
macro sophisticated_momentum(px: expr, lookback: number = 60):
  let raw = ret(px, lookback)
  let vol = rolling_std(ret(px, 1), 252)
  let vol_adj = raw / vol
  let normalized = zscore(vol_adj)
  let cleaned = winsor(normalized, p=0.01)
  emit cleaned
```

## Best Practices

### 1. Use Descriptive Names

```sig
// Good
macro sector_neutral_vol_adjusted_momentum(...)

// Avoid
macro snvam(...)
```

### 2. Provide Sensible Defaults

```sig
macro momentum(px: expr, lookback: number = 60, skip: number = 5):
  // 60 days is ~3 months, 5 days is ~1 week
  ...
```

### 3. Document Complex Macros

```sig
// Sector-neutral momentum with volatility adjustment
// Based on Moskowitz, Ooi, Pedersen (2012)
//
// Parameters:
//   px: Price data
//   sectors: Sector classifications
//   lookback: Return computation window (default 252 = 1 year)
//   vol_window: Volatility estimation window
//
macro sector_neutral_momentum(px: expr, sectors: expr, lookback: number = 252, vol_window: number = 60):
  ...
```

### 4. Keep Macros Focused

```sig
// Good: Single purpose
macro vol_adjustment(x: expr, window: number = 60):
  let vol = rolling_std(ret(x, 1), window)
  emit x / vol

// Avoid: Too many concerns
macro everything(px: expr, ...):
  // Too complex - break into smaller macros
```

### 5. Compose Smaller Macros

```sig
macro vol_adjustment(x: expr, window: number = 60):
  let vol = rolling_std(ret(x, 1), window)
  emit x / vol

macro clean_signal(x: expr):
  emit winsor(zscore(x), p=0.01)

// Compose in signal
signal composed:
  raw = ret(prices, 60)
  vol_adj = vol_adjustment(raw, 252)
  emit clean_signal(vol_adj)
```

## Next Steps

- [Functions](functions.md) - Simple reusable expressions
- [Signal Section](signal-section.md) - Using macros in signals
- [Strategy Library](../strategies/index.md) - Complete strategy examples
