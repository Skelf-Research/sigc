# sigc

A Rust-based quantitative finance platform with a DSL for trading signals and backtesting.

## Overview

`sigc` is a single binary that bundles:

- **Compiler** - Parse and compile the sigc DSL
- **Runtime** - Execute strategies with 120+ operators
- **Daemon** - Long-running service mode with RPC
- **CLI** - Interactive command-line interface

## Installation

```bash
cargo install sigc
```

Or build from source:

```bash
git clone https://github.com/skelf-Research/sigc.git
cd sigc
cargo build --release
```

## Quick Start

```bash
# Compile a strategy
sigc compile strategy.sig

# Run a backtest
sigc run strategy.sig

# Start daemon mode
sigc daemon
```

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

## Documentation

Full documentation is available at [https://docs.skelfresearch.com/sigc](https://docs.skelfresearch.com/sigc)

## License

MIT License - see [LICENSE](../../LICENSE) for details.
