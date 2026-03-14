// Variance Risk Premium Strategy
// Approximates the variance risk premium using realized vs implied
// volatility relationship. In practice, uses vol-of-vol as a proxy.
// Stocks with stable volatility tend to outperform.

data:
  px: load csv from "data/prices.csv"

params:
  vol_window = 21
  vol_of_vol_window = 63
  hold = 21

signal vol_stability:
  daily_ret = ret(px, 1)
  vol = rolling_std(daily_ret, vol_window)
  vol_of_vol = rolling_std(vol, vol_of_vol_window)
  sig = -zscore(vol_of_vol)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(vol_stability).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
