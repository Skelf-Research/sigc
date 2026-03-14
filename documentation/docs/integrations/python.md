# Python Integration

Use sigc from Python with the `pysigc` package.

## Installation

```bash
pip install pysigc
```

## Quick Start

```python
import pysigc

# Run a strategy file
results = pysigc.run("strategy.sig")

# Access results
print(f"Total Return: {results.total_return:.1%}")
print(f"Sharpe Ratio: {results.sharpe_ratio:.2f}")
print(f"Max Drawdown: {results.max_drawdown:.1%}")
```

## Running Strategies

### From File

```python
results = pysigc.run("momentum_strategy.sig")
```

### From String

```python
strategy = """
data:
  source = "prices.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices

signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
"""

results = pysigc.run_string(strategy)
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

### With Custom Data

```python
import pandas as pd

# Your data
df = pd.read_csv("my_prices.csv")

# Run with data
results = pysigc.run(
    "strategy.sig",
    data={"prices": df}
)
```

## Results Object

### Performance Metrics

```python
results = pysigc.run("strategy.sig")

# Return metrics
results.total_return      # Total return
results.cagr              # Compound annual growth rate
results.daily_return      # pandas Series of daily returns

# Risk metrics
results.volatility        # Annualized volatility
results.sharpe_ratio      # Sharpe ratio
results.sortino_ratio     # Sortino ratio
results.max_drawdown      # Maximum drawdown
results.calmar_ratio      # Calmar ratio

# Benchmark metrics (if benchmark specified)
results.alpha             # Alpha
results.beta              # Beta
results.information_ratio # Information ratio
results.tracking_error    # Tracking error
```

### Weights and Positions

```python
# Target weights (pandas DataFrame)
weights = results.weights
# Columns: date, ticker, weight

# Position history
positions = results.positions
# Columns: date, ticker, shares, value

# Current positions
current = results.current_positions()
```

### Signals

```python
# Access computed signals
signals = results.signals
# Dict[str, pd.DataFrame] - signal name to values

momentum = results.signals["momentum"]
# Columns: date, ticker, value
```

### Trades

```python
# Trade history
trades = results.trades
# Columns: date, ticker, side, quantity, price

# Trade statistics
print(f"Total trades: {len(trades)}")
print(f"Turnover: {results.turnover:.1%}")
```

## Jupyter Notebooks

### Display Results

```python
import pysigc

results = pysigc.run("strategy.sig")

# Rich display in Jupyter
results  # Shows formatted summary
```

### Plot Performance

```python
import matplotlib.pyplot as plt

# Plot cumulative returns
results.plot_cumulative_returns()
plt.show()

# Plot drawdown
results.plot_drawdown()
plt.show()

# Plot monthly returns heatmap
results.plot_monthly_returns()
plt.show()
```

### Interactive Analysis

```python
# Export to pandas for custom analysis
returns = results.daily_return
weights = results.weights

# Custom calculations
rolling_sharpe = returns.rolling(60).mean() / returns.rolling(60).std() * np.sqrt(252)
```

## Parameter Optimization

### Grid Search

```python
import pysigc
import pandas as pd

# Define parameter grid
lookbacks = [20, 40, 60, 80, 100]
top_pcts = [0.1, 0.2, 0.3]

results_list = []

for lookback in lookbacks:
    for top_pct in top_pcts:
        result = pysigc.run(
            "strategy.sig",
            params={"lookback": lookback, "top_pct": top_pct}
        )
        results_list.append({
            "lookback": lookback,
            "top_pct": top_pct,
            "sharpe": result.sharpe_ratio,
            "return": result.total_return,
            "drawdown": result.max_drawdown
        })

df = pd.DataFrame(results_list)
print(df.sort_values("sharpe", ascending=False))
```

### Parallel Optimization

```python
from concurrent.futures import ProcessPoolExecutor
import pysigc

def run_with_params(params):
    return pysigc.run("strategy.sig", params=params)

param_sets = [
    {"lookback": 20, "top_pct": 0.2},
    {"lookback": 40, "top_pct": 0.2},
    {"lookback": 60, "top_pct": 0.2},
    # ...
]

with ProcessPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(run_with_params, param_sets))
```

## Data Integration

### Pandas DataFrame

```python
import pandas as pd
import pysigc

# Load your data
prices = pd.read_csv("prices.csv", parse_dates=["date"])

# Run strategy with data
results = pysigc.run(
    "strategy.sig",
    data={"prices": prices}
)
```

### From Database

```python
import pandas as pd
import pysigc
from sqlalchemy import create_engine

# Load from database
engine = create_engine("postgresql://localhost/marketdb")
prices = pd.read_sql(
    "SELECT date, ticker, close FROM daily_prices",
    engine
)

# Run strategy
results = pysigc.run("strategy.sig", data={"prices": prices})
```

### Real-Time Data

```python
import pysigc

# Connect to live data
client = pysigc.LiveClient()

# Subscribe to updates
@client.on_bar
def on_new_bar(bar):
    # Recompute signals
    signals = pysigc.compute_signal("momentum", data=bar)
    print(f"New signal: {signals}")

client.subscribe(["AAPL", "MSFT", "GOOGL"])
client.run()
```

## Integration with ML

### Feature Engineering

```python
import pysigc
import pandas as pd
from sklearn.ensemble import RandomForestClassifier

# Get signal values as features
results = pysigc.run("feature_signals.sig")
signals = results.signals

# Combine signals into feature matrix
features = pd.merge(
    signals["momentum"],
    signals["value"],
    on=["date", "ticker"],
    suffixes=["_mom", "_val"]
)

# Add target (future returns)
features["target"] = features.groupby("ticker")["close"].pct_change(5).shift(-5) > 0

# Train model
X = features[["momentum", "value"]]
y = features["target"]
model = RandomForestClassifier().fit(X, y)
```

### Using Predictions

```python
# Generate predictions
features["ml_signal"] = model.predict_proba(X)[:, 1]

# Use in sigc strategy
strategy = """
signal ml_alpha:
  emit ml_predictions

portfolio main:
  weights = rank(ml_alpha).long_short(top=0.2, bottom=0.2)
"""

results = pysigc.run_string(
    strategy,
    data={"ml_predictions": features[["date", "ticker", "ml_signal"]]}
)
```

## API Reference

### pysigc.run()

```python
pysigc.run(
    file: str,                    # Strategy file path
    params: dict = None,          # Parameter overrides
    data: dict = None,            # Custom data
    config: dict = None,          # Runtime config
    output: str = None            # Output format
) -> Results
```

### pysigc.run_string()

```python
pysigc.run_string(
    strategy: str,                # Strategy code
    params: dict = None,
    data: dict = None,
    config: dict = None
) -> Results
```

### pysigc.compile()

```python
# Just compile (no run)
compiled = pysigc.compile("strategy.sig")
print(compiled.signals)  # List of signals
print(compiled.portfolios)  # List of portfolios
```

### pysigc.validate()

```python
# Validate without running
errors = pysigc.validate("strategy.sig")
if errors:
    for e in errors:
        print(f"{e.line}: {e.message}")
```

## Error Handling

```python
import pysigc
from pysigc.errors import (
    ParseError,
    DataError,
    ComputeError,
    ConfigError
)

try:
    results = pysigc.run("strategy.sig")
except ParseError as e:
    print(f"Syntax error at line {e.line}: {e.message}")
except DataError as e:
    print(f"Data error: {e}")
except ComputeError as e:
    print(f"Computation error: {e}")
except ConfigError as e:
    print(f"Configuration error: {e}")
```

## Performance Tips

### 1. Reuse Compiled Strategies

```python
# Compile once
compiled = pysigc.compile("strategy.sig")

# Run multiple times with different params
for params in param_sets:
    results = compiled.run(params=params)
```

### 2. Use Parquet Data

```python
# Faster than CSV
prices = pd.read_parquet("prices.parquet")
```

### 3. Limit Date Range

```python
results = pysigc.run(
    "strategy.sig",
    config={"start_date": "2023-01-01", "end_date": "2024-01-01"}
)
```

## Best Practices

### 1. Version Your Strategies

```python
import pysigc

# Check version compatibility
print(f"pysigc version: {pysigc.__version__}")
```

### 2. Use Type Hints

```python
from pysigc import Results

def analyze_results(results: Results) -> dict:
    return {
        "sharpe": results.sharpe_ratio,
        "return": results.total_return
    }
```

### 3. Handle Missing Data

```python
results = pysigc.run("strategy.sig")
weights = results.weights.dropna()
```

## Next Steps

- [VSCode](vscode.md) - IDE integration
- [Alpaca](alpaca.md) - Trading execution
- [Tutorials](../tutorials/python-workflow.md) - Python workflow tutorial
