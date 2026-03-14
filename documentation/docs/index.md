# sigc

<div class="hero" markdown>

**A Rust-based quantitative finance research platform**

Build, test, and deploy trading strategies with a type-safe DSL, deterministic backtester, and production-ready infrastructure.

[Get Started](getting-started/quickstart.md){ .md-button .md-button--primary }
[View on GitHub](https://github.com/skelf-Research/sigc){ .md-button }

</div>

## Why sigc?

Quant teams lose hours every week to ad hoc Python notebooks: mismatched calendars, fragile joins, non-deterministic backtests, and duplicated factor code. **sigc** is a composable, shape-aware DSL backed by a columnar runtime and deterministic backtester.

<div class="feature-grid" markdown>

<div class="feature-card" markdown>
### Type-Safe DSL
Write signals in a concise, readable language with compile-time type checking. Catch errors before running expensive backtests.
</div>

<div class="feature-card" markdown>
### Deterministic Backtesting
Get reproducible results every time. Content-addressed caching ensures identical inputs produce identical outputs.
</div>

<div class="feature-card" markdown>
### 120+ Operators
Time-series, cross-sectional, technical indicators, and portfolio construction operators built-in. No external dependencies needed.
</div>

<div class="feature-card" markdown>
### Production Ready
Circuit breakers, position limits, Slack alerts, audit logging, and Prometheus metrics. Deploy with confidence.
</div>

<div class="feature-card" markdown>
### Single Binary
No services to manage. Compile, run, and deploy with one `sigc` binary. Daemon mode available for long-lived services.
</div>

<div class="feature-card" markdown>
### IDE Support
VS Code extension with syntax highlighting, code completion, error diagnostics, and 25+ code snippets.
</div>

</div>

## Quick Example

```sig
data:
  prices: load parquet from "data/prices.parquet"

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

```bash
$ sigc run strategy.sig

=== Backtest Results ===
Total Return:         15.23%
Annualized Return:    15.23%
Sharpe Ratio:          1.45
Max Drawdown:          8.12%
Turnover:            312.00%
```

## Choose Your Path

<div class="path-cards" markdown>

<div class="path-card" markdown>
### For Quant Researchers

Learn to build and test trading strategies with sigc's DSL.

- [5-Minute Quickstart](getting-started/quickstart.md)
- [What Are Signals?](concepts/signals.md)
- [Operators Reference](operators/index.md)
- [Strategy Library](strategies/index.md)
- [Quant Guide](quant-guide/index.md)

</div>

<div class="path-card" markdown>
### For Developers

Integrate sigc into your infrastructure and extend its capabilities.

- [Installation Guide](getting-started/installation.md)
- [Architecture Overview](concepts/architecture.md)
- [CLI Reference](reference/cli.md)
- [Rust API](api/rust/index.md)
- [Python Integration](integrations/python.md)

</div>

</div>

## Key Features

### Signal Development

Write trading signals using a concise DSL with 120+ built-in operators:

=== "Momentum"

    ```sig
    signal momentum:
      returns = ret(prices, 252)
      skip = ret(prices, 21)
      mom = returns - skip
      emit zscore(mom)
    ```

=== "Mean Reversion"

    ```sig
    signal mean_reversion:
      ma = rolling_mean(prices, 20)
      std = rolling_std(prices, 20)
      z = (prices - ma) / std
      emit -zscore(z)
    ```

=== "Multi-Factor"

    ```sig
    signal combo:
      mom = zscore(ret(prices, 60))
      vol = -zscore(rolling_std(ret(prices, 1), 20))
      emit 0.6 * mom + 0.4 * vol
    ```

### Custom Functions and Macros

Create reusable components for your signal library:

```sig
// Custom function
fn volatility(x, window=20):
  rolling_std(ret(x, 1), window)

// Reusable macro
macro vol_adj_momentum(px: expr, ret_window: number = 20, vol_window: number = 60):
  let r = ret(px, ret_window)
  let vol = rolling_std(r, vol_window)
  emit zscore(r / vol)

signal my_signal:
  emit vol_adj_momentum(prices, 30, 90)
```

### Production Infrastructure

Deploy with enterprise-grade safety systems:

- **Circuit Breakers** - Automatic trading halts on anomalies
- **Position Limits** - Max weight, sector, and turnover constraints
- **Kill Switches** - Emergency stop for all trading
- **Alerting** - Slack, email, and console notifications
- **Audit Logging** - Full compliance trail
- **Monitoring** - Prometheus metrics and Grafana dashboards

### Integrations

Connect to your existing infrastructure:

| Integration | Description |
|-------------|-------------|
| **Python** | `pysigc` bindings for Jupyter notebooks |
| **VS Code** | Syntax highlighting, LSP, and snippets |
| **Alpaca** | Paper and live trading via broker API |
| **Yahoo Finance** | Free historical and real-time data |
| **PostgreSQL** | Async connection pooling for databases |
| **S3** | Cloud storage for data and results |

## Project Status

sigc is feature-complete with all 8 development phases finished:

- [x] Production Features (async DB, corporate actions, alerts)
- [x] Performance & Scale (SIMD kernels, memory mapping)
- [x] Integration & Polish (colored CLI, 330 tests)
- [x] Documentation (11-chapter Quant Guide, 23 strategies)
- [x] Advanced Analytics (factor models, regime detection)
- [x] Safety & Deployment (circuit breakers, Docker)
- [x] Language Enhancements (macros, type inference, LSP)
- [x] External Integrations (Yahoo Finance, Alpaca)

See the [Changelog](changelog.md) for release history.

## Getting Help

- **Documentation**: You're reading it!
- **GitHub Issues**: [Report bugs or request features](https://github.com/skelf-Research/sigc/issues)
- **Discussions**: [Ask questions and share strategies](https://github.com/skelf-Research/sigc/discussions)

---

<div style="text-align: center; margin-top: 2rem;">

**Ready to build your first strategy?**

[Get Started](getting-started/quickstart.md){ .md-button .md-button--primary }

</div>
