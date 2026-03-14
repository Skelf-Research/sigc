# Price Momentum Strategy

The classic momentum strategy based on past price returns.

## Strategy Overview

Buy stocks with strong recent performance, short stocks with weak performance.

## The Signal

### 12-1 Momentum

The most common momentum signal uses 12-month returns, skipping the most recent month:

```sig
signal momentum_12_1:
  // 12-month return
  ret_12m = ret(prices, 252)

  // Skip last month (1-month reversal effect)
  ret_1m = ret(prices, 21)

  // Net momentum
  momentum = ret_12m - ret_1m

  emit zscore(momentum)
```

### Why Skip Last Month?

Short-term reversals contaminate momentum:
- Stocks up recently tend to reverse in the next month
- Skipping avoids this "mean reversion" noise

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Momentum signal
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  raw = ret_12m - ret_1m

  // Sector neutralize to avoid sector bets
  emit neutralize(zscore(raw), by=sectors)

// Portfolio
portfolio price_momentum:
  weights = rank(momentum).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Variations

### Different Lookback Periods

```sig
// Shorter momentum (faster signal)
signal momentum_6_1:
  ret_6m = ret(prices, 126)
  ret_1m = ret(prices, 21)
  emit zscore(ret_6m - ret_1m)

// Longer momentum (slower signal)
signal momentum_18_1:
  ret_18m = ret(prices, 378)
  ret_1m = ret(prices, 21)
  emit zscore(ret_18m - ret_1m)
```

### Risk-Adjusted Momentum

```sig
signal sharpe_momentum:
  // Risk-adjusted returns
  ret_12m = ret(prices, 252)
  vol_12m = rolling_std(ret(prices, 1), 252) * sqrt(252)
  sharpe = ret_12m / vol_12m

  emit zscore(sharpe)
```

### Residual Momentum

```sig
signal residual_momentum:
  // Market-adjusted returns
  stock_ret = ret(prices, 252)
  market_ret = ret(market, 252)
  beta = rolling_cov(ret(prices, 1), ret(market, 1), 60) /
         rolling_var(ret(market, 1), 60)

  // Residual return
  residual = stock_ret - beta * market_ret

  emit zscore(residual)
```

## Parameter Sensitivity

```sig
params:
  lookback: [126, 189, 252, 315]  // 6, 9, 12, 15 months
  skip: [0, 21, 42]               // 0, 1, 2 months

signal momentum:
  ret_full = ret(prices, lookback)
  ret_skip = where(skip > 0, ret(prices, skip), 0)
  emit zscore(ret_full - ret_skip)

portfolio optimized:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(train_years=5, test_years=1) from 2010-01-01 to 2024-12-31
```

## Expected Results

```
Backtest Results: price_momentum
================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 156%
  Annual Return: 6.8%
  Annual Volatility: 10.2%
  Sharpe Ratio: 0.67

Risk:
  Max Drawdown: -22.4%
  Avg Drawdown: -5.8%

Turnover:
  Annual Turnover: 285%
  Avg Holding Period: 64 days
```

## Risk Considerations

### Momentum Crashes

Momentum can reverse sharply at market turning points:

```sig
// Add regime filter
signal trend_filter:
  ma_50 = rolling_mean(market, 50)
  ma_200 = rolling_mean(market, 200)
  bull = ma_50 > ma_200
  emit bull

signal safe_momentum:
  mom = momentum
  // Reduce in bear markets
  scale = where(trend_filter, 1.0, 0.5)
  emit mom * scale
```

### High Volatility Periods

```sig
// Scale down when volatility spikes
signal vol_scaled_momentum:
  mom = momentum
  vix_high = vix > 25
  scale = where(vix_high, 0.7, 1.0)
  emit mom * scale
```

## Enhancements

### Combine with Other Factors

```sig
signal enhanced_momentum:
  mom = momentum
  qual = zscore(roe)  // Quality filter

  // Only momentum in quality stocks
  emit where(qual > 0, mom, mom * 0.5)
```

## See Also

- [Industry Momentum](industry-momentum.md)
- [Trend Following](trend-following.md)
- [Multi-Factor Strategies](../multi-factor/index.md)
