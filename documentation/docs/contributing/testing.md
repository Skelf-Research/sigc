# Testing Guide

Testing standards and practices for sigc.

## Test Types

| Type | Location | Purpose |
|------|----------|---------|
| Unit | `src/` alongside code | Test individual functions |
| Integration | `tests/` | Test components together |
| End-to-end | `tests/e2e/` | Test full workflows |
| Benchmark | `benches/` | Performance testing |

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test -p sig_parser
cargo test -p sig_runtime
```

### Single Test

```bash
cargo test test_name
```

### With Output

```bash
cargo test -- --nocapture
```

### Integration Tests

```bash
cargo test --test integration
```

### Benchmarks

```bash
cargo bench
```

## Writing Unit Tests

### Basic Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = zscore(&values);

        assert_eq!(result.len(), 5);
        assert!((result[2] - 0.0).abs() < 1e-10); // Middle should be ~0
    }
}
```

### Test Error Cases

```rust
#[test]
fn test_division_by_zero() {
    let result = divide(10.0, 0.0);
    assert!(result.is_err());
    assert!(matches!(result, Err(Error::DivisionByZero)));
}
```

### Test with Setup

```rust
fn setup_test_data() -> DataFrame {
    DataFrame::new(vec![
        ("date", vec!["2024-01-01", "2024-01-02"]),
        ("price", vec![100.0, 102.0]),
    ])
}

#[test]
fn test_with_data() {
    let data = setup_test_data();
    let result = compute(&data);
    assert_eq!(result, expected);
}
```

## Writing Integration Tests

Create files in `tests/`:

```rust
// tests/backtest_integration.rs

use sigc::Strategy;

#[test]
fn test_momentum_backtest() {
    let strategy = Strategy::from_file("strategies/momentum.sig").unwrap();
    let results = strategy.run().unwrap();

    assert!(results.sharpe_ratio() > 0.0);
    assert!(results.total_return() > -1.0);
}
```

## Test Fixtures

### Sample Data

Create test data in `tests/fixtures/`:

```
tests/fixtures/
├── prices_small.csv
├── prices_with_gaps.csv
└── fundamentals.csv
```

### Load Fixtures

```rust
fn load_test_prices() -> DataFrame {
    DataFrame::from_csv("tests/fixtures/prices_small.csv").unwrap()
}
```

## Property-Based Testing

Use proptest for property-based tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_zscore_mean_zero(values in prop::collection::vec(any::<f64>(), 10..100)) {
        let zscores = zscore(&values);
        let mean: f64 = zscores.iter().sum::<f64>() / zscores.len() as f64;
        prop_assert!((mean - 0.0).abs() < 1e-10);
    }
}
```

## Mocking

For external dependencies:

```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use super::*;

    mock! {
        BrokerClient {
            fn submit_order(&self, order: Order) -> Result<()>;
        }
    }

    #[test]
    fn test_execution() {
        let mut mock_broker = MockBrokerClient::new();
        mock_broker
            .expect_submit_order()
            .times(1)
            .returning(|_| Ok(()));

        let executor = Executor::new(mock_broker);
        let result = executor.execute(order);
        assert!(result.is_ok());
    }
}
```

## Python Tests

### Setup

```bash
cd pysigc
pip install -e ".[dev]"
pytest
```

### Writing Tests

```python
# tests/test_run.py
import pysigc

def test_basic_run():
    results = pysigc.run("tests/fixtures/momentum.sig")
    assert results.sharpe_ratio is not None

def test_with_params():
    results = pysigc.run(
        "tests/fixtures/momentum.sig",
        params={"lookback": 60}
    )
    assert results.total_return > -1.0
```

## Test Coverage

### Generate Coverage Report

```bash
cargo tarpaulin --out Html
```

### Coverage Targets

| Area | Target |
|------|--------|
| Parser | 90%+ |
| Runtime | 85%+ |
| CLI | 70%+ |

## CI/CD Tests

Tests run automatically on:
- Pull requests
- Main branch pushes

Required to pass:
- All unit tests
- All integration tests
- Clippy (no warnings)
- rustfmt check

## Best Practices

### 1. Test One Thing

```rust
// Good: Focused test
#[test]
fn test_momentum_calculates_return() {
    // Only tests return calculation
}

// Avoid: Testing multiple things
#[test]
fn test_momentum_everything() {
    // Tests return, zscore, ranking, portfolio...
}
```

### 2. Descriptive Names

```rust
// Good
#[test]
fn test_zscore_returns_zero_for_mean_value() { }

// Avoid
#[test]
fn test_zscore() { }
```

### 3. Arrange-Act-Assert

```rust
#[test]
fn test_compute_signal() {
    // Arrange
    let data = setup_data();
    let params = SignalParams::default();

    // Act
    let result = compute_signal(&data, &params);

    // Assert
    assert_eq!(result.len(), expected_len);
}
```

### 4. Test Edge Cases

- Empty input
- Single element
- Negative values
- NaN/Inf values
- Boundary values

## See Also

- [Development Setup](development-setup.md)
- [Code Style](code-style.md)
