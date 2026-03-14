# API Reference

Programmatic interfaces for sigc.

## Overview

| API | Language | Use Case |
|-----|----------|----------|
| [Rust API](rust/index.md) | Rust | Core development, extensions |
| [Python API](python/index.md) | Python | Research, analysis, ML integration |

## Rust API

The core sigc library is written in Rust and can be used as a library:

```rust
use sigc::{Strategy, Backtest, Results};

// Load and run a strategy
let strategy = Strategy::from_file("momentum.sig")?;
let results = strategy.run()?;

println!("Sharpe: {:.2}", results.sharpe_ratio());
```

### Crates

| Crate | Purpose |
|-------|---------|
| `sigc` | Main entry point |
| `sig_parser` | DSL parser |
| `sig_runtime` | Computation engine |
| `sig_types` | Type definitions |

[Rust API Documentation →](rust/index.md)

## Python API (pysigc)

Python bindings for research and analysis:

```python
import pysigc

# Run a strategy
results = pysigc.run("momentum.sig")
print(f"Sharpe: {results.sharpe_ratio:.2f}")

# Access data
weights = results.weights  # pandas DataFrame
returns = results.daily_return  # pandas Series
```

### Features

- Run strategies from Python
- Access results as pandas DataFrames
- Parameter optimization
- Custom data integration
- Jupyter notebook support

[Python API Documentation →](python/index.md)

## Choosing an API

### Use Rust When

- Building sigc extensions
- Performance-critical applications
- Contributing to sigc core
- Building production systems

### Use Python When

- Research and exploration
- Jupyter notebooks
- Integration with ML frameworks
- Custom analysis

## Quick Comparison

### Running a Strategy

**Rust:**
```rust
use sigc::Strategy;

let strategy = Strategy::from_file("momentum.sig")?;
let results = strategy.run()?;
```

**Python:**
```python
import pysigc

results = pysigc.run("momentum.sig")
```

### Accessing Results

**Rust:**
```rust
let sharpe = results.sharpe_ratio();
let weights = results.weights();  // Vec<Weight>
```

**Python:**
```python
sharpe = results.sharpe_ratio
weights = results.weights  # pandas DataFrame
```

### With Parameters

**Rust:**
```rust
let results = strategy
    .with_param("lookback", 60)
    .with_param("top_pct", 0.2)
    .run()?;
```

**Python:**
```python
results = pysigc.run(
    "momentum.sig",
    params={"lookback": 60, "top_pct": 0.2}
)
```

## API Stability

| Version | Status |
|---------|--------|
| 0.x | API may change |
| 1.0+ | Stable, backwards compatible |

Current version: **0.10.0** (pre-1.0)

## Documentation

- [Rust API](rust/index.md) - Complete Rust documentation
- [Python API](python/index.md) - Complete Python documentation
- [Examples](https://github.com/skelf-Research/sigc/tree/main/examples) - Code examples
