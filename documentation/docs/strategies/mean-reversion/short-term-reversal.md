# Short-Term Reversal Strategy

Profit from weekly price reversals.

## Strategy Overview

Stocks that performed poorly last week tend to outperform next week, and vice versa. This is one of the most robust anomalies in finance.

## The Signal

```sig
signal short_term_reversal:
  // Last week's return
  ret_5d = ret(prices, 5)

  // Negative return = buy (expect reversal up)
  emit -zscore(ret_5d)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Short-term reversal
signal str:
  // 5-day return
  ret_5d = ret(prices, 5)

  // Revert (buy losers, short winners)
  emit neutralize(-zscore(ret_5d), by=sectors)

portfolio short_term_reversal:
  weights = rank(str).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  // Weekly rebalancing
  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

## Why It Works

1. **Liquidity shocks**: Forced sellers push prices down temporarily
2. **Overreaction**: Investors overreact to short-term news
3. **Bid-ask bounce**: Prices bounce between bid and ask
4. **Market microstructure**: Short-term noise

## Variations

### Volume-Weighted Reversal

```sig
signal volume_weighted:
  ret_5d = ret(prices, 5)

  // High volume = more conviction in reversal
  vol_ratio = volume / rolling_mean(volume, 20)
  high_volume = vol_ratio > 1.5

  // Stronger signal when volume is high
  signal = -ret_5d * where(high_volume, 1.5, 1.0)

  emit zscore(signal)
```

### Idiosyncratic Reversal

```sig
signal idio_reversal:
  // Remove market component
  stock_ret = ret(prices, 5)
  market_ret = ret(market, 5)

  // Idiosyncratic return
  idio_ret = stock_ret - market_ret

  // Revert idiosyncratic component
  emit -zscore(idio_ret)
```

### Multi-Day Reversal

```sig
signal multi_day:
  // Multiple short-term windows
  ret_1d = ret(prices, 1)
  ret_3d = ret(prices, 3)
  ret_5d = ret(prices, 5)

  // Weighted combination
  combined = -0.2 * zscore(ret_1d) - 0.3 * zscore(ret_3d) - 0.5 * zscore(ret_5d)

  emit neutralize(combined, by=sectors)
```

### Extreme Moves Only

```sig
signal extreme_reversal:
  ret_5d = ret(prices, 5)

  // Only trade extreme moves
  extreme_down = ret_5d < quantile(ret_5d, 0.1)  // Bottom 10%
  extreme_up = ret_5d > quantile(ret_5d, 0.9)    // Top 10%

  signal = where(extreme_down, -ret_5d,
           where(extreme_up, -ret_5d, 0))

  emit zscore(signal)
```

## Liquidity Filter

```sig
signal liquid_reversal:
  ret_5d = ret(prices, 5)

  // Only trade liquid stocks
  adv = rolling_mean(volume * prices, 20)
  liquid = adv > 1000000  // $1M+ daily volume

  signal = where(liquid, -ret_5d, 0)

  emit zscore(signal)
```

## Intraday vs Overnight

```sig
signal overnight_reversal:
  // Overnight return (close to open)
  overnight_ret = (open - lag(close, 1)) / lag(close, 1)

  // Intraday return (open to close)
  intraday_ret = (close - open) / open

  // Overnight moves revert more
  emit -zscore(overnight_ret)
```

## With Momentum Filter

Avoid reverting stocks with strong momentum:

```sig
signal filtered_reversal:
  // Short-term reversal
  ret_5d = ret(prices, 5)

  // Long-term momentum
  ret_60d = ret(prices, 60)

  // Don't fight strong momentum
  strong_up = ret_60d > quantile(ret_60d, 0.8)
  strong_down = ret_60d < quantile(ret_60d, 0.2)

  // Reduce reversal in momentum stocks
  signal = -ret_5d
  signal = where(strong_up and ret_5d < 0, signal * 0.5, signal)
  signal = where(strong_down and ret_5d > 0, signal * 0.5, signal)

  emit zscore(signal)
```

## Expected Results

```
Backtest Results: short_term_reversal
=====================================
Period: 2015-01-01 to 2024-12-31

Returns:
  Total Return: 38%
  Annual Return: 3.4%
  Annual Volatility: 6.8%
  Sharpe Ratio: 0.50

Turnover:
  Annual Turnover: 1040%
  Avg Holding Period: 5 days

Characteristics:
  Win Rate: 52%
  Profit Factor: 1.15
```

## Transaction Costs

High turnover means costs matter significantly:

```
Gross Return: 5.2%
Costs (10bps): -1.0%
Costs (20bps): -2.1%
Net Return: 3.1% - 4.2%
```

## Risk Considerations

### Momentum Stocks

Reversal fails in momentum stocks:

```sig
// Exclude strong momentum
signal safe_reversal:
  str = -ret(prices, 5)
  mom = ret(prices, 60)

  // Exclude top/bottom momentum quintiles
  strong_mom = abs(mom) > quantile(abs(mom), 0.8)

  emit where(strong_mom, 0, zscore(str))
```

### Event Risk

Earnings, M&A can cause permanent moves:

```sig
// Exclude around earnings
signal event_filtered:
  str = -ret(prices, 5)

  // Avoid if big move (might be event)
  big_move = abs(ret(prices, 1)) > 0.10

  emit where(big_move, 0, zscore(str))
```

## See Also

- [Statistical Reversion](statistical-reversion.md)
- [RSI Reversion](rsi-reversion.md)
