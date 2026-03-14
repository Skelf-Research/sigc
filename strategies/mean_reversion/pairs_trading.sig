// Statistical Pairs Trading Strategy
// Identifies deviation from historical price relationships
// and bets on convergence. Uses time-series z-score to detect
// when spread is extended.

data:
  px: load csv from "data/prices.csv"

params:
  zscore_window = 60
  entry_threshold = 2
  hold = 1

signal spread_zscore:
  ma = rolling_mean(px, zscore_window)
  std = rolling_std(px, zscore_window)
  z = (px - ma) / std
  sig = -z
  emit clip(sig, -3, 3)

portfolio main:
  weights = rank(spread_zscore).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
