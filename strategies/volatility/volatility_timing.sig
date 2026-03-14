// Volatility Timing Strategy
// Times exposure based on volatility regime.
// Increases position when vol is low, decreases when high.
// Based on inverse volatility weighting principle.

data:
  px: load csv from "data/prices.csv"

params:
  vol_window = 21
  long_vol_window = 252
  target_vol = 0.10

signal vol_adjusted_signal:
  mom = ret(px, 126)
  base_signal = zscore(mom)
  daily_ret = ret(px, 1)
  realized_vol = rolling_std(daily_ret, vol_window) * sqrt(252)
  vol_scale = target_vol / realized_vol
  capped_scale = clip(vol_scale, 0.5, 2.0)
  emit base_signal * capped_scale

portfolio main:
  weights = rank(vol_adjusted_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
