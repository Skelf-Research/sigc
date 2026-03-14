# sig_runtime

High-performance columnar runtime for sigc trading signal execution.

## Overview

`sig_runtime` executes compiled sigc strategies with:

- **Columnar execution** - Built on Polars/Arrow for vectorized operations
- **120+ operators** - Time-series, cross-sectional, technical indicators
- **Parallel execution** - Multi-threaded with Rayon
- **SIMD kernels** - Optimized for modern CPUs
- **Data loading** - CSV, Parquet, S3, PostgreSQL support

## Operators

**Time-series**: `ret`, `lag`, `diff`, `rolling_mean`, `rolling_std`, `ema`, `rsi`, `macd`, `atr`

**Cross-sectional**: `zscore`, `rank`, `winsor`, `demean`, `scale`, `quantile`

**Portfolio**: `long_short`, `neutralize`, `clip`

## Usage

```rust
use sig_runtime::Runtime;
use sig_compiler::Compiler;

let compiler = Compiler::new();
let ir = compiler.compile(source)?;

let mut runtime = Runtime::new()?;
let results = runtime.execute(&ir)?;

println!("Sharpe: {:.2}", results.sharpe_ratio());
```

## Part of sigc

This crate is part of the [sigc](https://github.com/skelf-Research/sigc) quantitative finance platform.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
