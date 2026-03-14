# Syntax Overview

This page provides a complete reference for sigc syntax.

## Program Structure

```sig
// Complete program structure

data:
  <data_declarations>

params:
  <parameter_definitions>

fn <name>(<params>):
  <expression>

macro <name>(<typed_params>):
  <macro_body>

signal <name>:
  <computations>
  emit <output>

portfolio <name>:
  <construction>
  backtest [options] from <date> to <date>
```

## Lexical Elements

### Whitespace

Whitespace (spaces, tabs, newlines) separates tokens but is otherwise insignificant:

```sig
// These are equivalent
x = zscore(ret(prices, 20))
x=zscore(ret(prices,20))
```

Indentation is conventional but not required:

```sig
signal example:
  x = ret(prices, 20)
  emit x

// Also valid (but less readable)
signal example:
x = ret(prices, 20)
emit x
```

### Comments

```sig
// C-style single-line
# Shell-style single-line

x = ret(prices, 20)  // Inline comment
```

Multi-line comments are not supported.

### Identifiers

```
identifier ::= (letter | '_') (letter | digit | '_')*
letter     ::= 'a'..'z' | 'A'..'Z'
digit      ::= '0'..'9'
```

Examples: `prices`, `my_signal`, `lookback_20`, `_internal`

### Keywords

Reserved words cannot be used as identifiers:

```
data, params, fn, macro, signal, portfolio,
emit, let, load, from, backtest, rebal, benchmark,
and, or, not, where, by, top, bottom, cap
```

### Literals

#### Numbers

```
integer ::= '-'? digit+
float   ::= '-'? digit+ '.' digit+ ('e' '-'? digit+)?
```

Examples: `42`, `-5`, `3.14`, `-0.5`, `1e-3`, `2.5e10`

#### Strings

```
string ::= '"' <characters> '"'
```

Examples: `"data/prices.csv"`, `"s3://bucket/file.parquet"`

#### Dates

```
date ::= digit{4} '-' digit{2} '-' digit{2}
```

Examples: `2024-01-01`, `2020-12-31`

## Data Section

```sig
data:
  <name>: load <format> from <path> [options]
```

### Formats

```sig
data:
  prices: load csv from "data/prices.csv"
  volume: load parquet from "data/volume.parquet"
  factors: load arrow from "data/factors.arrow"
```

### Options

```sig
data:
  prices: load csv from "data/prices.csv" adjust=split_div
  sectors: load csv from "data/sectors.csv" dtype=category
```

### S3 Sources

```sig
data:
  prices: load parquet from "s3://bucket/prices.parquet"
```

## Params Section

```sig
params:
  <name> = <default_value>
```

### Examples

```sig
params:
  lookback = 20
  threshold = 0.5
  top_pct = 0.2
  skip_days = 5
```

## Function Definition

```sig
fn <name>(<param1>, <param2>=<default>, ...):
  <expression>
```

### Examples

```sig
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

fn sharpe(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window)

fn momentum_score(prices, fast=10, slow=60):
  zscore(ret(prices, fast) - ret(prices, slow))
```

### Calling Functions

```sig
signal example:
  vol = volatility(prices)          // Uses default window=20
  vol60 = volatility(prices, 60)    // Overrides window
  emit vol
```

## Macro Definition

```sig
macro <name>(<param>: <type>, <param>: <type> = <default>, ...):
  let <var> = <expression>
  ...
  emit <expression>
```

### Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `expr` | Any expression | `px: expr` |
| `number` | Numeric value | `lookback: number` |
| `string` | String literal | `path: string` |
| `ident` | Identifier | `name: ident` |

### Examples

```sig
macro momentum_signal(px: expr, lookback: number = 20, vol_window: number = 60):
  let r = ret(px, lookback)
  let vol = rolling_std(ret(px, 1), vol_window)
  let vol_adj = r / vol
  emit zscore(vol_adj)

macro mean_reversion(px: expr, window: number = 20):
  let ma = rolling_mean(px, window)
  let std = rolling_std(px, window)
  let z = (px - ma) / std
  emit -zscore(z)
```

### Invoking Macros

```sig
signal example:
  emit momentum_signal(prices, 30, 90)

signal another:
  emit mean_reversion(prices)  // Uses defaults
```

## Signal Section

```sig
signal <name>:
  <var> = <expression>
  ...
  emit <expression>
```

### Variable Assignment

```sig
signal example:
  returns = ret(prices, 20)
  normalized = zscore(returns)
  cleaned = winsor(normalized, p=0.01)
  emit cleaned
```

### Multiple Signals

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal reversal:
  emit -zscore(ret(prices, 5))

signal combined:
  mom = zscore(ret(prices, 60))
  rev = -zscore(ret(prices, 5))
  emit 0.7 * mom + 0.3 * rev
```

### Using Other Signals

```sig
signal base:
  emit zscore(ret(prices, 20))

signal derived:
  emit winsor(base, p=0.01)
```

## Portfolio Section

```sig
portfolio <name>:
  weights = <weight_expression>
  [costs = <cost_expression>]
  backtest [rebal=<n>] [benchmark=<symbol>] from <date> to <date>
```

### Weight Construction

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
```

### With Position Cap

```sig
portfolio capped:
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
```

### With Costs

```sig
portfolio institutional:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest from 2024-01-01 to 2024-12-31
```

### With Benchmark

```sig
portfolio hedged:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest benchmark=SPY from 2024-01-01 to 2024-12-31
```

### Rebalancing Frequency

```sig
portfolio weekly:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest rebal=5 from 2024-01-01 to 2024-12-31
```

## Expressions

### Arithmetic

```
+  addition
-  subtraction (binary) or negation (unary)
*  multiplication
/  division
```

```sig
x + y
x - y
-x
x * y
x / y
x * 2 + y / 3
```

### Comparison

```
>   greater than
<   less than
>=  greater or equal
<=  less or equal
==  equal
!=  not equal
```

```sig
prices > threshold
score >= 0
status != excluded
```

### Logical

```
and  logical AND
or   logical OR
not  logical NOT
```

```sig
condition1 and condition2
a or b
not(excluded)
```

### Conditional

```sig
where(condition, true_value, false_value)
```

```sig
adjusted = where(is_nan(prices), 0, prices)
signed = where(score > 0, 1, -1)
```

### Function Calls

```sig
function(arg1, arg2, ...)
function(arg, named=value)
```

```sig
ret(prices, 20)
zscore(returns)
winsor(scores, p=0.01)
rolling_mean(data, 20)
```

### Method Chaining

```sig
expression.method(args)
```

```sig
rank(signal).long_short(top=0.2, bottom=0.2)
```

### Operator Precedence

From highest to lowest:

1. Function calls, method calls
2. Unary `-`, `not`
3. `*`, `/`
4. `+`, `-`
5. `>`, `<`, `>=`, `<=`, `==`, `!=`
6. `and`
7. `or`

Use parentheses to override:

```sig
(a + b) * c
not(x and y)
```

## Complete Example

```sig
// Volatility-Adjusted Momentum Strategy
// Implements 12-1 month momentum with vol adjustment

data:
  prices: load parquet from "s3://bucket/prices.parquet" adjust=split_div
  sectors: load csv from "data/sectors.csv" dtype=category

params:
  lookback = 252
  skip = 21
  vol_window = 60
  winsor_pct = 0.01
  top_pct = 0.2
  bottom_pct = 0.2
  max_weight = 0.05

// Custom function for volatility
fn vol(x, window=20):
  rolling_std(ret(x, 1), window)

// Macro for momentum calculation
macro mom_signal(px: expr, lb: number = 252, sk: number = 21):
  let total = ret(px, lb)
  let recent = ret(px, sk)
  let mom = total - recent
  emit zscore(mom)

signal momentum:
  // Use macro
  raw = mom_signal(prices, lookback, skip)

  // Volatility adjustment
  volatility = vol(prices, vol_window)
  vol_adj = raw / volatility

  // Sector neutralization
  neutral = neutralize(vol_adj, by=sectors)

  // Clean outliers
  cleaned = winsor(zscore(neutral), winsor_pct)

  emit cleaned

portfolio main:
  weights = rank(momentum).long_short(
    top=top_pct,
    bottom=bottom_pct,
    cap=max_weight
  )
  costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest rebal=21 benchmark=SPY from 2020-01-01 to 2024-12-31
```

## Next Steps

- [Data Section](data-section.md) - Detailed data loading
- [Signal Section](signal-section.md) - Signal computation
- [Operators](../operators/index.md) - Available operators
