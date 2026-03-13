data:
  px:   load price from "s3://example/prices.parquet" adjust=split_div
  sec:  load sector from "s3://example/sector.parquet" dtype=category

params:
  lookback = 126
  hold     = 21

signal momentum:
  ret = log(px / lag(px, lookback))
  xs  = as_xs(ret)
  xs  = zscore(xs)
  xs  = neutralize(xs, by=sec)
  emit winsor(xs, p=0.01)

portfolio longshort:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.02)
  costs   = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest rebal=hold benchmark=SPY from 2015-01-01 to 2024-12-31
