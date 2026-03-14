# Classic Multi-Factor Strategy

The standard combination of Value, Momentum, and Quality.

## Strategy Overview

Combine the most well-established factors:
- **Momentum**: Past winners continue winning
- **Value**: Cheap stocks outperform
- **Quality**: High-quality companies outperform

## Complete Strategy

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Momentum factor
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  emit neutralize(zscore(ret_12m - ret_1m), by=sectors)

// Value factor
signal value:
  btm = zscore(book_value / market_cap)
  ey = zscore(earnings / prices)
  emit neutralize(0.5 * btm + 0.5 * ey, by=sectors)

// Quality factor
signal quality:
  roe = zscore(net_income / equity)
  stability = -zscore(rolling_std(earnings, 8))
  emit neutralize(0.6 * roe + 0.4 * stability, by=sectors)

// Combined signal
signal multi_factor:
  emit 0.35 * momentum + 0.35 * value + 0.30 * quality

portfolio classic_multi_factor:
  weights = rank(multi_factor).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Expected Results

```
Backtest Results: classic_multi_factor
======================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 215%
  Annual Return: 8.4%
  Annual Volatility: 9.8%
  Sharpe Ratio: 0.86

Factor Contributions:
  Momentum: +3.2%
  Value: +2.5%
  Quality: +1.8%
  Selection: +0.9%
```

## See Also

- [Quality Factor](quality-factor.md)
- [Factor Timing](factor-timing.md)
