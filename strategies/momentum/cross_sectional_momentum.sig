// Cross-Sectional Momentum Strategy
// Ranks stocks by their relative momentum within the cross-section.
// Uses industry neutralization to remove sector bets.
// Reference: Jegadeesh & Titman (1993)

data:
  px: load csv from "data/prices.csv"
  sector: load csv from "data/sectors.csv"

params:
  lookback = 126
  skip = 5
  hold = 21

signal xsmom:
  r = ret(px, lookback)
  mom = lag(r, skip)
  z = zscore(mom)
  neutral = neutralize(z, by=sector)
  emit winsor(neutral, p=0.01)

portfolio main:
  weights = rank(xsmom).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
