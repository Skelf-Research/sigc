# Rust API Reference

Core sigc Rust library documentation.

## Overview

sigc is written in Rust and can be used as a library:

```rust
use sigc::{Strategy, Backtest, Results};

fn main() -> Result<(), sigc::Error> {
    // Load strategy
    let strategy = Strategy::from_file("momentum.sig")?;

    // Run backtest
    let results = strategy.run()?;

    // Access results
    println!("Sharpe: {:.2}", results.sharpe_ratio());
    println!("Return: {:.1}%", results.annual_return() * 100.0);

    Ok(())
}
```

## Crates

| Crate | Purpose |
|-------|---------|
| `sigc` | Main entry point, CLI |
| `sig_parser` | DSL parser |
| `sig_runtime` | Computation engine |
| `sig_types` | Type definitions |

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
sigc = "0.10"
```

## Quick Start

### Running a Strategy

```rust
use sigc::Strategy;

let strategy = Strategy::from_file("strategy.sig")?;
let results = strategy.run()?;

println!("Sharpe: {:.2}", results.sharpe_ratio());
```

### With Parameters

```rust
let results = Strategy::from_file("strategy.sig")?
    .with_param("lookback", 60)
    .with_param("top_pct", 0.2)
    .run()?;
```

### Accessing Results

```rust
// Metrics
let sharpe = results.sharpe_ratio();
let max_dd = results.max_drawdown();
let annual_ret = results.annual_return();

// Time series
let weights = results.weights();  // Vec<Weight>
let returns = results.daily_returns();  // Vec<f64>
let cumulative = results.cumulative_returns();  // Vec<f64>
```

## Detailed Documentation

- [Strategy Module](strategy.md) - Loading and running strategies
- [Results Module](results.md) - Accessing backtest results
- [Data Module](data.md) - Data loading and manipulation
- [Types Module](types.md) - Core type definitions

## Error Handling

```rust
use sigc::{Strategy, Error};

match Strategy::from_file("strategy.sig") {
    Ok(strategy) => { /* use strategy */ }
    Err(Error::ParseError(e)) => eprintln!("Parse error: {}", e),
    Err(Error::DataError(e)) => eprintln!("Data error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Building from Source

```bash
git clone https://github.com/skelf-Research/sigc
cd sigc
cargo build --release
```

## See Also

- [Python API](../python/index.md)
- [API Overview](../index.md)
