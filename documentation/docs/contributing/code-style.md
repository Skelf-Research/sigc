# Code Style Guide

Coding standards for sigc contributions.

## Rust Style

### Formatting

Use `rustfmt` for all Rust code:

```bash
cargo fmt
```

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `compute_signal` |
| Types | PascalCase | `SignalValue` |
| Constants | SCREAMING_SNAKE | `MAX_WORKERS` |
| Modules | snake_case | `sig_parser` |
| Local variables | snake_case | `daily_return` |

### Documentation

Document all public items:

```rust
/// Computes the signal value for a given date.
///
/// # Arguments
///
/// * `date` - The date to compute the signal for
/// * `prices` - Price data
///
/// # Returns
///
/// The computed signal value
///
/// # Errors
///
/// Returns an error if the date is not in the price data
pub fn compute_signal(date: NaiveDate, prices: &DataFrame) -> Result<f64> {
    // ...
}
```

### Error Handling

Use custom error types:

```rust
// Good
fn load_data(path: &str) -> Result<DataFrame, DataError> {
    let file = File::open(path)
        .map_err(|e| DataError::FileNotFound(path.to_string()))?;
    // ...
}

// Avoid
fn load_data(path: &str) -> Result<DataFrame, Box<dyn Error>> {
    // ...
}
```

### Testing

Test all public functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signal() {
        let prices = create_test_data();
        let result = compute_signal(date, &prices).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compute_signal_missing_date() {
        let prices = create_test_data();
        let result = compute_signal(missing_date, &prices);
        assert!(result.is_err());
    }
}
```

## sigc DSL Style

### Signal Definitions

```sig
// Good: Clear, descriptive names
signal momentum_12_1:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit zscore(ret_12m - ret_1m)

// Avoid: Cryptic names
signal s1:
  r = ret(prices, 252) - ret(prices, 21)
  emit zscore(r)
```

### Comments

```sig
// Explain the "why", not the "what"

// Good: Explains rationale
// Skip last month to avoid short-term reversal contamination
ret_skip = ret(prices, 21)

// Avoid: Just restates the code
// Calculate 21-day return
ret_skip = ret(prices, 21)
```

### Formatting

- Indent with 2 spaces
- One blank line between signal definitions
- Align `emit` statements

## Python Style

Follow PEP 8 with these additions:

### Type Hints

Use type hints for all public functions:

```python
def run(
    strategy: str,
    params: Optional[Dict[str, Any]] = None,
    start: Optional[str] = None,
    end: Optional[str] = None
) -> Results:
    """Run a backtest."""
    ...
```

### Docstrings

Use Google-style docstrings:

```python
def compute_signal(data: pd.DataFrame, lookback: int) -> pd.Series:
    """Compute the momentum signal.

    Args:
        data: Price data with 'close' column
        lookback: Number of days for return calculation

    Returns:
        Series of signal values

    Raises:
        ValueError: If lookback is negative
    """
    ...
```

## Git Commit Messages

### Format

```
type(scope): short description

Longer description if needed. Wrap at 72 characters.

Fixes #123
```

### Types

| Type | Use |
|------|-----|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `style` | Formatting |
| `refactor` | Code restructure |
| `test` | Tests |
| `chore` | Maintenance |

### Examples

```
feat(parser): add support for range() in params

fix(runtime): handle division by zero in zscore

docs(api): add Python API reference

test(signals): add tests for momentum calculation
```

## Pull Request Guidelines

### Title

Follow commit message format:
```
feat(parser): add support for range() in params
```

### Description

Include:
- What changed and why
- How to test
- Related issues

### Size

- Prefer small, focused PRs
- Split large changes into multiple PRs

## Code Review

### For Authors

- Respond to all comments
- Explain your reasoning
- Be open to feedback

### For Reviewers

- Be constructive
- Approve when satisfied
- Suggest, don't demand

## See Also

- [Development Setup](development-setup.md)
- [Testing](testing.md)
