# Python Integration

Use sigc from Python notebooks with pysigc.

## Installation

```bash
# Build the Python bindings
cd crates/pysigc
maturin develop --release
```

## Basic Usage

```python
import pysigc

# Compile a signal
source = """
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 20

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"""

# Compile
compiled = pysigc.compile(source)
print(f"Compiled {compiled.node_count} nodes")

# Run backtest
result = pysigc.backtest(source)
print(f"Sharpe: {result.sharpe_ratio:.2f}")
print(f"Return: {result.total_return * 100:.1f}%")
```

## API Reference

### compile(source: str) -> CompiledSignal

Compile DSL source to IR.

```python
compiled = pysigc.compile(source)
compiled.node_count      # Number of IR nodes
compiled.output_count    # Number of outputs
compiled.source_hash     # Hash for caching
```

### backtest(source: str) -> BacktestResult

Compile and run backtest.

```python
result = pysigc.backtest(source)
result.total_return      # Cumulative return
result.annualized_return # Annualized return
result.sharpe_ratio      # Risk-adjusted return
result.max_drawdown      # Worst peak-to-trough
result.turnover          # Annual turnover
```

## Notebook Workflow

```python
import pysigc
import pandas as pd
import matplotlib.pyplot as plt

# Define strategy
source = """
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = {lookback}

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
"""

# Parameter sweep
results = []
for lookback in [10, 20, 30, 40, 60]:
    code = source.format(lookback=lookback)
    result = pysigc.backtest(code)
    results.append({
        'lookback': lookback,
        'sharpe': result.sharpe_ratio,
        'return': result.total_return,
        'drawdown': result.max_drawdown
    })

# Analyze
df = pd.DataFrame(results)
print(df.to_string())

# Plot
df.plot(x='lookback', y='sharpe', kind='bar')
plt.title('Sharpe by Lookback')
plt.show()
```

## Error Handling

```python
try:
    result = pysigc.backtest(source)
except Exception as e:
    print(f"Error: {e}")
```

## Integration with Pandas

Load results into pandas for analysis:

```python
import json

# Export to JSON
result = pysigc.backtest(source)
metrics = {
    'total_return': result.total_return,
    'sharpe': result.sharpe_ratio,
    'drawdown': result.max_drawdown,
    'turnover': result.turnover
}

# Convert to DataFrame
df = pd.DataFrame([metrics])
```

## Limitations

Current pysigc limitations:
- No direct DataFrame input (use CSV files)
- No streaming results
- No custom Python operators

For advanced use cases, use the Rust API directly.
