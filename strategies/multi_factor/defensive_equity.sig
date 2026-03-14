// Defensive Equity Strategy
// Combines low volatility, quality, and value for a defensive profile.
// Aims for equity-like returns with bond-like risk.
// Reference: Pim van Vliet - "High Returns from Low Risk"

data:
  px: load csv from "data/prices.csv"
  roe: load csv from "data/roe.csv"
  earnings_yield: load csv from "data/earnings_yield.csv"

params:
  vol_window = 60
  low_vol_weight = 0.40
  quality_weight = 0.35
  value_weight = 0.25

signal low_vol:
  daily_ret = ret(px, 1)
  vol = rolling_std(daily_ret, vol_window)
  lv = -zscore(vol)
  emit winsor(lv, p=0.01)

signal quality:
  q = zscore(roe)
  emit winsor(q, p=0.01)

signal value:
  v = zscore(earnings_yield)
  emit winsor(v, p=0.01)

signal defensive:
  def = low_vol_weight * low_vol + quality_weight * quality + value_weight * value
  emit def

portfolio main:
  weights = rank(defensive).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
