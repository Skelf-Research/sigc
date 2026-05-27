// LOOK-AHEAD BIAS — this program MUST fail to compile.
//
// The textbook leak: using tomorrow's return as today's signal.
// ret(px, periods=-1) evaluates px[t+1] vs px[t], so the value at bar t
// depends on a price that is not observable until bar t+1.
//
// sigc rejects this at compile time: the emitted signal has peek = 1.

data:
  px: load parquet from "prices.parquet"

signal next_day:
  emit zscore(ret(px, periods=-1))
