// SAFE — this program MUST compile.
//
// Identical in shape to the leaky examples but uses only backward-looking
// operators: ret(px, periods=20) reads px[t] vs px[t-20] and rolling_std
// reads a trailing window. Every value is point-in-time (peek = 0), so no
// look-ahead bias is possible. This guards against false positives in the
// temporal type checker.

data:
  px: load parquet from "prices.parquet"

signal mom:
  r   = ret(px, periods=20)
  vol = rolling_std(r, window=60)
  emit zscore(r / vol)
