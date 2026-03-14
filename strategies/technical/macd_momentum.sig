// MACD Momentum Strategy
// Uses MACD (Moving Average Convergence Divergence) as a
// momentum indicator. MACD measures the relationship between
// two EMAs and includes a signal line.
// Signal: Long when MACD > Signal Line

data:
  px: load csv from "data/prices.csv"

params:
  fast = 12
  slow = 26
  signal_period = 9
  hold = 5

signal macd_signal:
  macd_line = macd(px, fast, slow, signal_period)
  z = zscore(macd_line)
  emit winsor(z, p=0.01)

portfolio main:
  weights = rank(macd_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
