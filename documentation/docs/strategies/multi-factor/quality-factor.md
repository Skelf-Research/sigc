# Quality Factor Strategy

Invest in high-quality, profitable companies.

## Strategy Overview

Quality stocks have:
- High profitability (ROE, ROA)
- Stable earnings
- Low leverage
- Strong cash flows

## The Signal

```sig
signal quality:
  // Profitability
  roe = zscore(net_income / equity)
  roa = zscore(net_income / assets)
  margin = zscore(gross_margin)

  // Stability
  earnings_stability = -zscore(rolling_std(earnings, 8))

  // Safety
  leverage = -zscore(debt_to_equity)

  // Combined
  profitability = 0.4 * roe + 0.3 * roa + 0.3 * margin
  safety = 0.5 * earnings_stability + 0.5 * leverage

  emit 0.7 * profitability + 0.3 * safety
```

## Complete Strategy

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

signal quality:
  // Profitability metrics
  roe = zscore(net_income / equity)
  roa = zscore(net_income / assets)
  gross_margin_z = zscore(gross_margin)

  // Stability
  earnings_vol = rolling_std(earnings, 8)
  stability = -zscore(earnings_vol)

  // Safety
  debt_equity = debt / equity
  leverage = -zscore(debt_equity)

  // Cash quality
  accruals = (net_income - operating_cash_flow) / assets
  cash_quality = -zscore(accruals)

  // Combined quality score
  profitability = 0.4 * roe + 0.3 * roa + 0.3 * gross_margin_z
  safety = 0.5 * stability + 0.5 * leverage

  quality_score = 0.5 * profitability + 0.3 * safety + 0.2 * cash_quality

  emit neutralize(quality_score, by=sectors)

portfolio quality:
  weights = rank(quality).long_short(
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
Backtest Results: quality
=========================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 145%
  Annual Return: 6.5%
  Annual Volatility: 8.5%
  Sharpe Ratio: 0.76

Quality Metrics (Longs):
  Avg ROE: 22%
  Avg Debt/Equity: 0.4
  Earnings Stability: High
```

## See Also

- [Classic Multi-Factor](classic-multi-factor.md)
- [Momentum + Quality](../momentum/momentum-quality.md)
