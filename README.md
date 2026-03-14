# sigc

**The Quant's Compiler: From Alpha Idea to Production in Minutes**

[![Crates.io](https://img.shields.io/crates/v/sigc.svg)](https://crates.io/crates/sigc)
[![Documentation](https://img.shields.io/badge/docs-skelfresearch.com-blue)](https://docs.skelfresearch.com/sigc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Downloads](https://img.shields.io/crates/d/sigc.svg)](https://crates.io/crates/sigc)

*A type-safe DSL and high-performance runtime for quantitative trading strategies.*

*Write signals like sentences. Backtest in milliseconds. Deploy with confidence.*

[Documentation](https://docs.skelfresearch.com/sigc) | [Crates.io](https://crates.io/crates/sigc) | [GitHub](https://github.com/skelf-Research/sigc)

---

## Why sigc?

Every quant team that scales past a handful of researchers eventually builds the same thing: a typed, reproducible signal language. We've seen it at every major desk. **sigc** is that system — open-sourced.

| Problem | sigc Solution |
|---------|---------------|
| Ad-hoc notebooks with mismatched calendars | Type-safe DSL with compile-time shape checking |
| Non-deterministic backtests | Content-addressed caching, identical inputs = identical outputs |
| Fragile pandas joins at 3am | 120+ vectorized operators, SIMD-optimized |
| "It worked in research" → fails in prod | Same binary: `sigc run` → `sigc daemon` |
| Factor code copy-pasted across teams | Macros, functions, importable signal libraries |

## Quick Start

```bash
# Install from crates.io
cargo install sigc

# Or build from source
git clone https://github.com/skelf-Research/sigc.git
cd sigc && cargo build --release
```

Create `momentum.sig`:

```sig
data:
  prices: load parquet from "prices.parquet"

params:
  lookback = 20
  top_pct = 0.2

signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit winsor(score, p=0.01)

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)
  backtest from 2024-01-01 to 2024-12-31
```

Run it:

```bash
$ sigc run momentum.sig

=== Backtest Results ===
Total Return:      15.23%
Sharpe Ratio:       1.45
Max Drawdown:       8.12%
Turnover:         312.00%
```

## For Quants: Express Alpha, Not Boilerplate

Write signals that read like your research notes:

```sig
// Momentum with skip-month
signal momentum:
  total_ret = ret(prices, 252)
  skip_ret = ret(prices, 21)
  mom = total_ret - skip_ret
  emit zscore(mom)

// Mean reversion on residuals
signal stat_arb:
  beta = rolling_beta(stock, market, 60)
  residual = stock - beta * market
  z = (residual - rolling_mean(residual, 20)) / rolling_std(residual, 20)
  emit -z

// Combine factors with explicit weights
signal multi_factor:
  emit 0.4 * momentum + 0.3 * value + 0.3 * quality
```

**120+ operators** built-in: `zscore`, `rank`, `rolling_mean`, `ema`, `rsi`, `macd`, `atr`, `vwap`, `neutralize`, `winsor`, and more.

## For Developers: Rust-Native, Zero Runtime Overhead

```rust
use sigc::{Strategy, Backtest};

// Embed in your systems
let strategy = Strategy::from_file("alpha.sig")?;
let results = strategy
    .with_param("lookback", 60)
    .with_param("threshold", 1.5)
    .run()?;

println!("Sharpe: {:.2}", results.sharpe_ratio());

// Access raw data
let weights: Vec<f64> = results.weights();
let returns: Vec<f64> = results.daily_returns();
```

**Architecture:**
- **Single binary** — compiler, runtime, daemon, CLI in one executable
- **Polars/Arrow** columnar execution with Rayon parallelism
- **SIMD kernels** for factor math (AVX2/AVX-512 where available)
- **Content-addressed cache** (sled + blake3) for reproducibility
- **nng RPC** for daemon mode, sub-millisecond latency

## For Architects: Production-Grade Infrastructure

```yaml
# sigc.yaml - Production configuration
mode: production

safety:
  circuit_breakers:
    max_drawdown: 0.15
    max_position: 0.10
    kill_switch: true

  rate_limits:
    orders_per_minute: 100

alerts:
  slack:
    webhook: ${SLACK_WEBHOOK}
    channels: ["#trading-alerts"]

schedule:
  jobs:
    - name: rebalance
      cron: "0 9 * * 1-5"
      strategy: momentum
```

**Enterprise features:**
- Circuit breakers with automatic position flattening
- Prometheus metrics (`/metrics` endpoint)
- Structured audit logging (JSON, compliance-ready)
- Slack/email alerting on anomalies
- Docker & Kubernetes ready (`ghcr.io/skelf-Research/sigc`)

## Crate Ecosystem

| Crate | Purpose |
|-------|---------|
| [`sigc`](https://crates.io/crates/sigc) | CLI and main entry point |
| [`sig_compiler`](https://crates.io/crates/sig_compiler) | DSL parser and type checker |
| [`sig_runtime`](https://crates.io/crates/sig_runtime) | Execution engine with 120+ operators |
| [`sig_types`](https://crates.io/crates/sig_types) | Core type definitions |
| [`sig_cache`](https://crates.io/crates/sig_cache) | Deterministic caching layer |
| [`sig_lsp`](https://crates.io/crates/sig_lsp) | Language server for IDE support |

## IDE Support

**VS Code Extension** with full language server:
- Real-time error diagnostics
- Hover documentation for all operators
- Code completion with signatures
- Go-to-definition for signals/functions
- 25+ snippets for common patterns

```bash
code --install-extension skelf-Research.sigc-vscode
```

## Integrations

| Integration | Status | Description |
|-------------|--------|-------------|
| **Alpaca** | ✅ | Paper & live trading |
| **Yahoo Finance** | ✅ | Free market data |
| **PostgreSQL** | ✅ | Async connection pooling |
| **S3/GCS** | ✅ | Cloud data sources |
| **Python** | ✅ | `pysigc` bindings for notebooks |

## Performance

Benchmarked on 5 years of daily data, 500 securities:

| Operation | Time |
|-----------|------|
| Parse + compile | 2ms |
| Full backtest | 45ms |
| Incremental update | 3ms |

Memory-mapped data loading. Warm cache hits in microseconds.

## Documentation

- **[Full Documentation](https://docs.skelfresearch.com/sigc)** — Tutorials, API reference, deployment guides
- **[Quant Guide](https://docs.skelfresearch.com/sigc/quant-guide/)** — 9-chapter deep dive for researchers
- **[Strategy Library](https://docs.skelfresearch.com/sigc/strategies/)** — 23 ready-to-use strategies
- **[Operator Reference](https://docs.skelfresearch.com/sigc/operators/)** — All 120+ operators documented

## Contributing

We welcome contributions. Please open an issue to discuss before submitting large PRs.

```bash
# Development setup
git clone https://github.com/skelf-Research/sigc.git
cd sigc
cargo build
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License — see [LICENSE](LICENSE) for details.

---

**Built by quants, for quants.**

[Get Started](https://docs.skelfresearch.com/sigc/getting-started/quickstart/) | [View Strategies](https://docs.skelfresearch.com/sigc/strategies/) | [Join Discussions](https://github.com/skelf-Research/sigc/discussions)

*Skelf Research — [skelfresearch.com](https://skelfresearch.com)*
