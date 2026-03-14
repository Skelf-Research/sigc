# Python API Reference

pysigc: Python bindings for sigc.

## Installation

```bash
pip install pysigc
```

## Quick Start

```python
import pysigc

# Run a strategy
results = pysigc.run("momentum.sig")

# Access metrics
print(f"Sharpe: {results.sharpe_ratio:.2f}")
print(f"Return: {results.total_return:.1%}")
```

## Running Strategies

### Basic Run

```python
results = pysigc.run("strategy.sig")
```

### With Parameters

```python
results = pysigc.run(
    "strategy.sig",
    params={
        "lookback": 60,
        "top_pct": 0.2
    }
)
```

### With Date Range

```python
results = pysigc.run(
    "strategy.sig",
    start="2020-01-01",
    end="2024-12-31"
)
```

### With Configuration

```python
results = pysigc.run(
    "strategy.sig",
    config="config.yaml"
)
```

## Accessing Results

### Metrics

```python
# Returns
total_return = results.total_return
annual_return = results.annual_return

# Risk
volatility = results.annual_volatility
max_drawdown = results.max_drawdown

# Risk-adjusted
sharpe = results.sharpe_ratio
sortino = results.sortino_ratio
```

### As DataFrames

```python
import pandas as pd

# Weights over time
weights: pd.DataFrame = results.weights

# Daily returns
returns: pd.Series = results.daily_returns

# Positions
positions: pd.DataFrame = results.positions
```

## Parameter Optimization

### Grid Search

```python
results = pysigc.optimize(
    "strategy.sig",
    params={
        "lookback": [20, 40, 60, 80],
        "top_pct": [0.15, 0.20, 0.25]
    },
    metric="sharpe"
)

print(f"Best params: {results.best_params}")
print(f"Best Sharpe: {results.best_metric:.2f}")
```

### Walk-Forward

```python
results = pysigc.walk_forward(
    "strategy.sig",
    train_years=5,
    test_years=1,
    params={
        "lookback": [40, 60, 80]
    }
)

print(f"Out-of-sample Sharpe: {results.oos_sharpe:.2f}")
```

## Signal Computation

### Get Signal Values

```python
signals = pysigc.compute_signals("strategy.sig")

momentum = signals["momentum"]  # pd.DataFrame
print(momentum.head())
```

### Custom Signal

```python
# Compute signal for custom data
import pandas as pd

prices = pd.read_parquet("prices.parquet")
signal = pysigc.compute_signal(
    "zscore(ret(prices, 60))",
    data={"prices": prices}
)
```

## Data Handling

### Load Data

```python
data = pysigc.load_data("prices.parquet")
```

### Convert Formats

```python
pysigc.convert("prices.csv", "prices.parquet")
```

## Jupyter Integration

### Display Results

```python
from pysigc import display

# Pretty print results
display.summary(results)

# Plot performance
display.plot_cumulative(results)
display.plot_drawdown(results)
```

### Interactive Dashboard

```python
display.dashboard(results)
```

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `run(strategy, **kwargs)` | Run backtest |
| `optimize(strategy, params, metric)` | Parameter optimization |
| `walk_forward(strategy, ...)` | Walk-forward validation |
| `compute_signals(strategy)` | Get signal values |
| `load_data(path)` | Load data file |

### Results Object

| Attribute | Type | Description |
|-----------|------|-------------|
| `sharpe_ratio` | float | Sharpe ratio |
| `total_return` | float | Total return |
| `max_drawdown` | float | Maximum drawdown |
| `weights` | DataFrame | Weight time series |
| `daily_returns` | Series | Daily returns |

## See Also

- [Rust API](../rust/index.md)
- [Tutorials](../../tutorials/python-workflow.md)
