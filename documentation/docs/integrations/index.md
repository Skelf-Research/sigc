# Integrations

Connect sigc with external tools, languages, and services.

## Overview

sigc integrates with:

| Category | Integrations |
|----------|--------------|
| **Languages** | [Python](python.md) |
| **IDEs** | [VSCode](vscode.md) |
| **Brokers** | [Alpaca](alpaca.md) |
| **Data** | [Yahoo Finance](yahoo-finance.md) |
| **Streaming** | [Real-time data](streaming.md) |

## Python Integration

Use sigc from Python for:

- Jupyter notebooks
- Custom analysis
- Integration with pandas/numpy
- Machine learning pipelines

```python
import pysigc

# Run a strategy
results = pysigc.run("strategy.sig")
print(f"Sharpe: {results.sharpe_ratio:.2f}")

# Access weights
weights = results.weights  # pandas DataFrame
```

[Learn more →](python.md)

## VSCode Extension

Full IDE support:

- Syntax highlighting
- Code completion
- Error diagnostics
- Hover documentation

```bash
# Install extension
code --install-extension skelf-Research.sigc-vscode
```

[Learn more →](vscode.md)

## Alpaca Trading

Execute trades through Alpaca:

```yaml
output:
  type: alpaca
  alpaca:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
    paper: true  # Paper trading
```

[Learn more →](alpaca.md)

## Yahoo Finance

Free market data:

```sig
data:
  source = "yahoo"
  symbols = ["AAPL", "MSFT", "GOOGL"]
  start = "2020-01-01"
```

[Learn more →](yahoo-finance.md)

## Real-Time Streaming

Stream live data:

```yaml
data:
  type: streaming
  provider: polygon
  symbols: ["AAPL", "MSFT"]
```

[Learn more →](streaming.md)

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         sigc Core                           │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   Parser    │    │   Runtime   │    │   Output    │    │
│  │   (DSL)     │    │  (Compute)  │    │  (Execute)  │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│         │                  │                  │            │
└─────────┼──────────────────┼──────────────────┼────────────┘
          │                  │                  │
    ┌─────▼─────┐      ┌─────▼─────┐      ┌─────▼─────┐
    │  VSCode   │      │  Python   │      │  Alpaca   │
    │ Extension │      │ (pysigc)  │      │  Broker   │
    └───────────┘      └───────────┘      └───────────┘
```

## Adding Custom Integrations

### Data Provider

```rust
// Implement DataProvider trait
impl DataProvider for MyProvider {
    fn fetch(&self, symbols: &[String], range: DateRange) -> DataFrame;
}
```

### Execution Provider

```rust
// Implement ExecutionProvider trait
impl ExecutionProvider for MyBroker {
    fn submit_order(&self, order: Order) -> OrderResult;
    fn get_positions(&self) -> Vec<Position>;
}
```

## Best Practices

### 1. Use Paper Trading First

```yaml
output:
  type: alpaca
  alpaca:
    paper: true  # Always start with paper
```

### 2. Handle Rate Limits

```yaml
data:
  provider: yahoo
  rate_limit:
    requests_per_minute: 30
```

### 3. Cache External Data

```yaml
data:
  cache:
    enabled: true
    ttl_hours: 24
```

### 4. Error Handling

```python
try:
    results = pysigc.run("strategy.sig")
except pysigc.DataError as e:
    print(f"Data issue: {e}")
except pysigc.ExecutionError as e:
    print(f"Execution issue: {e}")
```

## Next Steps

- [Python](python.md) - Python bindings
- [VSCode](vscode.md) - IDE setup
- [Alpaca](alpaca.md) - Trading integration
