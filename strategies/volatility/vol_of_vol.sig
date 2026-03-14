// Volatility of Volatility Strategy
// Trades based on changes in volatility regime.
// Captures the tendency for vol clustering and regime persistence.

data:
  px: load csv from "data/prices.csv"

params:
  short_vol = 10
  long_vol = 60
  hold = 5

signal vol_regime:
  r = ret(px, 1)
  vol_short = rolling_std(r, short_vol)
  vol_long = rolling_std(r, long_vol)
  vol_ratio = vol_short / vol_long
  vol_z = ts_zscore(vol_ratio, long_vol)
  sig = -vol_z
  emit clip(sig, -3, 3)

portfolio main:
  weights = rank(vol_regime).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
