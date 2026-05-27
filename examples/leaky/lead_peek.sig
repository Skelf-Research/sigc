// LOOK-AHEAD BIAS — this program MUST fail to compile.
//
// Normalizing today's price against a price five bars in the FUTURE.
// lag(px, periods=-5) is a *lead*: it evaluates px[t+5]. The deviation
// (px / future) therefore reads five bars of unobservable data.
//
// sigc rejects this at compile time: the emitted signal has peek = 5.

data:
  px: load parquet from "prices.parquet"

signal peek5:
  future = lag(px, periods=-5)
  emit zscore(px / future)
