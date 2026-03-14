# Tutorial: Volatility Strategy

Build strategies that trade volatility patterns and profit from volatility risk premium.

## Overview

Volatility strategies exploit:

- **Low volatility anomaly** - Low vol stocks outperform
- **Volatility clustering** - Vol is persistent
- **Volatility mean reversion** - Extreme vol reverts
- **Volatility risk premium** - Selling vol is profitable

## Strategy 1: Low Volatility

### Defensive Low-Vol Strategy

```sig
data:
  source = "prices.parquet"
  format = parquet

signal low_volatility:
  // Historical volatility (annualized)
  daily_ret = ret(prices, 1)
  vol_60 = rolling_std(daily_ret, 60) * sqrt(252)

  // Lower vol = higher score
  emit -zscore(vol_60)

portfolio low_vol:
  weights = rank(low_volatility).long_only(top=0.2, cap=0.05)
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 2: Volatility-Adjusted Momentum

### Normalize Returns by Volatility

```sig
signal vol_adjusted_momentum:
  // Raw momentum
  ret_60 = ret(prices, 60)

  // Volatility adjustment
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Risk-adjusted momentum
  sharpe_signal = ret_60 / vol_60

  emit zscore(sharpe_signal)

portfolio vol_adj_mom:
  weights = rank(vol_adjusted_momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 3: Volatility Breakout

### Trade Expanding Volatility

```sig
signal vol_breakout:
  // Current vs historical volatility
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Volatility expansion ratio
  vol_expansion = vol_20 / vol_60

  // Direction from price trend
  trend = ret(prices, 20)

  // Signal: Vol expanding + positive trend = long
  signal = zscore(vol_expansion) * sign(trend)

  emit signal

portfolio vol_breakout:
  weights = rank(vol_breakout).long_short(top=0.15, bottom=0.15)
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 4: Volatility Mean Reversion

### Fade Extreme Volatility

```sig
signal vol_mean_reversion:
  // Current volatility
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)

  // Long-term average
  vol_252 = rolling_mean(vol_20, 252)

  // Deviation from mean
  vol_zscore = (vol_20 - vol_252) / rolling_std(vol_20, 252)

  // Extreme high vol will mean revert down
  // This creates opportunity when combined with price signal
  price_ret = ret(prices, 5)

  // Oversold + high vol = buying opportunity
  signal = where(vol_zscore > 1.5 and price_ret < 0, 1,
           where(vol_zscore < -1 and price_ret > 0, -1, 0))

  emit signal

portfolio vol_reversion:
  weights = rank(vol_mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 5: Risk Parity Weighting

### Equal Risk Contribution

```sig
signal inverse_vol:
  // Inverse volatility weighting
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  inv_vol = 1 / vol

  // Normalize to sum to 1
  weight = inv_vol / sum(inv_vol)

  emit weight

portfolio risk_parity:
  weights = inverse_vol
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 6: Volatility Regime Strategy

### Adaptive Based on VIX Levels

```sig
data:
  source = "prices_with_vix.parquet"
  format = parquet

signal volatility_regime:
  // VIX levels
  low_vix = vix < 15
  normal_vix = vix >= 15 and vix < 25
  high_vix = vix >= 25

  // Return regime indicator
  emit where(high_vix, 3, where(normal_vix, 2, 1))

signal regime_adaptive:
  regime = volatility_regime

  // Momentum signal
  momentum = zscore(ret(prices, 60))

  // Low vol signal
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  low_vol = -zscore(vol)

  // Mean reversion
  ma_20 = rolling_mean(prices, 20)
  mean_rev = -zscore((prices - ma_20) / ma_20)

  // Regime-based strategy selection
  // Low VIX: Momentum works
  // Normal VIX: Balanced
  // High VIX: Defensive (low vol) and mean reversion
  signal = where(regime == 1, momentum,
           where(regime == 2, 0.5 * momentum + 0.5 * low_vol,
                 0.3 * low_vol + 0.7 * mean_rev))

  emit signal

portfolio regime_adaptive:
  weights = rank(regime_adaptive).long_short(top=0.2, bottom=0.2)

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0

  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 7: Volatility Carry

### Short High Implied Vol, Long Low Implied Vol

```sig
data:
  source = "prices_with_iv.parquet"
  format = parquet

signal vol_carry:
  // Implied vs Realized spread (volatility risk premium)
  implied_vol = iv_30  // 30-day implied vol
  realized_vol = rolling_std(ret(prices, 1), 30) * sqrt(252)

  // Positive spread = sell vol = buy stock
  vrp = implied_vol - realized_vol

  // High VRP = stock has expensive options = potential headwind
  // Low VRP = cheap options = potential tailwind
  emit -zscore(vrp)

portfolio vol_carry:
  weights = rank(vol_carry).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Strategy 8: Complete Volatility Strategy

### Production Implementation

```sig
data:
  source = "full_dataset.parquet"
  format = parquet

// ============ VOLATILITY SIGNALS ============

// Low volatility factor
signal low_vol_factor:
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  emit neutralize(-zscore(vol), by=sectors)

// Volatility momentum (vol trending)
signal vol_momentum:
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_60 = rolling_std(ret(prices, 1), 60) * sqrt(252)
  vol_trend = vol_20 / vol_60
  // Rising vol = risk increasing
  emit neutralize(-zscore(vol_trend), by=sectors)

// Volatility mean reversion
signal vol_reversion:
  vol_20 = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_mean = rolling_mean(vol_20, 252)
  vol_deviation = (vol_20 - vol_mean) / vol_mean
  // Extreme high vol reverts = opportunity
  emit neutralize(-zscore(vol_deviation), by=sectors)

// Idiosyncratic volatility
signal idio_vol:
  // Market-adjusted returns
  market_ret = rolling_mean(ret(prices, 1), 1)  // Proxy
  stock_ret = ret(prices, 1)
  residual = stock_ret - market_ret
  idio = rolling_std(residual, 60) * sqrt(252)
  // Low idio vol = better risk-adjusted
  emit neutralize(-zscore(idio), by=sectors)

// ============ REGIME DETECTION ============

signal market_vol_regime:
  avg_vol = mean(rolling_std(ret(prices, 1), 20) * sqrt(252))
  vol_percentile = ts_rank(avg_vol, 252) / 252
  high_vol_regime = vol_percentile > 0.7
  emit high_vol_regime

// ============ COMBINED SIGNAL ============

signal volatility_composite:
  high_vol = market_vol_regime

  // Dynamic weights based on regime
  w_low = where(high_vol, 0.40, 0.25)
  w_mom = where(high_vol, 0.15, 0.25)
  w_rev = where(high_vol, 0.30, 0.25)
  w_idio = where(high_vol, 0.15, 0.25)

  combined = w_low * low_vol_factor +
             w_mom * vol_momentum +
             w_rev * vol_reversion +
             w_idio * idio_vol

  emit combined

// ============ PORTFOLIO ============

portfolio volatility_strategy:
  weights = rank(volatility_composite).long_short(
    top = 0.15,
    bottom = 0.15,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure: [-0.1, 0.1]
    max_sector = 0.20
    max_turnover = 0.25

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Analysis and Metrics

### Run with Volatility Analysis

```bash
sigc run volatility_strategy.sig --vol-analysis
```

Output:
```
Volatility Analysis
==================

Strategy Volatility:
  Realized: 8.2%
  Target: 10.0%

Factor Volatility Loadings:
  Market Vol: -0.15 (defensive)
  VIX: -0.22 (negative correlation)

Regime Performance:
  Low VIX (<15):    +9.2% annual
  Normal VIX:       +6.8% annual
  High VIX (>25):   +4.1% annual

Vol-of-Vol Analysis:
  Vol Stability: 0.78 (stable)
  Max Drawdown: -8.5%
```

## Key Insights

### Why Low Volatility Works

1. **Leverage constraints** - Investors can't lever low vol
2. **Lottery preference** - People overpay for high vol
3. **Benchmarking** - Managers take extra risk for alpha

### Volatility Clustering

```
High vol today → High vol tomorrow
Low vol today → Low vol tomorrow
```

Use this for position sizing and risk management.

### Transaction Costs

Volatility signals often have moderate turnover. Balance:
- Signal decay (faster rebalancing)
- Transaction costs (slower rebalancing)

## Common Pitfalls

1. **Ignoring regime** - Vol strategies fail in transitions
2. **Concentration** - Low vol can concentrate in sectors
3. **Beta exposure** - Low vol often = low beta
4. **Timing risk** - Vol spikes can cause large losses

## Next Steps

- [Custom Functions](custom-functions.md) - Create reusable code
- [Walk-Forward Optimization](walk-forward-optimization.md) - Robust testing
- [Risk Models](../advanced/risk-models.md) - Advanced risk analysis
