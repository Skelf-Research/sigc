// Residual Momentum Strategy
// Momentum on market-residualized returns (idiosyncratic momentum).
// Uses a simplified approach: momentum minus market return.
// Reference: Blitz, Huij, Martens (2011)

data:
  px: load csv from "data/prices.csv"
  market: load csv from "data/market.csv"

params:
  mom_lookback = 126
  hold = 21

signal residual_mom:
  stock_ret = ret(px, mom_lookback)
  mkt_ret = ret(market, mom_lookback)
  residual = stock_ret - mkt_ret
  sig = zscore(residual)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(residual_mom).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
