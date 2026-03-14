# Getting Started

Welcome to sigc! This section will help you get up and running quickly.

## Choose Your Path

<div class="path-cards" markdown>

<div class="path-card" markdown>

### Quick Start (5 minutes)

Already familiar with quant concepts? Jump straight in:

1. [Install sigc](installation.md)
2. [Run your first backtest](quickstart.md)

</div>

<div class="path-card" markdown>

### Guided Tutorial (30 minutes)

New to sigc or want a thorough introduction:

1. [Install sigc](installation.md)
2. [Build your first strategy](first-strategy.md)
3. [Set up your IDE](ide-setup.md)

</div>

</div>

## What You'll Learn

By the end of this section, you'll be able to:

- [x] Install the sigc binary on your system
- [x] Write a basic trading signal in the sigc DSL
- [x] Run a backtest and interpret the results
- [x] Set up VS Code for sigc development
- [x] Load and work with sample data

## Prerequisites

Before starting, ensure you have:

- **Operating System**: Linux, macOS, or Windows (WSL2)
- **Rust Toolchain**: Version 1.70 or later
- **Basic Knowledge**: Familiarity with command-line tools

!!! note "No Rust experience required"
    You don't need to know Rust to use sigc. The DSL is a separate language designed for quantitative research. Rust knowledge is only needed if you want to contribute to sigc itself or use the Rust API directly.

## Section Overview

| Page | Description | Time |
|------|-------------|------|
| [Installation](installation.md) | Install Rust and build sigc | 10 min |
| [Quickstart](quickstart.md) | Run your first backtest | 5 min |
| [First Strategy](first-strategy.md) | Build a complete momentum strategy | 15 min |
| [IDE Setup](ide-setup.md) | Configure VS Code extension | 5 min |
| [Sample Data](sample-data.md) | Work with included datasets | 5 min |

## Quick Reference

### Essential Commands

```bash
# Compile a signal (validate syntax)
sigc compile strategy.sig

# Run a backtest
sigc run strategy.sig

# Export results to JSON
sigc run strategy.sig --output results.json

# Compare two strategies
sigc diff strategy_a.sig strategy_b.sig
```

### Basic Signal Structure

```sig
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal momentum:
  returns = ret(prices, lookback)
  emit zscore(returns)

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Next Steps

Ready to begin? Start with [Installation](installation.md).

If you prefer learning by example, check out the [Strategy Library](../strategies/index.md) for 23 complete, documented strategies.
