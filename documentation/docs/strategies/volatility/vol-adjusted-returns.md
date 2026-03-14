# Volatility-Adjusted Returns Strategy

Rank stocks by risk-adjusted returns rather than raw returns.

## Strategy Overview

High returns aren't meaningful without considering risk. Adjust returns for volatility to find truly superior stocks.

## The Signal

```sig
signal sharpe_momentum:
  ret_60 = ret(prices, 60)
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Risk-adjusted return
  sharpe = ret_60 / vol_60

  emit zscore(sharpe)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal vol_adjusted:
  // 60-day returns
  ret_60 = ret(prices, 60)

  // 60-day realized volatility
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Sharpe-like ratio
  risk_adj_ret = ret_60 / vol_60

  emit neutralize(zscore(risk_adj_ret), by=sectors)

portfolio vol_adjusted:
  weights = rank(vol_adjusted).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Variations

### Information Ratio Style

```sig
signal info_ratio:
  // Excess return vs market
  stock_ret = ret(prices, 60)
  market_ret = ret(market, 60)
  excess_ret = stock_ret - market_ret

  // Tracking error
  residual = ret(prices, 1) - ret(market, 1)
  te = rolling_std(residual, 60) * sqrt(252)

  emit zscore(excess_ret / te)
```

### Sortino-Based

```sig
signal sortino_momentum:
  ret_60 = ret(prices, 60)

  // Downside deviation only
  daily_ret = ret(prices, 1)
  negative_ret = where(daily_ret < 0, daily_ret, 0)
  downside_vol = rolling_std(negative_ret, 60) * sqrt(252)

  sortino = ret_60 / downside_vol

  emit zscore(sortino)
```

## Expected Results

```
Vol-Adjusted vs Raw Momentum
============================

                    Raw Mom   Vol-Adjusted
Annual Return:      6.8%      7.2%
Annual Volatility:  10.5%     9.8%
Sharpe Ratio:       0.65      0.73
```

## See Also

- [Volatility Targeting](volatility-targeting.md)
- [Price Momentum](../momentum/price-momentum.md)
