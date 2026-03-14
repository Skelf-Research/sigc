// Crash-Protected Momentum Strategy
// Standard momentum with dynamic hedging based on market volatility.
// Reduces exposure when volatility is elevated (momentum crash protection).
// Reference: Daniel & Moskowitz (2016) "Momentum Crashes"

data:
  px: load csv from "data/prices.csv"

params:
  mom_lookback = 252
  vol_lookback = 21
  vol_threshold = 1.5

signal momentum_raw:
  r = ret(px, mom_lookback)
  z = zscore(r)
  emit winsor(z, p=0.01)

signal vol_scaling:
  daily_ret = ret(px, 1)
  realized_vol = rolling_std(daily_ret, vol_lookback)
  baseline_vol = rolling_mean(realized_vol, 252)
  vol_ratio = realized_vol / baseline_vol
  scale = clip(1.0 / vol_ratio, 0.25, 1.0)
  emit scale

signal protected_momentum:
  scaled = momentum_raw * vol_scaling
  emit scaled

portfolio main:
  weights = rank(protected_momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
