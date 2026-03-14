# Results Module

Accessing and analyzing backtest results.

## Results Struct

```rust
pub struct Results {
    // Internal fields
}
```

## Performance Metrics

### Returns

```rust
let results = strategy.run()?;

// Return metrics
let total_return = results.total_return();
let annual_return = results.annual_return();
let excess_return = results.excess_return(); // vs benchmark
```

### Risk

```rust
let volatility = results.annual_volatility();
let max_drawdown = results.max_drawdown();
let avg_drawdown = results.avg_drawdown();
let var_95 = results.var(0.95);
let cvar_95 = results.cvar(0.95);
```

### Risk-Adjusted

```rust
let sharpe = results.sharpe_ratio();
let sortino = results.sortino_ratio();
let calmar = results.calmar_ratio();
let information = results.information_ratio(); // vs benchmark
```

### Trading

```rust
let turnover = results.annual_turnover();
let win_rate = results.win_rate();
let profit_factor = results.profit_factor();
let avg_holding = results.avg_holding_period();
```

## Time Series Data

### Returns

```rust
// Daily returns as Vec<f64>
let daily_returns = results.daily_returns();

// Cumulative returns
let cumulative = results.cumulative_returns();
```

### Weights

```rust
// Portfolio weights over time
let weights: Vec<Weight> = results.weights();

for w in weights {
    println!("{}: {} = {:.3}", w.date, w.symbol, w.weight);
}
```

### Positions

```rust
let positions = results.positions();

for pos in positions {
    println!("{}: {} shares of {}", pos.date, pos.quantity, pos.symbol);
}
```

### Drawdowns

```rust
let drawdowns = results.drawdowns();  // Vec<f64>
```

## Analysis

### Factor Attribution

```rust
let attribution = results.factor_attribution()?;

println!("Market: {:.2}%", attribution.market * 100.0);
println!("Size: {:.2}%", attribution.size * 100.0);
println!("Value: {:.2}%", attribution.value * 100.0);
println!("Momentum: {:.2}%", attribution.momentum * 100.0);
println!("Alpha: {:.2}%", attribution.alpha * 100.0);
```

### Sector Analysis

```rust
let sector_returns = results.returns_by_sector();

for (sector, ret) in sector_returns {
    println!("{}: {:.2}%", sector, ret * 100.0);
}
```

### Regime Analysis

```rust
let regime_performance = results.performance_by_regime()?;

for (regime, metrics) in regime_performance {
    println!("{}: Sharpe = {:.2}", regime, metrics.sharpe);
}
```

## Export

### To DataFrame

```rust
use polars::prelude::DataFrame;

let df: DataFrame = results.to_dataframe()?;
```

### To CSV

```rust
results.to_csv("results.csv")?;
```

### To JSON

```rust
let json = results.to_json()?;
```

### To Parquet

```rust
results.to_parquet("results.parquet")?;
```

## Summary

### Print Summary

```rust
results.print_summary();
```

Output:
```
Backtest Results
================
Period: 2020-01-01 to 2024-12-31

Returns:
  Total Return: 45.2%
  Annual Return: 8.3%
  Annual Volatility: 10.5%
  Sharpe Ratio: 0.79

Risk:
  Max Drawdown: -15.2%
  ...
```

### Get Summary Struct

```rust
let summary = results.summary();
println!("Sharpe: {}", summary.sharpe_ratio);
```

## Methods Summary

| Method | Returns | Description |
|--------|---------|-------------|
| `sharpe_ratio()` | `f64` | Sharpe ratio |
| `total_return()` | `f64` | Total return |
| `max_drawdown()` | `f64` | Maximum drawdown |
| `daily_returns()` | `Vec<f64>` | Daily return series |
| `weights()` | `Vec<Weight>` | Weight time series |
| `to_csv(path)` | `Result<()>` | Export to CSV |
| `print_summary()` | `()` | Print summary |

## See Also

- [Strategy Module](strategy.md)
- [Types Module](types.md)
