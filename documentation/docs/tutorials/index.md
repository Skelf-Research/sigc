# Tutorials

Hands-on guides to building strategies with sigc.

## Strategy Tutorials

| Tutorial | Level | Description |
|----------|-------|-------------|
| [Momentum Strategy](momentum-strategy.md) | Beginner | Classic cross-sectional momentum |
| [Mean Reversion](mean-reversion.md) | Beginner | Short-term price reversion |
| [Multi-Factor](multi-factor.md) | Intermediate | Combining multiple signals |
| [Volatility Strategy](volatility-strategy.md) | Intermediate | Volatility-based trading |

## Technical Tutorials

| Tutorial | Level | Description |
|----------|-------|-------------|
| [Custom Functions](custom-functions.md) | Intermediate | Creating reusable functions |
| [Walk-Forward Optimization](walk-forward-optimization.md) | Advanced | Proper validation |
| [Production Deployment](production-deployment.md) | Advanced | Going live |
| [Python Workflow](python-workflow.md) | Intermediate | Integration with Python |

## Learning Path

### Beginner Path

1. **[First Strategy](../getting-started/first-strategy.md)** - Your first backtest
2. **[Momentum Strategy](momentum-strategy.md)** - Classic momentum
3. **[Mean Reversion](mean-reversion.md)** - Price reversion
4. **[Quickstart](../getting-started/quickstart.md)** - Core concepts

### Intermediate Path

1. **[Multi-Factor](multi-factor.md)** - Combining signals
2. **[Volatility Strategy](volatility-strategy.md)** - Vol-based trading
3. **[Custom Functions](custom-functions.md)** - Code reuse
4. **[Python Workflow](python-workflow.md)** - Python integration

### Advanced Path

1. **[Walk-Forward Optimization](walk-forward-optimization.md)** - Proper validation
2. **[Factor Models](../advanced/factor-models.md)** - Factor construction
3. **[Risk Models](../advanced/risk-models.md)** - Risk management
4. **[Production Deployment](production-deployment.md)** - Going live

## Quick Examples

### Momentum

```sig
signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

[Full tutorial →](momentum-strategy.md)

### Mean Reversion

```sig
signal reversion:
  z = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)
  emit -zscore(z)  // Fade extremes

portfolio main:
  weights = rank(reversion).long_short(top=0.2, bottom=0.2)
  backtest rebal=5 from 2020-01-01 to 2024-12-31
```

[Full tutorial →](mean-reversion.md)

### Multi-Factor

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal combined:
  emit 0.5 * momentum + 0.5 * value

portfolio main:
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

[Full tutorial →](multi-factor.md)

## Prerequisites

Before starting tutorials:

1. Install sigc ([Installation](../getting-started/installation.md))
2. Set up your IDE ([IDE Setup](../getting-started/ide-setup.md))
3. Get sample data ([Sample Data](../getting-started/sample-data.md))

## Getting Help

- Check [Concepts](../concepts/index.md) for foundational knowledge
- See [Operators](../operators/index.md) for function reference
- Browse [Strategy Library](../strategies/index.md) for more examples

## Next Steps

Start with the [Momentum Strategy](momentum-strategy.md) tutorial.
