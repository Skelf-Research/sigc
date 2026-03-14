# sig_compiler

Parser and compiler for the sigc quantitative finance DSL.

## Overview

`sig_compiler` transforms sigc DSL source code into executable plans:

- **Lexer & Parser** - Parses the sigc DSL using chumsky
- **Type checking** - Validates types and shapes at compile time
- **IR generation** - Produces optimized intermediate representation
- **Error reporting** - Beautiful error messages with ariadne

## DSL Example

```sig
data:
  prices: load parquet from "data/prices.parquet"

params:
  lookback = 20

signal momentum:
  returns = ret(prices, lookback)
  emit zscore(returns)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Usage

```rust
use sig_compiler::Compiler;

let compiler = Compiler::new();
let ir = compiler.compile(source)?;
```

## Part of sigc

This crate is part of the [sigc](https://github.com/skelf-Research/sigc) quantitative finance platform.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
