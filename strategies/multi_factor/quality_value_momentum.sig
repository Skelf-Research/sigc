// Quality + Value + Momentum Strategy
// Three-factor model combining profitability, value, and momentum.
// Quality screens help avoid "value traps" and momentum crashes.
// Reference: AQR research on Quality Minus Junk (QMJ)

data:
  px: load csv from "data/prices.csv"
  roe: load csv from "data/roe.csv"
  earnings_yield: load csv from "data/earnings_yield.csv"
  debt_to_equity: load csv from "data/debt_to_equity.csv"

params:
  mom_lookback = 126
  quality_weight = 0.35
  value_weight = 0.35
  mom_weight = 0.30

signal quality:
  profitability = zscore(roe)
  safety = -zscore(debt_to_equity)
  q = 0.6 * profitability + 0.4 * safety
  emit winsor(q, p=0.01)

signal value:
  v = zscore(earnings_yield)
  emit winsor(v, p=0.01)

signal momentum:
  r = ret(px, mom_lookback)
  m = zscore(r)
  emit winsor(m, p=0.01)

signal qvm:
  combined = quality_weight * quality + value_weight * value + mom_weight * momentum
  emit combined

portfolio main:
  weights = rank(qvm).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
