# Low Volatility Strategy

Invest in stable, low-risk stocks.

## Strategy Overview

Buy stocks with lower historical volatility. The "low volatility anomaly" shows these stocks often outperform on a risk-adjusted basis.

## The Signal

```sig
signal low_volatility:
  // Annualized volatility
  daily_ret = ret(prices, 1)
  vol = rolling_std(daily_ret, 60) * sqrt(252)

  // Lower vol = higher score
  emit -zscore(vol)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal low_vol:
  daily_ret = ret(prices, 1)
  vol_60 = rolling_std(daily_ret, 60) * sqrt(252)

  // Sector neutralize to avoid sector concentration
  emit neutralize(-zscore(vol_60), by=sectors)

portfolio low_volatility:
  // Long-only, low vol stocks
  weights = rank(low_vol).long_only(
    top = 0.3,
    cap = 0.05
  )

  constraints:
    max_sector = 0.25

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Long-Short Version

```sig
portfolio low_vol_ls:
  weights = rank(low_vol).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0

  backtest from 2010-01-01 to 2024-12-31
```

## Variations

### Beta-Adjusted

```sig
signal low_beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)

  emit -zscore(beta)
```

### Idiosyncratic Volatility

```sig
signal low_idio_vol:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)

  // Residual returns
  residual = stock_ret - beta * market_ret
  idio_vol = rolling_std(residual, 60) * sqrt(252)

  emit -zscore(idio_vol)
```

### Downside Volatility

```sig
signal low_downside_vol:
  daily_ret = ret(prices, 1)
  negative_ret = where(daily_ret < 0, daily_ret, 0)
  downside_vol = rolling_std(negative_ret, 60) * sqrt(252)

  emit -zscore(downside_vol)
```

## Expected Results

```
Backtest Results: low_volatility
================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 195%
  Annual Return: 7.8%
  Annual Volatility: 10.2%
  Sharpe Ratio: 0.76

Risk:
  Beta: 0.65
  Max Drawdown: -18.2%

Sector Exposure:
  Utilities: 18%
  Consumer Staples: 15%
  Healthcare: 14%
```

## See Also

- [Volatility Targeting](volatility-targeting.md)
- [Vol-Adjusted Returns](vol-adjusted-returns.md)
