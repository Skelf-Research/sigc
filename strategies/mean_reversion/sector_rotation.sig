// Sector Rotation Mean Reversion Strategy
// Bets on sector mean reversion: underweight recent sector winners,
// overweight recent sector losers. Assumes sector returns revert
// over intermediate horizons.

data:
  px: load csv from "data/prices.csv"
  sector: load csv from "data/sectors.csv"

params:
  lookback = 21
  reversion_window = 63

signal sector_mean_reversion:
  r = ret(px, lookback)
  r_demean = demean(r)
  sig = -zscore(r_demean)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(sector_mean_reversion).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
