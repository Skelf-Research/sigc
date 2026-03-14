// Time-Series Momentum Strategy
// Classic 12-1 month momentum: long assets with positive past returns,
// short assets with negative past returns. Skips the most recent month
// to avoid short-term reversal effects.
// Reference: Moskowitz, Ooi, Pedersen (2012) "Time Series Momentum"

data:
  px: load csv from "data/prices.csv"

params:
  lookback = 252
  skip = 21
  hold = 21

signal tsmom:
  past_price = lag(px, lookback)
  recent_price = lag(px, skip)
  momentum_ret = (recent_price - past_price) / past_price
  z = zscore(momentum_ret)
  emit winsor(z, p=0.01)

portfolio main:
  weights = rank(tsmom).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
