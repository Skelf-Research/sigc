# Python API Detailed Reference

Complete function and class reference for pysigc.

## Module: pysigc

### run()

Run a backtest.

```python
def run(
    strategy: str,
    params: dict = None,
    config: str = None,
    start: str = None,
    end: str = None,
    verbose: bool = False
) -> Results:
```

**Parameters:**
- `strategy`: Path to .sig file
- `params`: Parameter overrides
- `config`: Path to config file
- `start`: Start date (YYYY-MM-DD)
- `end`: End date (YYYY-MM-DD)
- `verbose`: Print progress

**Returns:** `Results` object

**Example:**
```python
results = pysigc.run(
    "momentum.sig",
    params={"lookback": 60},
    start="2020-01-01"
)
```

### optimize()

Parameter optimization.

```python
def optimize(
    strategy: str,
    params: dict,
    metric: str = "sharpe",
    parallel: bool = True
) -> OptimizationResults:
```

**Parameters:**
- `strategy`: Path to .sig file
- `params`: Parameter ranges (dict of lists)
- `metric`: Optimization target
- `parallel`: Use parallel execution

**Returns:** `OptimizationResults` object

### walk_forward()

Walk-forward validation.

```python
def walk_forward(
    strategy: str,
    train_years: int,
    test_years: int,
    params: dict = None,
    metric: str = "sharpe"
) -> WalkForwardResults:
```

### compute_signals()

Compute signal values without backtesting.

```python
def compute_signals(
    strategy: str,
    data: dict = None
) -> dict:
```

**Returns:** Dictionary of signal name → DataFrame

### load_data()

Load data file.

```python
def load_data(path: str) -> pd.DataFrame:
```

## Class: Results

### Properties

```python
class Results:
    # Return metrics
    total_return: float
    annual_return: float
    excess_return: float

    # Risk metrics
    annual_volatility: float
    max_drawdown: float
    avg_drawdown: float
    var_95: float
    cvar_95: float

    # Risk-adjusted
    sharpe_ratio: float
    sortino_ratio: float
    calmar_ratio: float
    information_ratio: float

    # Trading metrics
    annual_turnover: float
    win_rate: float
    profit_factor: float

    # Time series
    weights: pd.DataFrame
    daily_returns: pd.Series
    positions: pd.DataFrame
    trades: pd.DataFrame
```

### Methods

```python
# Export
def to_csv(self, path: str) -> None
def to_parquet(self, path: str) -> None
def to_dict(self) -> dict

# Analysis
def factor_attribution(self) -> dict
def regime_performance(self) -> dict

# Display
def summary(self) -> str
def plot(self) -> None
```

## Class: OptimizationResults

```python
class OptimizationResults:
    best_params: dict
    best_metric: float
    all_results: pd.DataFrame

    def plot_heatmap(self, x: str, y: str) -> None
```

## Class: WalkForwardResults

```python
class WalkForwardResults:
    in_sample_sharpe: float
    out_of_sample_sharpe: float
    degradation: float
    period_results: pd.DataFrame

    def plot_periods(self) -> None
```

## Display Module

```python
from pysigc import display

# Summary
display.summary(results)

# Plots
display.plot_cumulative(results)
display.plot_drawdown(results)
display.plot_monthly_returns(results)
display.plot_rolling_sharpe(results, window=252)

# Dashboard
display.dashboard(results)
```

## Error Handling

```python
from pysigc import (
    SigcError,
    ParseError,
    DataError,
    RuntimeError
)

try:
    results = pysigc.run("strategy.sig")
except ParseError as e:
    print(f"Syntax error: {e}")
except DataError as e:
    print(f"Data error: {e}")
except SigcError as e:
    print(f"Error: {e}")
```

## Configuration

```python
import pysigc

# Set defaults
pysigc.config.set_default_workers(8)
pysigc.config.set_log_level("debug")

# Get version
print(pysigc.__version__)
```

## See Also

- [Python API Overview](index.md)
- [Tutorials](../../tutorials/python-workflow.md)
