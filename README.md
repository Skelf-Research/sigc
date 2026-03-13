# sigc

`sigc` is a Rust-first research platform that lets quantitative teams prototype, validate, and operationalize alpha ideas with the speed of an interactive DSL and the determinism of a compiler/runtime pair. The end-state goal is a **single, self-contained binary** that ships the compiler, runtime, and orchestration surface so researchers get a turnkey “compile + run” loop without juggling services.

## Why it exists

Quant shops lose hours every week to ad hoc Python notebooks: mismatched calendars, fragile joins, non-deterministic backtests, and duplicated factor code. Every major institutional desk that scaled past a handful of researchers eventually built a typed, reproducible signal language. `sigc` is our open implementation of that idea — a composable, shape-aware DSL backed by a columnar runtime and deterministic backtester.

## What the system looks like

- **Single binary (`sigc`)** bundles compiler, runtime, daemon, and CLI personas so deployments stay simple while still exposing subcommands (`sigc compile`, `sigc run`, `sigc daemon`, etc.).
- **Compiler module** parses the DSL into a typed IR, optimizes it, and produces executable plans that can be cached and reused.
- **Runtime module** executes plans against columnar data (Arrow/Polars), parallelized with Rayon and SIMD kernels for heavy factor math.
- **Daemon mode** serves `Compile+Run` requests over nng for clients that prefer a long-lived service.
- **CLI/REPL mode** offers DSL editing, hot reload, helpful errors, and artifact inspection.
- **Caching** with sled + blake3 keeps both plan compilation and materialized panels fast and reproducible.
- **Adapters** let BacktestPlans execute on the builtin engine, PyO3 bridges, C ABI stubs, or RPC clients — without touching the core.

## Getting started

1. **Install Rust toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build the binary**
   ```bash
   cargo build --release
   ```

3. **Compile a signal**
   ```bash
   ./target/release/sigc compile examples/momentum.sig
   ```

4. **Run a backtest**
   ```bash
   ./target/release/sigc run examples/momentum.sig
   ```

5. **Start daemon mode**
   ```bash
   ./target/release/sigc daemon
   ```

## DSL Example

```
data:
  prices: load parquet from "data/prices.parquet"

params:
  lookback = 10
  top_pct = 0.2

signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  cleaned = winsor(score, p=0.01)
  emit cleaned

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)
  backtest from 2024-01-01 to 2024-12-31
```

## Available Operators

**Time-series**: `ret`, `lag`, `diff`, `rolling_mean`, `rolling_std`, `rolling_sum`, `rolling_min`, `rolling_max`, `rsi`, `macd`, `atr`, `vwap`

**Cross-sectional**: `zscore`, `rank`, `winsor`, `demean`, `scale`, `quantile`, `bucket`, `median`, `mad`

**Data handling**: `abs`, `sqrt`, `floor`, `ceil`, `round`, `is_nan`, `fill_nan`, `coalesce`, `cumsum`, `cumprod`

**Portfolio**: `long_short`, `neutralize`, `clip`

## Advanced Features

### Walk-Forward Optimization
```rust
use sig_runtime::{WalkForward, WalkForwardConfig};

let config = WalkForwardConfig::new(252, 126, 21); // total, train, test
let mut wf = WalkForward::new(config);
wf.add_range("period", 5.0, 30.0, 5.0);

let result = wf.run(&ir, &mut runtime)?;
println!("Efficiency ratio: {:.2}%", result.efficiency_ratio * 100.0);
```

### Transaction Costs
```rust
use sig_runtime::{CostModel, ImpactModel};

let model = CostModel::institutional()
    .with_impact(ImpactModel::SquareRoot { coefficient: 0.05 });

let cost = model.calculate_cost(100000.0, Some(1000000.0), false, 21.0);
```

### Universe Management
```rust
use sig_runtime::{Universe, UniverseManager};

let manager = UniverseManager::new().with_builtins();
let sp500 = manager.get("SP500").unwrap();
let tech = sp500.by_sector("Technology");
```

### Visualization
```rust
use sig_runtime::ReportVisualizer;

let visualizer = ReportVisualizer::new();
visualizer.save_html(&report, &returns, "report.html")?;
```

## Project status

**Completed:**
- Phase 1: Foundations (workspace, types, caching, connectors)
- Phase 2: Compiler (parser, type inference, 60+ operators, IR)
- Phase 3: Runtime (data loading, 60+ kernels, backtester, Panel data, GridSearch, PyO3)
- Phase 4: Services (daemon mode, PyO3 adapter)

**In Progress:**
- Phase 5: Quality bar (integration tests, benchmarks, reporting, CI/CD, documentation)

See [`docs/build-roadmap.md`](docs/build-roadmap.md) for the full implementation roadmap.

## Documentation

- High-level specs: [`specs.md`](specs.md)
- Build plan and milestones: [`docs/build-roadmap.md`](docs/build-roadmap.md)
- Sample strategies: [`examples/momentum.sig`](examples/momentum.sig), [`examples/meanreversion.sig`](examples/meanreversion.sig), [`examples/combo.sig`](examples/combo.sig)
- Python front door (planned): thin PyO3 API for notebook workflows, see roadmap milestone M7
- OSS contribution guidelines: (planned)

## Contributing

This repository is in active design. Please open an issue or discussion with your context before submitting large PRs so we can keep the roadmap coherent.

## License

License TBD (likely Apache-2.0 or MIT once legal clears). For now, treat the repo as all-rights-reserved until we formalize the terms.
