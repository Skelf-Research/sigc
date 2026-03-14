// Factor Timing Strategy
// Dynamically allocates across factors based on recent performance.
// When momentum is working, increase momentum exposure; etc.
// Implements simple trend-following on factor returns.

data:
  px: load csv from "data/prices.csv"
  book_to_market: load csv from "data/book_to_market.csv"

params:
  factor_trend_window = 63
  base_lookback = 126

signal value_factor:
  v = zscore(book_to_market)
  emit v

signal momentum_factor:
  m = zscore(ret(px, base_lookback))
  emit m

signal timed_allocation:
  value_trend = rolling_mean(value_factor, factor_trend_window)
  mom_trend = rolling_mean(momentum_factor, factor_trend_window)
  value_score = ts_rank(value_trend, factor_trend_window)
  mom_score = ts_rank(mom_trend, factor_trend_window)
  value_weight = value_score
  mom_weight = mom_score
  total = value_weight + mom_weight
  combined = (value_weight * value_factor + mom_weight * momentum_factor) / total
  emit winsor(combined, p=0.01)

portfolio main:
  weights = rank(timed_allocation).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
