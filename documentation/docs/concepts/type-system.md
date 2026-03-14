# Type System

sigc's type system catches errors at compile time, before expensive backtests run.

## Overview

The type system has three components:

1. **Data Types** (DType): float64, int32, bool, string, date
2. **Shapes**: Scalar, vector, matrix dimensions
3. **Semantic Categories**: Time-series, cross-sectional, panel

```sig
signal example:
  prices    // Shape: (T, N) - panel
  ret_20 = ret(prices, 20)  // Shape: (T, N) - panel (time-series op)
  z = zscore(ret_20)        // Shape: (T, N) - panel (cross-sectional op)
  emit z
```

## Data Types

| DType | Description | Example |
|-------|-------------|---------|
| `Float64` | Double-precision float | Prices, returns |
| `Float32` | Single-precision float | Memory-optimized data |
| `Int64` | 64-bit integer | Counts, IDs |
| `Int32` | 32-bit integer | Small integers |
| `Bool` | Boolean | Filters, conditions |
| `String` | Text | Symbols, sectors |
| `Date` | Calendar date | Trading dates |
| `DateTime` | Date and time | Timestamps |

## Shapes

Shapes describe the dimensionality of data:

| Shape | Dimensions | Example |
|-------|------------|---------|
| Scalar | () | Single value: `20`, `0.01` |
| Vector | (N,) | Asset scores at one time |
| Matrix | (T, N) | Full price panel |

### Shape Inference

The compiler infers shapes from operations:

```sig
signal shapes:
  prices        // (T, N) - loaded data
  ret_5 = ret(prices, 5)    // (T, N) - time-series preserves shape
  mean = rolling_mean(ret_5, 20)  // (T, N)
  z = zscore(mean)          // (T, N) - cross-sectional preserves T
  emit z
```

## Semantic Categories

Beyond shape, sigc tracks what kind of data each value represents:

| Category | Description | Operators |
|----------|-------------|-----------|
| `TimeSeries` | Per-asset over time | `ret`, `lag`, `rolling_*`, `ema` |
| `CrossSection` | All assets at one time | `zscore`, `rank`, `winsor` |
| `Panel` | Full time × asset matrix | Most operations |
| `Scalar` | Single value | `20`, `0.01` |
| `Boolean` | True/false | `>`, `<`, `and`, `or` |

### Why Categories Matter

Prevents meaningless operations:

```sig
// ERROR: Can't apply time-series operator to cross-sectional data
signal bad:
  returns = ret(prices, 20)     // TimeSeries operation
  z = zscore(returns)           // CrossSection operation - OK
  lagged_z = lag(z, 5)          // TimeSeries on CrossSection result - OK
  emit lagged_z
```

## Operator Signatures

Each operator has a signature defining valid inputs and outputs:

```
zscore(x: CrossSection | Panel) -> CrossSection | Panel
  - Input: Numeric series (cross-sectional or panel)
  - Output: Same shape, z-scored across assets

ret(x: TimeSeries | Panel, n: Scalar) -> TimeSeries | Panel
  - Input: Time-series data, lookback period
  - Output: Same shape, returns computed

rank(x: CrossSection | Panel) -> CrossSection | Panel
  - Input: Numeric scores
  - Output: Ranks from 0 to 1
```

### Example Signatures

| Operator | Input | Output | Notes |
|----------|-------|--------|-------|
| `ret(x, n)` | Panel, int | Panel | Time-series |
| `lag(x, n)` | Panel, int | Panel | Time-series |
| `zscore(x)` | Panel | Panel | Cross-sectional |
| `rank(x)` | Panel | Panel | Cross-sectional |
| `rolling_mean(x, n)` | Panel, int | Panel | Time-series |
| `where(c, a, b)` | Bool, Any, Any | Any | Conditional |

## Type Errors

The compiler reports type errors with helpful messages:

### Wrong Number of Arguments

```sig
signal bad:
  x = zscore(prices, 20)  // Error!
```

```
Error: zscore takes 1 argument, got 2
  --> strategy.sig:3:7
    |
  3 |   x = zscore(prices, 20)
    |       ^^^^^^^^^^^^^^^^^^
    |
help: zscore(x) takes a single series argument
```

### Type Mismatch

```sig
signal bad:
  x = ret(prices, "twenty")  // Error!
```

```
Error: Expected numeric, got String
  --> strategy.sig:3:19
    |
  3 |   x = ret(prices, "twenty")
    |                   ^^^^^^^^
    |
help: ret(x, n) requires numeric lookback period
      try: ret(prices, 20)
```

### Unknown Identifier

```sig
signal bad:
  x = zscore(prces)  // Typo!
```

```
Error: Unknown identifier 'prces'
  --> strategy.sig:3:14
    |
  3 |   x = zscore(prces)
    |              ^^^^^
    |
help: Did you mean 'prices'?
```

## Type Annotations

Usually, types are inferred. But you can add annotations for clarity:

```sig
// In macros, parameter types are required
macro momentum(px: expr, lookback: number = 20):
  let r = ret(px, lookback)
  emit zscore(r)
```

### Parameter Types

| Type | Description |
|------|-------------|
| `expr` | Any expression |
| `number` | Numeric value (int or float) |
| `string` | String literal |
| `ident` | Identifier name |

## Type Inference

The compiler infers types through the computation graph:

```sig
signal inferred:
  // prices: Panel<Float64, (T, N)>
  prices

  // returns: Panel<Float64, (T, N)>
  // Inferred from ret() signature
  returns = ret(prices, 20)

  // z: Panel<Float64, (T, N)>
  // Inferred from zscore() signature
  z = zscore(returns)

  emit z
```

### Inference Rules

1. **Literals**: Type from value (`20` → Int64, `0.5` → Float64)
2. **Data Sources**: Type from schema (CSV → Float64 by default)
3. **Operators**: Output type from signature
4. **Arithmetic**: Wider type wins (Int32 + Float64 → Float64)

## Benefits of Type Checking

### 1. Catch Errors Early

```sig
// Caught at compile time, not after a 2-hour backtest
signal bad:
  x = unknownfunction(prices)
```

### 2. Better Error Messages

```
Error: Unknown function 'unknownfunction'
help: Did you mean 'rolling_mean' or 'rolling_std'?
```

### 3. IDE Support

The type system enables:

- Hover documentation
- Code completion
- Go to definition
- Refactoring

### 4. Optimization

The compiler uses types to:

- Choose efficient implementations
- Parallelize safe operations
- Eliminate redundant computations

## Working with Types

### Check Types in IDE

Hover over any expression to see its type:

```
z = zscore(returns)
    ^^^^^^^^^^^^^^^
    zscore(x: Panel<Float64, (T, N)>) -> Panel<Float64, (T, N)>
```

### Use Explicit Types When Unclear

```sig
// In functions, specify parameter types
fn volatility(x: expr, window: number = 20):
  rolling_std(ret(x, 1), window)
```

### Handle Missing Data

Use appropriate null handling:

```sig
signal with_nulls:
  raw = ret(prices, 20)
  // fill_nan has signature: (x: Panel, value: Scalar) -> Panel
  filled = fill_nan(raw, 0)
  emit zscore(filled)
```

## Advanced Type Features

### Conditional Types

`where` returns the common type of its branches:

```sig
signal conditional:
  condition = prices > lag(prices, 1)  // Bool
  result = where(condition, 1, -1)     // Int64
  emit result
```

### Type Coercion

Numeric types are coerced when needed:

```sig
signal coerced:
  x = 5        // Int64
  y = 3.14     // Float64
  z = x + y    // Float64 (coerced)
  emit z
```

## Next Steps

- [Architecture](architecture.md) - System design
- [Language Reference](../language/index.md) - Full DSL documentation
- [Operators](../operators/index.md) - Operator signatures
