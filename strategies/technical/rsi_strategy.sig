// RSI Strategy
// Relative Strength Index based mean reversion.
// Buys oversold (low RSI), sells overbought (high RSI).
// RSI is a bounded oscillator [0, 100], classic levels:
// Oversold: RSI < 30, Overbought: RSI > 70

data:
  px: load csv from "data/prices.csv"

params:
  rsi_period = 14
  hold = 5

signal rsi_signal:
  rsi_val = rsi(px, rsi_period)
  centered = rsi_val - 50
  scaled = centered / 25
  sig = -scaled
  emit clip(sig, -2, 2)

portfolio main:
  weights = rank(rsi_signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
