// Momentum + Quality Strategy
// Combines price momentum with quality filters to avoid
// low-quality momentum stocks that tend to crash.
// Quality defined as: profitability + earnings stability

data:
  px: load csv from "data/prices.csv"
  roe: load csv from "data/roe.csv"
  earnings_var: load csv from "data/earnings_variance.csv"

params:
  mom_lookback = 126
  quality_weight = 0.3

signal momentum:
  r = ret(px, mom_lookback)
  emit zscore(r)

signal quality:
  roe_z = zscore(roe)
  stability_z = -zscore(earnings_var)
  q = 0.5 * roe_z + 0.5 * stability_z
  emit q

signal mom_quality:
  combined = (1.0 - quality_weight) * momentum + quality_weight * quality
  emit winsor(combined, p=0.01)

portfolio main:
  weights = rank(mom_quality).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
