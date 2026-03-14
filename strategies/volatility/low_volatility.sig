// Low Volatility Anomaly Strategy
// Exploits the low volatility anomaly: low-vol stocks earn
// higher risk-adjusted returns than high-vol stocks.
// Reference: Ang, Hodrick, Xing, Zhang (2006)

data:
  px: load csv from "data/prices.csv"

params:
  vol_window = 60
  hold = 21

signal low_vol:
  daily_ret = ret(px, 1)
  vol = rolling_std(daily_ret, vol_window)
  sig = -zscore(vol)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(low_vol).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
