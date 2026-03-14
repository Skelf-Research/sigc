// Trend Following Strategy
// Classic moving average crossover system.
// Goes long when short MA > long MA, short when opposite.
// Simple but effective trend-following approach.

data:
  px: load csv from "data/prices.csv"

params:
  short_win = 50
  long_win = 200
  hold = 5

signal trend:
  ma_short = rolling_mean(px, short_win)
  ma_long = rolling_mean(px, long_win)
  trend_diff = ma_short - ma_long
  normalized_trend = trend_diff / ma_long
  z = zscore(normalized_trend)
  emit winsor(z, p=0.01)

portfolio main:
  weights = rank(trend).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
