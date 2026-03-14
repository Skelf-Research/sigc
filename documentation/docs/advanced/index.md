# Advanced Topics

Deep dives into advanced sigc features and techniques.

## Overview

| Topic | Description |
|-------|-------------|
| [Factor Models](factor-models.md) | Multi-factor portfolio construction |
| [Risk Models](risk-models.md) | Portfolio risk estimation |
| [Regime Detection](regime-detection.md) | Market regime identification |
| [Portfolio Optimization](portfolio-optimization.md) | Mean-variance and beyond |
| [Parallel Execution](parallel-execution.md) | Multi-core computation |
| [Incremental Computation](incremental-computation.md) | Efficient updates |
| [Memory Mapping](memory-mapping.md) | Large dataset handling |

## Prerequisites

Before diving into advanced topics, ensure familiarity with:

- [Core Concepts](../concepts/index.md)
- [Language Reference](../language/index.md)
- [Backtesting](../backtesting/index.md)

## Quick Overview

### Factor Models

Build multi-factor strategies:

```sig
signal value:
  emit zscore(book_to_market)

signal momentum:
  emit zscore(ret(prices, 60))

signal quality:
  emit zscore(roe)

signal composite:
  emit 0.4 * value + 0.4 * momentum + 0.2 * quality
```

[Learn more →](factor-models.md)

### Risk Models

Estimate and manage portfolio risk:

```sig
portfolio risk_managed:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    target_volatility = 0.10
    max_beta = 1.2
    max_sector = 0.25
```

[Learn more →](risk-models.md)

### Regime Detection

Adapt to market conditions:

```sig
signal regime_aware:
  vol = rolling_std(ret(prices, 1), 60)
  high_vol = vol > quantile(vol, 0.8)

  momentum = zscore(ret(prices, 60))
  reversion = -zscore(ret(prices, 5))

  // Momentum in low vol, reversion in high vol
  emit where(high_vol, reversion, momentum)
```

[Learn more →](regime-detection.md)

### Portfolio Optimization

Beyond equal-weight:

```sig
portfolio optimized:
  weights = optimize(
    signal = alpha_signal,
    objective = maximize("sharpe"),
    constraints:
      max_position = 0.05
      dollar_neutral = true
      target_volatility = 0.12
  )
```

[Learn more →](portfolio-optimization.md)

### Performance Optimization

For large-scale computation:

```yaml
performance:
  parallel:
    enabled: true
    workers: 8

  incremental:
    enabled: true
    cache: true

  memory:
    mmap: true
    max_memory_gb: 16
```

[Learn more →](parallel-execution.md)

## When to Use Advanced Features

### Use Factor Models When

- Building multi-factor strategies
- Combining signals from different sources
- Need factor attribution

### Use Risk Models When

- Managing portfolio volatility
- Controlling factor exposures
- Meeting risk constraints

### Use Regime Detection When

- Strategy performance varies by market conditions
- Want adaptive allocation
- Combining multiple strategy types

### Use Optimization When

- Want risk-adjusted weighting
- Have specific risk targets
- Need constrained optimization

### Use Performance Features When

- Processing large datasets (>1M rows)
- Running many backtests
- Real-time computation needs

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Advanced sigc                            │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   Factor    │    │    Risk     │    │  Regime     │    │
│  │   Models    │    │   Models    │    │ Detection   │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│         │                  │                  │            │
│         ▼                  ▼                  ▼            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Portfolio Optimization                 │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                               │
│                           ▼                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │           Parallel / Incremental Compute            │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Best Practices

### 1. Start Simple

Add complexity only when needed.

### 2. Test Each Component

Validate each advanced feature separately.

### 3. Monitor Performance

Track computation time and memory.

### 4. Document Assumptions

Advanced features have more assumptions.

## Next Steps

- [Factor Models](factor-models.md) - Multi-factor construction
- [Risk Models](risk-models.md) - Risk estimation
- [Tutorials](../tutorials/index.md) - Hands-on examples
