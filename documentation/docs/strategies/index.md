# Strategy Library

A collection of example strategies organized by category.

## Categories

| Category | Description | Count |
|----------|-------------|-------|
| [Momentum](momentum/index.md) | Trend-following strategies | 4 |
| [Mean Reversion](mean-reversion/index.md) | Contrarian strategies | 4 |
| [Volatility](volatility/index.md) | Volatility-based strategies | 4 |
| [Multi-Factor](multi-factor/index.md) | Combined signal strategies | 4 |
| [Technical](technical/index.md) | Technical indicator strategies | 4 |
| [Statistical Arbitrage](statistical-arbitrage/index.md) | Stat arb strategies | 3 |

## Quick Reference

### Momentum Strategies

```sig
// Classic 12-1 Month Momentum
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit zscore(ret_12m - ret_1m)
```

### Mean Reversion Strategies

```sig
// Bollinger Band Mean Reversion
signal reversion:
  z = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)
  emit -zscore(z)  // Fade extremes
```

### Volatility Strategies

```sig
// Low Volatility
signal low_vol:
  vol = rolling_std(ret(prices, 1), 252)
  emit -zscore(vol)  // Prefer low vol
```

### Multi-Factor Strategies

```sig
// Value + Momentum
signal combined:
  value = zscore(book_to_market)
  momentum = zscore(ret(prices, 60))
  emit 0.5 * value + 0.5 * momentum
```

### Technical Strategies

```sig
// RSI Mean Reversion
signal rsi_signal:
  rsi_val = rsi(prices, 14)
  emit -zscore((rsi_val - 50) / 25)
```

## Strategy Performance Summary

| Strategy | CAGR | Sharpe | Max DD | Period |
|----------|------|--------|--------|--------|
| Momentum 12-1 | 8.5% | 0.60 | -28% | 2015-2024 |
| Mean Reversion | 6.2% | 0.75 | -15% | 2015-2024 |
| Low Volatility | 7.1% | 0.85 | -18% | 2015-2024 |
| Multi-Factor | 9.2% | 0.82 | -22% | 2015-2024 |

*Past performance is not indicative of future results.*

## Strategy Components

### Signal Construction

All strategies follow this pattern:

```sig
signal my_signal:
  // 1. Compute raw metric
  raw = some_calculation(prices)

  // 2. Normalize
  z = zscore(raw)

  // 3. Handle outliers
  clean = winsor(z, p=0.01)

  // 4. Sector neutralize (optional)
  neutral = neutralize(clean, by=sectors)

  emit neutral
```

### Portfolio Construction

```sig
portfolio my_portfolio:
  // Convert signal to weights
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.03)

  // Add constraints
  constraints:
    max_sector = 0.25
    dollar_neutral = true

  // Add costs
  costs = tc.bps(10)

  // Run backtest
  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## How to Use

### 1. Browse Categories

Explore strategies by category to find approaches that match your objectives.

### 2. Understand the Logic

Each strategy includes:
- Rationale and theory
- Signal construction
- Parameter choices
- Expected behavior

### 3. Customize

Modify parameters for your needs:

```sig
params:
  lookback: 60        // Adjust lookback
  top_pct: 0.2        // Adjust concentration
  rebal_days: 21      // Adjust rebalancing
```

### 4. Validate

Always validate on out-of-sample data:

```sig
backtest walk_forward(
  train_years = 5,
  test_years = 2
) from 2010-01-01 to 2024-12-31
```

## Risk Disclaimer

These strategies are for educational purposes only. Past performance does not guarantee future results. Always:

- Conduct your own research
- Validate on your data
- Test with paper trading
- Understand the risks

## Contributing Strategies

Share your strategies:

1. Fork the repository
2. Add strategy file to appropriate category
3. Include documentation
4. Submit pull request

See [Contributing](../contributing/index.md) for guidelines.

## Next Steps

- [Momentum Strategies](momentum/index.md) - Start with classic momentum
- [Tutorials](../tutorials/index.md) - Step-by-step guides
- [Backtesting](../backtesting/index.md) - Testing strategies
