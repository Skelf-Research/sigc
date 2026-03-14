# Chapter 10: Language Enhancements

This chapter covers advanced language features including custom functions, improved error handling, and IDE integration.

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

**Building Libraries:**

Create a file of common functions:

```sig
// common.sig
fn returns(x, periods=1):
  x.ret(periods=periods)

fn vol(x, window=20):
  returns(x).rolling_std(window=window)

fn mom(x, window=20):
  returns(x).rolling_mean(window=window)

fn sr(x, window=60):
  mom(x, window=window) / vol(x, window=window)
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

## IDE Integration

### Diagnostics System

sigc provides a diagnostics API for IDE integration:

```rust
use sig_compiler::{Compiler, DiagnosticCollector, Severity};

let compiler = Compiler::new();
let mut collector = DiagnosticCollector::new();

// Collect diagnostics during compilation
match compiler.parse(source) {
    Ok(program) => {
        // Analyze for warnings
        analyze_program(&program, &mut collector);
    }
    Err(e) => {
        // Parse errors
        collector.error(e.to_string(), 0..source.len());
    }
}

// Format for display
if collector.has_errors() {
    println!("{}", collector.format(source));
}
```

### Diagnostic Types

```rust
use sig_compiler::{Diagnostic, Severity};

// Create error diagnostic
let error = Diagnostic::error("undefined variable", 10..15)
    .with_code("E0001")
    .with_related("variable defined here", 5..8);

// Create warning diagnostic
let warning = Diagnostic::warning("unused variable", 20..25);
```

### Position Information

Convert byte offsets to line/column positions:

```rust
use sig_compiler::diagnostics::byte_to_position;

let pos = byte_to_position(source, byte_offset);
println!("Line {}, Column {}", pos.line, pos.column);
```

## Language Server Protocol (LSP)

The diagnostics system is designed for LSP integration.

### Future LSP Features

1. **Hover information**: Type info and documentation
2. **Go to definition**: Jump to function definitions
3. **Autocomplete**: Suggest functions and parameters
4. **Diagnostics**: Real-time error highlighting
5. **Code actions**: Quick fixes for common errors

### VS Code Extension (Planned)

```json
{
  "name": "sigc-lang",
  "displayName": "sigc Language Support",
  "description": "Language support for sigc DSL",
  "engines": {
    "vscode": "^1.60.0"
  },
  "contributes": {
    "languages": [{
      "id": "sigc",
      "aliases": ["sigc", "sig"],
      "extensions": [".sig"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "sigc",
      "scopeName": "source.sig",
      "path": "./syntaxes/sigc.tmLanguage.json"
    }]
  }
}
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

### Error Handling

1. **Check error messages carefully**: They often suggest fixes
2. **Verify parameter names**: Match function definition exactly
3. **Use suggested alternatives**: "Did you mean" suggestions are usually correct

### Code Organization

```sig
// 1. Data section
data:
  prices: load csv from "data/prices.csv"

// 2. Parameters
params:
  lookback = 20
  threshold = 0.5

// 3. Custom functions
fn momentum(x, window=20):
  x.ret(periods=1).rolling_mean(window=window)

// 4. Signals
signal main:
  emit momentum(prices, window=lookback)

// 5. Portfolios
portfolio strategy:
  weights = main.long_short(top=0.2, bottom=0.2)
  backtest rebal=monthly from 2020-01-01 to 2024-12-31
```

## Key Takeaways

1. **Custom functions**: Encapsulate common patterns for reuse
2. **Default parameters**: Make functions flexible with sensible defaults
3. **Clear errors**: Line numbers and suggestions help fix issues quickly
4. **IDE support**: Diagnostics system enables editor integration
5. **Code organization**: Follow consistent structure for readability

## Exercises

1. **Function library**: Create a library of 5+ commonly used functions.

2. **Composite signals**: Build a signal using multiple custom functions.

3. **Error exploration**: Intentionally create errors to understand error messages.

4. **Parameter tuning**: Create functions with configurable parameters for optimization.

## Resources

- [Language Reference](04-language.md)
- [Signal Development](03-signals.md)
- [Backtesting Guide](05-backtesting.md)
