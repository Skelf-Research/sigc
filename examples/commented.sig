// Test file with comments
// This demonstrates comment support

data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 5

// Signal definition
signal momentum:
  returns = ret(prices, lookback)
  score = zscore(returns)
  emit winsor(score, p=0.01)

// Portfolio construction
portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
