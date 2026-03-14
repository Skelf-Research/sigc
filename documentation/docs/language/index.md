# Language Reference

The sigc DSL (Domain-Specific Language) is designed for expressing quantitative trading strategies concisely and safely.

## Philosophy

The sigc language prioritizes:

- **Readability**: Code should be self-documenting
- **Safety**: Catch errors at compile time
- **Composability**: Build complex strategies from simple parts
- **Performance**: Compile to efficient execution plans

## Program Structure

A sigc program has up to six sections:

```sig
// 1. Data declarations
data:
  prices: load csv from "data/prices.csv"

// 2. Parameter definitions
params:
  lookback = 20

// 3. Custom functions (optional)
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

// 4. Macro definitions (optional)
macro momentum(px: expr, lookback: number = 20):
  let r = ret(px, lookback)
  emit zscore(r)

// 5. Signal computations
signal my_signal:
  returns = ret(prices, lookback)
  emit zscore(returns)

// 6. Portfolio construction
portfolio main:
  weights = rank(my_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Section Reference

| Section | Required | Purpose | Link |
|---------|----------|---------|------|
| `data:` | Yes | Load data sources | [Data Section](data-section.md) |
| `params:` | No | Define parameters | [Params Section](params-section.md) |
| `fn` | No | Custom functions | [Functions](functions.md) |
| `macro` | No | Reusable patterns | [Macros](macros.md) |
| `signal` | Yes | Compute scores | [Signal Section](signal-section.md) |
| `portfolio` | Yes | Build portfolios | [Portfolio Section](portfolio-section.md) |

## Quick Syntax Reference

### Data Loading

```sig
data:
  name: load format from "path"
```

Formats: `csv`, `parquet`, `arrow`

### Parameters

```sig
params:
  name = default_value
```

### Functions

```sig
fn name(param1, param2=default):
  expression
```

### Macros

```sig
macro name(param: type, ...):
  let var = expression
  emit result
```

### Signals

```sig
signal name:
  var = expression
  emit output
```

### Portfolios

```sig
portfolio name:
  weights = construction_expression
  backtest from start to end
```

## Expressions

Expressions compute values from data and operators:

```sig
// Arithmetic
x + y, x - y, x * y, x / y

// Function calls
zscore(x), ret(prices, 20)

// Chaining
rank(signal).long_short(top=0.2, bottom=0.2)

// Conditional
where(condition, true_value, false_value)
```

See [Expressions](expressions.md) for full reference.

## Comments

```sig
// Single-line comment
# Also single-line

signal example:
  x = ret(prices, 20)  // Inline comment
  emit x
```

See [Comments](comments.md) for conventions.

## Keywords

Reserved words in sigc:

| Keyword | Purpose |
|---------|---------|
| `data` | Data section |
| `params` | Parameters section |
| `fn` | Function definition |
| `macro` | Macro definition |
| `signal` | Signal definition |
| `portfolio` | Portfolio definition |
| `emit` | Output statement |
| `let` | Variable in macro |
| `load` | Data loading |
| `from` | Source path |
| `backtest` | Run simulation |
| `rebal` | Rebalancing frequency |
| `benchmark` | Comparison benchmark |

## Identifiers

Valid identifier names:

- Start with letter or underscore
- Contain letters, digits, underscores
- Case-sensitive

```sig
// Valid
my_signal
Signal2
_private

// Invalid
2signal   // starts with digit
my-signal // contains hyphen
```

## Literals

### Numbers

```sig
42        // Integer
3.14      // Float
-0.5      // Negative
1e-3      // Scientific
```

### Strings

```sig
"path/to/file.csv"
"s3://bucket/data.parquet"
```

### Dates

```sig
backtest from 2024-01-01 to 2024-12-31
```

Format: `YYYY-MM-DD`

## Type System

sigc infers types and checks operations:

```sig
signal typed:
  prices     // Panel<Float64>
  returns = ret(prices, 20)  // Panel<Float64>
  z = zscore(returns)        // Panel<Float64>
  emit z
```

See [Type System](../concepts/type-system.md) for details.

## Error Handling

The compiler provides helpful error messages:

```
Error: Unknown identifier 'prces'
  --> strategy.sig:3:14
    |
  3 |   x = zscore(prces)
    |              ^^^^^
    |
help: Did you mean 'prices'?
```

See [Error Messages](../reference/error-messages.md) for common errors.

## Section Index

| Page | Description |
|------|-------------|
| [Syntax](syntax.md) | Complete syntax reference |
| [Data Section](data-section.md) | Loading data sources |
| [Params Section](params-section.md) | Defining parameters |
| [Signal Section](signal-section.md) | Computing signals |
| [Portfolio Section](portfolio-section.md) | Building portfolios |
| [Functions](functions.md) | Custom functions |
| [Macros](macros.md) | Reusable patterns |
| [Expressions](expressions.md) | Expression syntax |
| [Comments](comments.md) | Documentation |

## Next Steps

Start with [Syntax Overview](syntax.md) for a comprehensive reference, or jump to specific sections as needed.
