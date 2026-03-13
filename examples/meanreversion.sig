data:
  px: load price from "s3://example/prices.parquet" adjust=split_div

params:
  fast = 5
  slow = 60

signal meanrev:
  r_fast = ret(px, fast)
  r_slow = ret(px, slow)
  z      = -zscore(as_xs(r_fast - r_slow))
  emit clip(z, -3, 3)

portfolio vol_target:
  weights = scale_vol(meanrev, target_ann_vol=0.1, lookback=252)
  backtest rebal=5d benchmark=SPY from 2018-01-01 to 2024-12-31
