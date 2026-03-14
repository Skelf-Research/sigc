# Factor Timing Strategy

Dynamically adjust factor weights based on market conditions.

## Strategy Overview

Different factors work in different environments:
- Momentum: Strong in trending markets
- Value: Strong in recoveries
- Quality: Strong in downturns

## Complete Strategy

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// Individual factors
signal momentum:
  emit neutralize(zscore(ret(prices, 60)), by=sectors)

signal value:
  emit neutralize(zscore(book_to_market), by=sectors)

signal quality:
  emit neutralize(zscore(roe), by=sectors)

// Regime detection
signal volatility_regime:
  vol = rolling_std(ret(market, 1), 20) * sqrt(252)
  long_vol = rolling_std(ret(market, 1), 60) * sqrt(252)
  high_vol = vol > long_vol * 1.3
  emit high_vol

signal trend_regime:
  ma_50 = rolling_mean(market, 50)
  ma_200 = rolling_mean(market, 200)
  uptrend = ma_50 > ma_200
  emit uptrend

// Dynamic weighting
signal factor_timed:
  high_vol = volatility_regime
  uptrend = trend_regime

  // Bull + Low Vol: Momentum heavy
  // Bull + High Vol: Balanced
  // Bear + Low Vol: Value heavy
  // Bear + High Vol: Quality heavy

  w_mom = where(uptrend and not(high_vol), 0.45,
          where(uptrend and high_vol, 0.30, 0.20))

  w_val = where(not(uptrend) and not(high_vol), 0.45,
          where(uptrend, 0.30, 0.25))

  w_qual = 1.0 - w_mom - w_val

  emit w_mom * momentum + w_val * value + w_qual * quality

portfolio factor_timed:
  weights = rank(factor_timed).long_short(
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

## Expected Results

```
Factor-Timed vs Equal Weight
============================

                Equal Weight  Factor-Timed
Annual Return:  8.2%          9.1%
Sharpe Ratio:   0.82          0.91
Max Drawdown:   -16.5%        -14.2%
```

## See Also

- [Classic Multi-Factor](classic-multi-factor.md)
- [Regime Detection](../../advanced/regime-detection.md)
