data:
  earnings_yield: load feature from "s3://example/earnings_yield.parquet"
  roa:            load feature from "s3://example/roa.parquet"
  market_cap:     load feature from "s3://example/market_cap.parquet"

signal value:
  emit zscore(as_xs(earnings_yield))

signal quality:
  emit zscore(as_xs(roa))

signal size:
  emit -zscore(as_xs(log(market_cap)))

signal combo:
  emit 0.5 * value + 0.3 * quality + 0.2 * size

portfolio balanced:
  weights = rank(combo).long_short(top=0.25, bottom=0.25, cap=0.015)
  backtest rebal=21d benchmark=SPY from 2016-01-01 to 2024-12-31
