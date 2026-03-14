// Industry Momentum Strategy
// Exploits momentum at the industry level rather than stock level.
// Industries that have performed well continue to outperform.
// Lower turnover than stock-level momentum.

data:
  px: load csv from "data/prices.csv"
  sector: load csv from "data/sectors.csv"

params:
  lookback = 126
  skip = 21
  hold = 21

signal industry_mom:
  raw_mom = ret(px, lookback)
  mom = lag(raw_mom, skip)
  relative_strength = demean(mom)
  sig = zscore(mom) + 0.5 * zscore(relative_strength)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(industry_mom).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
