// Price Breakout Strategy
// Identifies stocks breaking out to new highs or lows.
// Long stocks near 52-week highs, short those near 52-week lows.
// Based on: George & Hwang (2004) "The 52-Week High and Momentum Investing"

data:
  px: load csv from "data/prices.csv"

params:
  lookback = 252
  hold = 21

signal breakout:
  high_52w = rolling_max(px, lookback)
  low_52w = rolling_min(px, lookback)
  dist_to_high = (px - low_52w) / (high_52w - low_52w)
  sig = zscore(dist_to_high)
  emit winsor(sig, p=0.01)

portfolio main:
  weights = rank(breakout).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
