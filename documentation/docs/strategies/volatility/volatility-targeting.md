# Volatility Targeting Strategy

Dynamically scale positions to target constant portfolio volatility.

## Strategy Overview

Instead of fixed position sizes, adjust exposure inversely to volatility. This produces more stable risk-adjusted returns.

## The Concept

```
Target Vol = 10%
Current Vol = 15%
Scale Factor = 10% / 15% = 0.67

Apply: weights = base_weights × 0.67
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Alpha signal (any signal works)
signal momentum:
  emit zscore(ret(prices, 60))

// Volatility scaling
signal vol_scale:
  // Portfolio-level realized vol estimate
  port_ret = weighted_return(prices, current_weights)
  realized_vol = rolling_std(port_ret, 20) * sqrt(252)

  // Target 10% vol
  target = 0.10
  scale = target / realized_vol

  // Constrain scaling
  emit clip(scale, 0.5, 1.5)

portfolio vol_targeted:
  // Base weights
  base_weights = rank(momentum).long_short(top=0.2, bottom=0.2)

  // Apply vol scaling
  weights = base_weights * vol_scale

  constraints:
    gross_exposure: [1.0, 3.0]  // Allow dynamic exposure
    net_exposure = 0.0

  costs = tc.bps(10)

  backtest rebal=5 from 2010-01-01 to 2024-12-31
```

## Per-Asset Vol Targeting

```sig
signal asset_vol_targeted:
  // Individual stock volatility
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)

  // Target equal risk contribution
  target_vol = 0.20
  scale = target_vol / vol

  // Scale signal by inverse vol
  raw_signal = momentum
  scaled = raw_signal * scale

  emit zscore(scaled)
```

## Risk Parity Approach

```sig
signal risk_parity:
  // Weight inversely to volatility
  vol = rolling_std(ret(prices, 1), 60)
  inv_vol = 1 / vol

  // Normalize
  weight = inv_vol / sum(inv_vol)

  emit weight
```

## Expected Results

```
Vol-Targeted vs Base Strategy
=============================

                    Base    Vol-Targeted
Annual Return:      8.5%    7.8%
Annual Volatility:  12.1%   10.2%
Sharpe Ratio:       0.70    0.76
Max Drawdown:      -22.5%  -16.8%
```

## See Also

- [Low Volatility](low-volatility.md)
- [Risk Models](../../advanced/risk-models.md)
