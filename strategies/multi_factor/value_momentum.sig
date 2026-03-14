// Value + Momentum Strategy
// Classic two-factor model combining value and momentum.
// These factors have historically low correlation, providing
// diversification benefits.
// Reference: Asness, Moskowitz, Pedersen (2013)

data:
  px: load csv from "data/prices.csv"
  book_to_market: load csv from "data/book_to_market.csv"

params:
  mom_lookback = 126
  value_weight = 0.5
  mom_weight = 0.5

signal value:
  v = zscore(book_to_market)
  emit winsor(v, p=0.01)

signal momentum:
  r = ret(px, mom_lookback)
  m = zscore(r)
  emit winsor(m, p=0.01)

signal value_momentum:
  combined = value_weight * value + mom_weight * momentum
  emit combined

portfolio main:
  weights = rank(value_momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
