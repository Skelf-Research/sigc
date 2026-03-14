# sigc Documentation

Welcome to the sigc documentation. This guide covers everything from getting started to advanced usage.

## Getting Started

New to sigc? Start here:

- **[Quickstart](getting-started/quickstart.md)** - Run your first signal in 5 minutes
- **[DSL Basics](guide/dsl-basics.md)** - Learn the language syntax
- **[Operators Reference](reference/operators-table.md)** - All available functions

## Guides

Core concepts and usage:

- [DSL Basics](guide/dsl-basics.md) - Language syntax and structure
- [CLI Reference](guide/cli-reference.md) - All command-line options
- [Data Loading](guide/data-loading.md) - CSV, Parquet, S3, PostgreSQL sources
- Portfolio Construction *(coming soon)* - Weight calculation
- Backtesting *(coming soon)* - Running simulations

## Tutorials

Step-by-step learning:

- **[Momentum Strategy](tutorials/momentum-strategy.md)** - Build a classic factor
- Mean Reversion *(coming soon)* - Contrarian signals
- Multi-Factor *(coming soon)* - Combining alphas
- Parameter Optimization *(coming soon)* - Finding best params

## Advanced Topics

For power users:

- **[Production Features](advanced/production-features.md)** - Constraints, attribution, parallel execution
- **[Cost Models](advanced/cost-models.md)** - Transaction costs and slippage
- **[Python Integration](advanced/python-integration.md)** - pysigc notebooks
- Walk-Forward Validation *(coming soon)* - Out-of-sample testing
- Universe Management *(coming soon)* - Sectors and filters
- Daemon Mode *(coming soon)* - Service deployment

## Reference

Quick lookups:

- **[Operators Table](reference/operators-table.md)** - Complete operator list
- Error Messages *(coming soon)* - Common errors and fixes
- Configuration *(coming soon)* - Environment settings

## Examples

Working code:

- [Sample Data](examples/data/sample_prices.csv) - Price CSV for testing
- Example signals in `examples/` directory

## Quick Reference

### Basic Signal

```
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal momentum:
  r = ret(prices, lookback)
  emit zscore(r)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

### Common Commands

```bash
# Compile
sigc compile strategy.sig

# Run backtest
sigc run strategy.sig

# Export results
sigc run strategy.sig --output results.json

# Compare strategies
sigc diff strategy_a.sig strategy_b.sig

# Inspect IR
sigc explain strategy.sig
```

### Top Operators

| Category | Operators |
|----------|-----------|
| Time-series | `ret`, `lag`, `rolling_mean`, `rolling_std`, `ema` |
| Cross-sectional | `zscore`, `rank`, `winsor`, `demean`, `scale` |
| Portfolio | `long_short`, `neutralize` |
| Technical | `rsi`, `macd`, `atr`, `vwap` |

## Getting Help

- Run `sigc --help` for CLI help
- Check [Error Messages](reference/operators-table.md#common-issues) for troubleshooting
- Open issues at [GitHub](https://github.com/anthropics/sigc/issues)

## Roadmap

See the **[Production Roadmap](ROADMAP.md)** for planned features and implementation phases.

## Contributing

See the main [README](../README.md) for contribution guidelines.
