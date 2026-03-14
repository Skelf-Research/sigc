// Short-Term Reversal Strategy
// Exploits the well-documented short-term reversal effect where
// recent losers outperform recent winners over weekly horizons.
// Reference: Jegadeesh (1990), Lehmann (1990)

data:
  px: load csv from "data/prices.csv"

params:
  lookback = 5
  hold = 5

signal reversal:
  r = ret(px, lookback)
  rev = -zscore(r)
  emit winsor(rev, p=0.01)

portfolio main:
  weights = rank(reversal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
