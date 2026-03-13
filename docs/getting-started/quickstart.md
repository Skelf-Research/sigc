# Quickstart

Get your first signal running in 5 minutes.

## Prerequisites

- Rust 1.70+ installed
- Built sigc binary (`cargo build --release`)

## Step 1: Create Sample Data

Create a file `data/prices.csv` with daily price data:

```csv
date,AAPL,MSFT,GOOGL,AMZN,META
2024-01-02,185.64,374.58,140.25,151.94,346.29
2024-01-03,184.25,373.31,139.12,149.93,344.47
2024-01-04,181.91,367.94,137.98,147.44,343.19
2024-01-05,181.18,367.75,136.69,148.47,347.12
...
```

A complete sample file is available at `docs/examples/data/sample_prices.csv`.

## Step 2: Write Your First Signal

Create `my_signal.sig`:

```
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit score

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Step 3: Compile

Validate your signal compiles correctly:

```bash
./target/release/sigc compile my_signal.sig
```

Output:
```
INFO sigc: sigc v0.1.0
INFO sig_compiler: Parsing source
INFO sig_compiler: Parsed 1 data, 1 params, 1 signals, 1 portfolios
INFO sig_compiler: Lowered to 5 IR nodes
INFO sigc: Compilation complete: 5 nodes
```

## Step 4: Run Backtest

Execute the backtest:

```bash
./target/release/sigc run my_signal.sig
```

Output:
```
=== Backtest Results ===
Total Return:         12.45%
Annualized Return:    12.45%
Sharpe Ratio:          1.23
Max Drawdown:          8.76%
Turnover:            245.00%
```

## Step 5: Export Results

Save results to JSON:

```bash
./target/release/sigc run my_signal.sig --output results.json
```

## What Just Happened?

1. **Data loaded**: Prices were read from CSV
2. **Signal computed**: For each asset, computed 20-day returns then z-scored
3. **Weights assigned**: Cross-sectionally ranked assets, long top 20%, short bottom 20%
4. **Backtest run**: Simulated daily rebalancing with transaction costs
5. **Metrics calculated**: Sharpe, drawdown, turnover

## Next Steps

- [DSL Basics](../guide/dsl-basics.md) - Learn the language
- [Operators Reference](../reference/operators-table.md) - Available functions
- [Momentum Tutorial](../tutorials/momentum-strategy.md) - Build a real strategy

## Common Issues

### "Failed to load CSV"

Check the path is correct and file exists:
```bash
ls -la data/prices.csv
```

### "No data sources found"

Ensure your `data:` block has the correct format:
```
data:
  prices: load csv from "path/to/file.csv"
```

### Low Sharpe ratio

- Check your lookback period
- Ensure enough data history
- Consider transaction costs
