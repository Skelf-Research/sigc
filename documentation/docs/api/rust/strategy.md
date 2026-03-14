# Strategy Module

Loading, configuring, and running strategies.

## Strategy Struct

```rust
pub struct Strategy {
    // Internal fields
}
```

## Loading Strategies

### From File

```rust
use sigc::Strategy;

let strategy = Strategy::from_file("path/to/strategy.sig")?;
```

### From String

```rust
let code = r#"
signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
"#;

let strategy = Strategy::from_str(code)?;
```

### With Data

```rust
use sigc::{Strategy, DataFrame};

let data: DataFrame = load_data("prices.parquet")?;
let strategy = Strategy::from_file("strategy.sig")?
    .with_data(data);
```

## Configuration

### Parameters

```rust
let strategy = Strategy::from_file("strategy.sig")?
    .with_param("lookback", 60)
    .with_param("top_pct", 0.2)
    .with_param("rebal_days", 21);
```

### Date Range

```rust
use chrono::NaiveDate;

let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

let strategy = Strategy::from_file("strategy.sig")?
    .with_date_range(start, end);
```

### Configuration File

```rust
use sigc::Config;

let config = Config::from_file("config.yaml")?;
let strategy = Strategy::from_file("strategy.sig")?
    .with_config(config);
```

## Running

### Basic Run

```rust
let results = strategy.run()?;
```

### With Options

```rust
use sigc::RunOptions;

let options = RunOptions::default()
    .parallel(true)
    .workers(8)
    .verbose(true);

let results = strategy.run_with_options(options)?;
```

### Streaming Results

```rust
let mut runner = strategy.stream()?;

while let Some(day_result) = runner.next()? {
    println!("Day {}: P&L = {:.2}", day_result.date, day_result.pnl);
}
```

## Validation

### Check Syntax

```rust
let validation = Strategy::validate_file("strategy.sig")?;

if validation.has_errors() {
    for error in validation.errors() {
        eprintln!("Error: {}", error);
    }
}
```

### Check Types

```rust
let type_check = strategy.type_check()?;
```

## Signal Access

### Get Signal Values

```rust
let signals = strategy.compute_signals()?;
let momentum = signals.get("momentum")?;

// momentum is a DataFrame with signal values
```

### Get Weights

```rust
let weights = strategy.compute_weights()?;
// Returns target portfolio weights
```

## Methods Summary

| Method | Description |
|--------|-------------|
| `from_file(path)` | Load from file |
| `from_str(code)` | Load from string |
| `with_param(name, value)` | Set parameter |
| `with_config(config)` | Apply configuration |
| `with_data(data)` | Provide data |
| `with_date_range(start, end)` | Set backtest range |
| `run()` | Execute strategy |
| `compute_signals()` | Get signal values |
| `compute_weights()` | Get target weights |

## See Also

- [Results Module](results.md)
- [Data Module](data.md)
