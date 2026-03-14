# Chapter 10: Language Enhancements

This chapter covers advanced language features including custom functions, macros, type inference, IDE integration, and the VS Code extension.

## Custom Functions

Define reusable functions to encapsulate common patterns.

### Function Definition Syntax

```sig
fn function_name(param1, param2=default_value):
  expression
```

### Example: Momentum Functions

```sig
data:
  prices: load csv from "prices.csv"

// Define custom functions
fn momentum(x, window=20):
  x.ret(periods=1).rolling_mean(window=window)

fn volatility(x, window=20):
  x.ret(periods=1).rolling_std(window=window)

fn sharpe_ratio(x, window=60):
  momentum(x, window=window) / volatility(x, window=window)

signal risk_adjusted:
  score = sharpe_ratio(prices, window=60)
  emit zscore(score)

portfolio main:
  weights = risk_adjusted.long_short(top=0.2, bottom=0.2)
```

### Function Features

**Default Parameters:**
```sig
fn ema_crossover(x, fast=12, slow=26):
  ema(x, span=fast) - ema(x, span=slow)
```

**Nested Function Calls:**
```sig
fn normalized_momentum(x, window=20):
  zscore(momentum(x, window=window))
```

## Macro System

Macros provide reusable signal patterns with typed parameters and multiple statements.

### Macro Definition Syntax

```sig
macro name(param1: type, param2: type = default):
  let var1 = expression
  let var2 = expression
  emit final_expression
```

### Parameter Types

| Type | Description | Example |
|------|-------------|---------|
| `expr` | Any expression (prices, signals) | `px: expr` |
| `number` | Numeric value | `window: number = 20` |
| `string` | String literal | `col: string = "close"` |
| `ident` | Identifier name | `var: ident` |

### Example: Custom Macro

```sig
data:
  prices: load csv from "prices.csv"

// Define a volatility-adjusted momentum macro
macro vol_adj_mom(px: expr, ret_window: number = 20, vol_window: number = 60):
  let r = ret(px, ret_window)
  let vol = rolling_std(r, vol_window)
  let adj = r / vol
  emit zscore(adj)

signal main:
  emit prices

portfolio test:
  weights = main
```

### Built-in Macros

sigc includes 8 built-in macro patterns for common strategies:

| Macro | Description | Parameters |
|-------|-------------|------------|
| `momentum` | Time-series momentum with z-score | `prices: expr`, `lookback: number = 20` |
| `mean_reversion` | Deviation from moving average | `prices: expr`, `window: number = 20` |
| `vol_adj_momentum` | Volatility-adjusted momentum | `prices: expr`, `ret_window: number = 20`, `vol_window: number = 60` |
| `trend` | EMA crossover signal | `prices: expr`, `fast: number = 10`, `slow: number = 50` |
| `rsi_signal` | RSI-based contrarian | `prices: expr`, `period: number = 14` |
| `breakout` | Channel breakout | `prices: expr`, `window: number = 20` |
| `cs_momentum` | Cross-sectional momentum | `prices: expr`, `lookback: number = 252` |
| `quality` | Low volatility factor | `prices: expr`, `window: number = 60` |

### Built-in Macro Examples

```sig
// Momentum macro expands to:
macro momentum(prices: expr, lookback: number = 20):
  let r = ret(prices, lookback)
  emit zscore(r)

// Mean reversion macro expands to:
macro mean_reversion(prices: expr, window: number = 20):
  let ma = rolling_mean(prices, window)
  let std = rolling_std(prices, window)
  let z = (prices - ma) / std
  emit -z

// Trend following macro expands to:
macro trend(prices: expr, fast: number = 10, slow: number = 50):
  let fast_ma = ema(prices, fast)
  let slow_ma = ema(prices, slow)
  emit zscore(fast_ma - slow_ma)
```

## Type Inference System

sigc includes a comprehensive type system with operator signatures and arity checking.

### Type Categories

| Category | Description | Example |
|----------|-------------|---------|
| `Scalar` | Single numeric value | Constants, parameters |
| `TimeSeries` | Vector indexed by time | Rolling operations output |
| `CrossSection` | Vector indexed by asset | Rank, zscore output |
| `Panel` | Matrix (time × asset) | Price data |
| `Boolean` | Condition values | Comparison results |

### Operator Signatures

Every operator has a type signature specifying:

1. **Arity**: Number of required arguments
2. **Input types**: Type requirements for each argument
3. **Output type**: Resulting type category

```
Operator        Arity    Inputs           Output
─────────────────────────────────────────────────────
add, sub, mul   2        Numeric, Numeric SameAsFirst
abs, log, sqrt  1        Numeric          SameAsFirst
rolling_mean    1        Numeric          TimeSeries
zscore, rank    1        Numeric          CrossSection
gt, lt, eq      2        Any, Any         Boolean
where           3        Boolean, Any, Any SameAsFirst
```

### Type Checking Benefits

1. **Arity validation**: Catch wrong number of arguments
2. **Type compatibility**: Ensure operations are valid
3. **Better error messages**: Specific suggestions for fixes

```
Type error: Wrong number of arguments for 'rolling_mean'
  Expected: 1
  Got: 2
  Suggestion: 'rolling_mean' requires 1 argument(s)
```

## Error Messages

sigc provides detailed error messages with source locations.

### Parse Errors

```
error: Expected ')', found ','
  --> line 5:15
     |
   5 | fn test(a, b,):
     |               ^

```

### Type Errors

```
error: Undefined identifier: 'pricse'. Did you mean: 'prices'?
  --> line 8:12
     |
   8 |   score = momentum(pricse)
     |           ^^^^^^^^^^^^^^^
```

### Validation Errors

```
error: Unknown function: 'rolling_avg'. Did you mean: 'rolling_mean'?
  --> line 6:8
     |
   6 |   x.rolling_avg(window=20)
     |   ^^^^^^^^^^^^^^^^^^^^^^^^
```

## VS Code Extension

sigc includes a full-featured VS Code extension for development.

### Installation

```bash
# Build the extension
cd editors/vscode
npm install
npm run compile
npx @vscode/vsce package

# Install in VS Code
# Press Ctrl+Shift+P → "Extensions: Install from VSIX..."
# Select sigc-0.1.0.vsix
```

### Features

**Syntax Highlighting**
- Keywords: `data`, `params`, `signal`, `portfolio`, `fn`, `macro`, `emit`
- Functions: All 50+ built-in operators
- Strings, numbers, comments, operators

**Snippets**

| Prefix | Description |
|--------|-------------|
| `strategy` | Complete strategy template |
| `signal` | Signal block |
| `data` | Data section |
| `params` | Parameters section |
| `portfolio` | Portfolio block |
| `fn` | User-defined function |
| `macro` | Macro definition |
| `momentum` | Momentum signal template |
| `meanrev` | Mean reversion template |
| `volatility` | Volatility signal template |

**Function Snippets**

| Prefix | Description |
|--------|-------------|
| `rmean` | Rolling mean |
| `rstd` | Rolling std |
| `zs` | Z-score |
| `win` | Winsorize |
| `ret` | Return calculation |
| `lag` | Lag function |
| `ema` | Exponential moving average |
| `rsi` | RSI indicator |
| `macd` | MACD indicator |

**Commands**

| Command | Description |
|---------|-------------|
| `sigc: Compile Current File` | Compile the current .sig file |
| `sigc: Run Backtest` | Run backtest on current file |
| `sigc: Explain IR` | Show IR explanation |

### Language Server (LSP)

Configure the language server for advanced features:

```json
{
  "sigc.server.path": "/path/to/sigc-lsp"
}
```

**LSP Features:**

1. **Diagnostics**: Real-time error highlighting as you type
2. **Hover**: Documentation for functions and keywords
3. **Completion**: Context-aware suggestions with snippets
4. **Go to Definition**: Jump to signal, portfolio, function definitions
5. **Document Symbols**: Outline view of signals, portfolios, functions

### Building the LSP Server

```bash
# Build release version
cargo build --release --package sig_lsp

# Binary location
./target/release/sigc-lsp
```

## Best Practices

### Function Design

1. **Keep functions focused**: One concept per function
2. **Use descriptive names**: `momentum_zscore` not `mz`
3. **Provide sensible defaults**: Most common values as defaults
4. **Document complex functions**: Add comments explaining purpose

```sig
// RSI with overbought/oversold levels
// Returns 1 for buy signal (oversold), -1 for sell (overbought)
fn rsi_signal(x, window=14, oversold=30, overbought=70):
  where(rsi(x, window=window) < oversold, 1,
        where(rsi(x, window=window) > overbought, -1, 0))
```

### Macro Design

1. **Use macros for multi-step patterns**: When you need intermediate variables
2. **Type parameters appropriately**: Use `expr` for data, `number` for windows
3. **Provide defaults**: Make macros usable with minimal configuration

```sig
// Good: Clear types, sensible defaults
macro momentum_filter(px: expr, lookback: number = 252, vol_window: number = 60):
  let r = ret(px, lookback)
  let vol = rolling_std(ret(px, 1), vol_window)
  let adj = r / vol
  emit zscore(adj)
```

### Code Organization

```sig
// 1. Data section
data:
  prices: load csv from "data/prices.csv"

// 2. Parameters
params:
  lookback = 20
  threshold = 0.5

// 3. Macros (if needed)
macro custom_signal(px: expr, window: number = 20):
  let r = ret(px, window)
  emit zscore(r)

// 4. Custom functions
fn momentum(x, window=20):
  x.ret(periods=1).rolling_mean(window=window)

// 5. Signals
signal main:
  emit momentum(prices, window=lookback)

// 6. Portfolios
portfolio strategy:
  weights = main.long_short(top=0.2, bottom=0.2)
  backtest rebal=monthly from 2020-01-01 to 2024-12-31
```

## Key Takeaways

1. **Custom functions**: Encapsulate common patterns for reuse
2. **Macros**: Define multi-step patterns with typed parameters
3. **Type system**: Operator signatures catch errors early
4. **Clear errors**: Line numbers and suggestions help fix issues quickly
5. **VS Code extension**: Full IDE support with syntax highlighting and LSP
6. **Code organization**: Follow consistent structure for readability

## Exercises

1. **Function library**: Create a library of 5+ commonly used functions.

2. **Custom macro**: Create a macro that combines momentum and volatility signals.

3. **Multi-factor signal**: Build a signal using multiple custom functions and macros.

4. **IDE exploration**: Install the VS Code extension and explore hover, completion, and go-to-definition.

5. **Error exploration**: Intentionally create errors to understand error messages and type checking.

## Resources

- [Language Reference](04-language.md)
- [Signal Development](03-signals.md)
- [Backtesting Guide](05-backtesting.md)
- [Strategy Library](/strategies/README.md)
