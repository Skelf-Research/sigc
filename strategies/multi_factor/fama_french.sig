// Fama-French Style Multi-Factor Strategy
// Implements a simplified version of Fama-French factors:
// Size (SMB): Small minus Big
// Value (HML): High minus Low book-to-market
// Momentum (UMD): Up minus Down
// Reference: Fama & French (1993, 2015)

data:
  px: load csv from "data/prices.csv"
  market_cap: load csv from "data/market_cap.csv"
  book_to_market: load csv from "data/book_to_market.csv"

params:
  mom_lookback = 252
  size_weight = 0.25
  value_weight = 0.35
  mom_weight = 0.40

signal size_factor:
  size = -zscore(log(market_cap))
  emit winsor(size, p=0.01)

signal value_factor:
  value = zscore(book_to_market)
  emit winsor(value, p=0.01)

signal momentum_factor:
  r12 = ret(px, mom_lookback)
  r1 = ret(px, 21)
  mom = zscore(r12 - r1)
  emit winsor(mom, p=0.01)

signal fama_french:
  ff = size_weight * size_factor + value_weight * value_factor + mom_weight * momentum_factor
  emit ff

portfolio main:
  weights = rank(fama_french).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
