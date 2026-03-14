# Tutorial: Python Workflow

Integrate sigc with Python for advanced analysis and ML.

## Overview

The Python API (pysigc) enables:

- Running strategies from Python
- Analyzing results with pandas
- Machine learning integration
- Custom visualization
- Jupyter notebook workflows

## Installation

### Install pysigc

```bash
pip install pysigc
```

### Verify Installation

```python
import pysigc
print(pysigc.__version__)
```

## Basic Usage

### Run a Strategy

```python
import pysigc

# Run a .sig file
results = pysigc.run("momentum.sig")

# Access metrics
print(f"Sharpe Ratio: {results.sharpe_ratio:.2f}")
print(f"Total Return: {results.total_return:.1%}")
print(f"Max Drawdown: {results.max_drawdown:.1%}")
```

### Access Results as DataFrames

```python
import pandas as pd

# Portfolio weights over time
weights = results.weights  # pd.DataFrame
print(weights.head())

# Daily returns
returns = results.daily_returns  # pd.Series
print(returns.describe())

# Positions
positions = results.positions  # pd.DataFrame
```

## Running with Parameters

### Override Parameters

```python
# Run with specific parameters
results = pysigc.run(
    "momentum.sig",
    params={
        "lookback": 60,
        "top_pct": 0.20,
        "rebal_days": 21
    }
)
```

### Parameter Sweep

```python
import itertools

# Define parameter grid
lookbacks = [20, 40, 60, 80]
top_pcts = [0.15, 0.20, 0.25]

# Run all combinations
results_grid = []

for lb, tp in itertools.product(lookbacks, top_pcts):
    result = pysigc.run(
        "momentum.sig",
        params={"lookback": lb, "top_pct": tp}
    )
    results_grid.append({
        "lookback": lb,
        "top_pct": tp,
        "sharpe": result.sharpe_ratio,
        "return": result.total_return,
        "max_dd": result.max_drawdown
    })

# Analyze
df_results = pd.DataFrame(results_grid)
print(df_results.sort_values("sharpe", ascending=False).head(10))
```

## Data Analysis

### Analyze Returns

```python
import numpy as np

returns = results.daily_returns

# Basic statistics
stats = {
    "Mean (Annual)": returns.mean() * 252,
    "Std (Annual)": returns.std() * np.sqrt(252),
    "Skewness": returns.skew(),
    "Kurtosis": returns.kurtosis(),
    "Sharpe": returns.mean() / returns.std() * np.sqrt(252),
    "Max DD": (returns.cumsum() - returns.cumsum().cummax()).min()
}

for k, v in stats.items():
    print(f"{k}: {v:.4f}")
```

### Rolling Performance

```python
# Rolling Sharpe ratio
rolling_sharpe = (
    returns.rolling(252).mean() / returns.rolling(252).std()
) * np.sqrt(252)

# Plot
import matplotlib.pyplot as plt

plt.figure(figsize=(12, 6))
rolling_sharpe.plot()
plt.title("Rolling 1-Year Sharpe Ratio")
plt.axhline(y=0, color='r', linestyle='--')
plt.ylabel("Sharpe Ratio")
plt.show()
```

### Drawdown Analysis

```python
# Calculate drawdowns
cumulative = (1 + returns).cumprod()
rolling_max = cumulative.cummax()
drawdown = (cumulative - rolling_max) / rolling_max

# Plot
fig, axes = plt.subplots(2, 1, figsize=(12, 8), sharex=True)

axes[0].plot(cumulative)
axes[0].set_title("Cumulative Return")
axes[0].set_ylabel("Value")

axes[1].fill_between(drawdown.index, drawdown.values, 0, alpha=0.5, color='red')
axes[1].set_title("Drawdowns")
axes[1].set_ylabel("Drawdown")

plt.tight_layout()
plt.show()
```

## Visualization

### Performance Dashboard

```python
def plot_performance_dashboard(results):
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))

    # Cumulative returns
    cum_ret = (1 + results.daily_returns).cumprod()
    axes[0, 0].plot(cum_ret)
    axes[0, 0].set_title("Cumulative Return")

    # Monthly returns heatmap
    monthly = results.daily_returns.resample('M').sum()
    monthly_matrix = monthly.values.reshape(-1, 12)[:, :12]
    im = axes[0, 1].imshow(monthly_matrix, cmap='RdYlGn', aspect='auto')
    axes[0, 1].set_title("Monthly Returns")
    plt.colorbar(im, ax=axes[0, 1])

    # Return distribution
    axes[1, 0].hist(results.daily_returns, bins=50, edgecolor='black')
    axes[1, 0].axvline(x=0, color='r', linestyle='--')
    axes[1, 0].set_title("Return Distribution")

    # Rolling volatility
    rolling_vol = results.daily_returns.rolling(60).std() * np.sqrt(252)
    axes[1, 1].plot(rolling_vol)
    axes[1, 1].set_title("Rolling 60-Day Volatility (Annualized)")

    plt.tight_layout()
    plt.show()

plot_performance_dashboard(results)
```

### Weight Analysis

```python
def analyze_weights(weights):
    # Concentration
    concentration = (weights ** 2).sum(axis=1)

    # Long/Short exposure
    long_exposure = weights[weights > 0].sum(axis=1)
    short_exposure = weights[weights < 0].sum(axis=1).abs()

    fig, axes = plt.subplots(2, 1, figsize=(12, 8))

    axes[0].plot(long_exposure, label='Long')
    axes[0].plot(short_exposure, label='Short')
    axes[0].legend()
    axes[0].set_title("Long/Short Exposure Over Time")

    axes[1].plot(concentration)
    axes[1].set_title("Portfolio Concentration (HHI)")

    plt.tight_layout()
    plt.show()

analyze_weights(results.weights)
```

## Machine Learning Integration

### Feature Engineering

```python
import pysigc

# Get signal values
signal_values = pysigc.compute_signals("strategy.sig")

# Extract features
features = pd.DataFrame({
    "momentum": signal_values["momentum"],
    "value": signal_values["value"],
    "quality": signal_values["quality"]
})

# Add forward returns as target
prices = pysigc.load_data("prices.parquet")
forward_ret = prices.pct_change(21).shift(-21)  # 21-day forward return

# Combine
ml_data = features.join(forward_ret.rename("target")).dropna()
```

### Train ML Model

```python
from sklearn.ensemble import RandomForestRegressor
from sklearn.model_selection import TimeSeriesSplit

X = ml_data[["momentum", "value", "quality"]]
y = ml_data["target"]

# Time series cross-validation
tscv = TimeSeriesSplit(n_splits=5)

scores = []
for train_idx, test_idx in tscv.split(X):
    X_train, X_test = X.iloc[train_idx], X.iloc[test_idx]
    y_train, y_test = y.iloc[train_idx], y.iloc[test_idx]

    model = RandomForestRegressor(n_estimators=100, max_depth=5)
    model.fit(X_train, y_train)

    # Predict and score
    pred = model.predict(X_test)
    ic = np.corrcoef(pred, y_test)[0, 1]
    scores.append(ic)

print(f"Average IC: {np.mean(scores):.4f}")
```

### Use ML Predictions in Strategy

```python
# Generate predictions
X_all = ml_data[["momentum", "value", "quality"]]
final_model = RandomForestRegressor(n_estimators=100, max_depth=5)
final_model.fit(X_all[:-252], y[:-252])  # Train on all but last year

# Save predictions
predictions = pd.Series(
    final_model.predict(X_all),
    index=X_all.index,
    name="ml_signal"
)
predictions.to_parquet("ml_predictions.parquet")
```

```sig
// strategy_ml.sig
data:
  prices = "prices.parquet"
  ml_signal = "ml_predictions.parquet"

signal combined:
  // Use ML predictions as signal
  emit zscore(ml_signal)

portfolio ml_strategy:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Jupyter Notebook Workflow

### Complete Notebook Example

```python
# Cell 1: Setup
import pysigc
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# Cell 2: Run Strategy
results = pysigc.run("momentum.sig")

# Cell 3: Summary Statistics
print("Performance Summary")
print("=" * 40)
print(f"Sharpe Ratio:  {results.sharpe_ratio:.2f}")
print(f"Annual Return: {results.annual_return:.1%}")
print(f"Annual Vol:    {results.annual_volatility:.1%}")
print(f"Max Drawdown:  {results.max_drawdown:.1%}")
print(f"Win Rate:      {results.win_rate:.1%}")

# Cell 4: Parameter Optimization
param_results = pysigc.optimize(
    "momentum.sig",
    params={
        "lookback": [20, 40, 60, 80, 100],
        "top_pct": [0.10, 0.15, 0.20, 0.25]
    },
    metric="sharpe"
)

# Cell 5: Visualize Optimization
pivot = param_results.pivot(index="lookback", columns="top_pct", values="sharpe")
plt.figure(figsize=(10, 6))
plt.imshow(pivot, cmap="RdYlGn", aspect="auto")
plt.colorbar(label="Sharpe Ratio")
plt.xticks(range(len(pivot.columns)), pivot.columns)
plt.yticks(range(len(pivot.index)), pivot.index)
plt.xlabel("Top %")
plt.ylabel("Lookback")
plt.title("Parameter Optimization: Sharpe Ratio")
plt.show()

# Cell 6: Best Parameters
best = param_results.loc[param_results["sharpe"].idxmax()]
print(f"Best Parameters: lookback={best['lookback']}, top_pct={best['top_pct']}")
print(f"Best Sharpe: {best['sharpe']:.2f}")
```

## Custom Analysis Functions

### Factor Attribution

```python
def factor_attribution(results, factor_returns):
    """Attribute returns to factors."""
    import statsmodels.api as sm

    # Portfolio returns
    port_ret = results.daily_returns

    # Factor returns (e.g., Fama-French)
    X = factor_returns[["MKT", "SMB", "HML", "MOM"]]
    X = sm.add_constant(X)
    y = port_ret

    # Align dates
    X, y = X.align(y, join="inner")

    # Regression
    model = sm.OLS(y, X).fit()

    print("Factor Attribution")
    print("=" * 40)
    print(model.summary())

    return model
```

### Risk Analysis

```python
def risk_analysis(results, confidence=0.95):
    """Comprehensive risk analysis."""
    returns = results.daily_returns

    # VaR and CVaR
    var = returns.quantile(1 - confidence)
    cvar = returns[returns <= var].mean()

    # Downside deviation
    downside = returns[returns < 0].std() * np.sqrt(252)

    # Max drawdown details
    cumulative = (1 + returns).cumprod()
    rolling_max = cumulative.cummax()
    drawdown = (cumulative - rolling_max) / rolling_max

    max_dd_end = drawdown.idxmin()
    max_dd_start = cumulative[:max_dd_end].idxmax()
    recovery = cumulative[max_dd_end:][cumulative[max_dd_end:] >= cumulative[max_dd_start]].index
    recovery_date = recovery[0] if len(recovery) > 0 else "Not recovered"

    print("Risk Analysis")
    print("=" * 40)
    print(f"VaR ({confidence:.0%}):      {var:.2%}")
    print(f"CVaR ({confidence:.0%}):     {cvar:.2%}")
    print(f"Downside Vol:      {downside:.1%}")
    print(f"Max Drawdown:      {drawdown.min():.1%}")
    print(f"DD Start:          {max_dd_start}")
    print(f"DD End:            {max_dd_end}")
    print(f"Recovery:          {recovery_date}")

    return {
        "var": var,
        "cvar": cvar,
        "downside_vol": downside,
        "max_drawdown": drawdown.min()
    }
```

## Batch Processing

### Run Multiple Strategies

```python
import glob

# Find all strategy files
strategies = glob.glob("strategies/*.sig")

# Run all
all_results = {}
for strat in strategies:
    name = strat.split("/")[-1].replace(".sig", "")
    try:
        results = pysigc.run(strat)
        all_results[name] = {
            "sharpe": results.sharpe_ratio,
            "return": results.total_return,
            "max_dd": results.max_drawdown,
            "vol": results.annual_volatility
        }
    except Exception as e:
        print(f"Error running {name}: {e}")

# Compare
comparison = pd.DataFrame(all_results).T.sort_values("sharpe", ascending=False)
print(comparison)
```

### Export Results

```python
# Export to various formats
results.to_csv("results/performance.csv")
results.weights.to_parquet("results/weights.parquet")
results.daily_returns.to_csv("results/returns.csv")

# Export report
pysigc.generate_report(results, output="results/report.html")
```

## Best Practices

### 1. Use Virtual Environments

```bash
python -m venv sigc_env
source sigc_env/bin/activate
pip install pysigc pandas numpy matplotlib
```

### 2. Cache Expensive Computations

```python
import joblib

# Cache results
cache_file = "cache/results.joblib"

if os.path.exists(cache_file):
    results = joblib.load(cache_file)
else:
    results = pysigc.run("strategy.sig")
    joblib.dump(results, cache_file)
```

### 3. Use Parallel Processing

```python
from concurrent.futures import ProcessPoolExecutor

def run_with_params(params):
    return pysigc.run("strategy.sig", params=params)

param_list = [
    {"lookback": 40, "top_pct": 0.2},
    {"lookback": 60, "top_pct": 0.2},
    {"lookback": 80, "top_pct": 0.2},
]

with ProcessPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(run_with_params, param_list))
```

## Next Steps

- [API Reference](../api/python/index.md) - Complete Python API
- [Tutorials Index](index.md) - More tutorials
- [Production Deployment](production-deployment.md) - Go live
