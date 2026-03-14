# MACD Strategy

Trading using Moving Average Convergence Divergence.

## Strategy Overview

MACD measures momentum through the relationship between two moving averages.

## The Indicator

```
MACD Line = 12-day EMA - 26-day EMA
Signal Line = 9-day EMA of MACD Line
Histogram = MACD Line - Signal Line
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

signal macd:
  // Calculate MACD
  ema_12 = ema(prices, 12)
  ema_26 = ema(prices, 26)
  macd_line = ema_12 - ema_26
  signal_line = ema(macd_line, 9)
  histogram = macd_line - signal_line

  // Positive histogram = bullish momentum
  emit neutralize(zscore(histogram), by=sectors)

portfolio macd_strategy:
  weights = rank(macd).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  costs = tc.bps(10)

  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## MACD Crossover

```sig
signal macd_crossover:
  ema_12 = ema(prices, 12)
  ema_26 = ema(prices, 26)
  macd_line = ema_12 - ema_26
  signal_line = ema(macd_line, 9)

  // Crossover detection
  macd_above = macd_line > signal_line
  crossover_up = macd_above and not(lag(macd_above, 1))
  crossover_down = not(macd_above) and lag(macd_above, 1)

  signal = where(crossover_up, 1, where(crossover_down, -1, 0))

  emit zscore(signal)
```

## MACD Divergence

```sig
signal macd_divergence:
  ema_12 = ema(prices, 12)
  ema_26 = ema(prices, 26)
  macd_line = ema_12 - ema_26

  // Price making new lows but MACD not = bullish divergence
  price_low = prices == rolling_min(prices, 20)
  macd_not_low = macd_line > rolling_min(macd_line, 20)
  bullish_div = price_low and macd_not_low

  price_high = prices == rolling_max(prices, 20)
  macd_not_high = macd_line < rolling_max(macd_line, 20)
  bearish_div = price_high and macd_not_high

  emit where(bullish_div, 1, where(bearish_div, -1, 0))
```

## Expected Results

```
Annual Return: 4.8%
Sharpe: 0.45
Turnover: 450%
```

## See Also

- [Moving Averages](moving-averages.md)
- [Trend Following](../momentum/trend-following.md)
