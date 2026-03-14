// Bollinger Band Mean Reversion Strategy
// Uses Bollinger Bands to identify overbought/oversold conditions.
// Shorts when price exceeds upper band, longs when below lower band.

data:
  px: load csv from "data/prices.csv"

params:
  window = 20
  num_std = 2.0
  hold = 5

signal bollinger_signal:
  middle = rolling_mean(px, window)
  std = rolling_std(px, window)
  upper = middle + num_std * std
  lower = middle - num_std * std
  band_zscore = (px - middle) / std
  sig = -band_zscore
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(bollinger_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
