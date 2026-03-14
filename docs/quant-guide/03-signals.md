# Chapter 3: Building Trading Signals

A signal is a numerical score predicting future asset performance. This chapter covers how to develop effective signals from various data sources.

## What Makes a Good Signal

### Characteristics of Strong Signals

1. **Predictive power**: Correlation with future returns
2. **Economic rationale**: Explainable reason for working
3. **Robustness**: Works across time periods and markets
4. **Low correlation**: Independent from existing signals
5. **Capacity**: Can be traded at meaningful size

### Signal Evaluation Metrics

**Information Coefficient (IC)**: Correlation between signal and forward returns
```
IC = corr(signal_t, return_{t+1})
```

Good signals have IC > 0.03 (daily) or IC > 0.10 (monthly).

**IC Information Ratio (ICIR)**: IC divided by its standard deviation
```
ICIR = mean(IC) / std(IC)
```

ICIR > 0.5 indicates a consistent signal.

## Technical Signals

Technical signals use price and volume data. They're simple but often effective.

### Momentum

**Premise**: Winners keep winning, losers keep losing.

```sig
data prices = load("prices.csv")

// Simple momentum (past return)
signal mom_20 = prices / lag(prices, 20) - 1
signal mom_60 = prices / lag(prices, 60) - 1

// Smoother momentum
signal mom_smooth = sma(mom_20, 5)

output mom_smooth
```

**Variations**:
```sig
// Exclude most recent week (avoid short-term reversal)
signal mom_skip = lag(prices, 5) / lag(prices, 65) - 1

// Volatility-adjusted
signal ret = prices / lag(prices, 1) - 1
signal vol = std(ret, 60)
signal sharpe_mom = mom_60 / vol

// Sector-neutral momentum
signal sector_mom = mom_60 - sma(mom_60, 10)  // Simplified
```

### Mean Reversion

**Premise**: Prices revert to some "fair value".

```sig
data prices = load("prices.csv")

// Distance from moving average
signal sma_20 = sma(prices, 20)
signal deviation = (prices - sma_20) / sma_20

// Z-score (standardized deviation)
signal zscore = (prices - sma(prices, 60)) / std(prices, 60)

// Bollinger Band position
signal upper = sma_20 + 2 * std(prices, 20)
signal lower = sma_20 - 2 * std(prices, 20)
signal bb_pos = (prices - lower) / (upper - lower)

// Mean reversion signal (short when high, long when low)
signal mean_rev = -zscore

output mean_rev
```

### Trend Following

**Premise**: Follow established trends.

```sig
data prices = load("prices.csv")

// Moving average crossover
signal sma_fast = sma(prices, 20)
signal sma_slow = sma(prices, 60)
signal trend = sma_fast / sma_slow - 1

// Breakout (distance from recent high)
signal high_20 = max(prices, 20)
signal breakout = prices / high_20 - 1

// ADX-style trend strength
signal returns = prices / lag(prices, 1) - 1
signal trend_str = abs(sma(returns, 20)) / std(returns, 20)

output trend
```

### Volatility Signals

**Premise**: Low volatility stocks outperform on risk-adjusted basis.

```sig
data prices = load("prices.csv")

signal returns = prices / lag(prices, 1) - 1
signal vol = std(returns, 60)

// Low volatility signal (negative because we want low vol)
signal low_vol = -vol

// Volatility change
signal vol_change = vol / lag(vol, 20) - 1

output low_vol
```

### Volume Signals

**Premise**: Volume confirms price moves.

```sig
data prices = load("prices.csv")
data volume = load("volume.csv")

signal returns = prices / lag(prices, 1) - 1

// Volume-weighted returns
signal vwap_signal = sma(returns * volume, 20) / sma(volume, 20)

// Abnormal volume
signal avg_vol = sma(volume, 60)
signal abnormal_vol = volume / avg_vol - 1

// Volume trend (accumulation/distribution)
signal vol_trend = sma(volume, 5) / sma(volume, 20) - 1

output vwap_signal
```

### RSI (Relative Strength Index)

**Premise**: Overbought/oversold conditions predict reversals.

```sig
data prices = load("prices.csv")

signal changes = prices - lag(prices, 1)
signal gains = max(changes, 0)
signal losses = max(-changes, 0)

// Smoothed average gains and losses
signal avg_gain = ema(gains, 14)
signal avg_loss = ema(losses, 14)

// RSI calculation
signal rs = avg_gain / avg_loss
signal rsi = 100 - (100 / (1 + rs))

// Mean reversion based on RSI
signal rsi_signal = 50 - rsi  // Long when oversold, short when overbought

output rsi_signal
```

## Fundamental Signals

Fundamental signals use company financial data.

### Value

**Premise**: Cheap stocks outperform expensive ones.

```sig
data prices = load("prices.csv")
data earnings = load("earnings.csv")    // Earnings per share
data book = load("book_value.csv")      // Book value per share
data sales = load("sales.csv")          // Sales per share

// P/E ratio (lower is cheaper)
signal pe = prices / earnings
signal value_pe = -pe  // Negative because we want low P/E

// P/B ratio
signal pb = prices / book
signal value_pb = -pb

// P/S ratio
signal ps = prices / sales
signal value_ps = -ps

// Composite value
signal value = (zscore(value_pe) + zscore(value_pb) + zscore(value_ps)) / 3

output value
```

### Quality

**Premise**: High-quality companies outperform.

```sig
data net_income = load("net_income.csv")
data total_equity = load("equity.csv")
data total_assets = load("assets.csv")
data operating_cf = load("cash_flow.csv")

// Return on Equity
signal roe = net_income / total_equity

// Return on Assets
signal roa = net_income / total_assets

// Accruals (lower is better - cash earnings)
signal accruals = (net_income - operating_cf) / total_assets
signal quality_accruals = -accruals

// Composite quality
signal quality = (zscore(roe) + zscore(roa) + zscore(quality_accruals)) / 3

output quality
```

### Growth

**Premise**: Companies with growing fundamentals outperform.

```sig
data earnings = load("earnings.csv")
data sales = load("sales.csv")

// Year-over-year growth
signal earnings_growth = earnings / lag(earnings, 252) - 1
signal sales_growth = sales / lag(sales, 252) - 1

// Acceleration
signal earnings_accel = earnings_growth - lag(earnings_growth, 63)

// Composite growth
signal growth = (zscore(earnings_growth) + zscore(sales_growth)) / 2

output growth
```

## Alternative Data Signals

Alternative data provides unique insights beyond traditional sources.

### Sentiment

```sig
data news_sentiment = load("sentiment.csv")  // NLP sentiment scores

// Raw sentiment
signal sentiment = sma(news_sentiment, 5)

// Sentiment change
signal sentiment_delta = sentiment - lag(sentiment, 20)

// Extreme sentiment (contrarian)
signal extreme = -abs(zscore(sentiment, 60))

output sentiment
```

### Web Traffic

```sig
data web_visits = load("web_traffic.csv")  // Company website visits

// Traffic growth
signal traffic_growth = web_visits / lag(web_visits, 30) - 1

// Abnormal traffic
signal normal_traffic = sma(web_visits, 90)
signal abnormal = (web_visits - normal_traffic) / std(web_visits, 90)

output abnormal
```

### Satellite/Geospatial

```sig
data parking_lots = load("parking_counts.csv")  // Retail store parking
data oil_storage = load("tank_levels.csv")      // Oil storage tank fill

// Parking lot activity (proxy for sales)
signal retail_signal = parking_lots / lag(parking_lots, 30) - 1

// Oil inventory surprise
signal inventory_signal = oil_storage - sma(oil_storage, 30)

output retail_signal
```

## Signal Combination

Combining signals improves robustness and reduces overfitting.

### Simple Average

```sig
// Equal-weighted combination
signal combined = (zscore(momentum) + zscore(value) + zscore(quality)) / 3
```

### Weighted Average

```sig
// Weight by historical performance
signal combined = 0.5 * zscore(momentum) + 0.3 * zscore(value) + 0.2 * zscore(quality)
```

### Conditional Combination

```sig
// Use momentum in trending markets, mean reversion in ranging
signal vol = std(returns, 60)
signal vol_regime = vol / sma(vol, 252)

signal combined = if(vol_regime > 1, mean_reversion, momentum)
```

### Orthogonalization

Remove overlap between signals for true diversification:

```sig
// Make value orthogonal to momentum
signal value_resid = value - beta(value, momentum) * momentum
```

## Signal Decay

Signals lose predictive power over time. Understanding decay helps with portfolio construction.

### Measuring Decay

Calculate IC at different lags:
- IC at lag 1 (1 day forward)
- IC at lag 5 (1 week forward)
- IC at lag 20 (1 month forward)

Fast-decaying signals need frequent rebalancing.
Slow-decaying signals can be traded less often (lower costs).

### Decay Characteristics

| Signal Type | Typical Decay |
|-------------|---------------|
| Short-term momentum | 1-5 days |
| Mean reversion | 1-10 days |
| Earnings momentum | 1-3 months |
| Value | 6-12 months |

### Implications

```sig
// Fast decay → higher turnover
signal fast = rsi_signal  // Rebalance daily

// Slow decay → lower turnover
signal slow = value       // Rebalance monthly
```

## Avoiding Overfitting

The biggest risk in signal development is overfitting - finding patterns that don't persist.

### Red Flags

1. **Too many parameters**: Each parameter is a degree of freedom
2. **Perfect backtest**: If it's too good, it's probably overfit
3. **No economic rationale**: "The data told me so" is not enough
4. **Fails out-of-sample**: Works on training data, fails on test data

### Best Practices

1. **Start with hypothesis**: Decide what you're testing before looking at data
2. **Simple models**: Fewer parameters = less overfitting
3. **Out-of-sample testing**: Reserve data for validation
4. **Multiple testing correction**: Account for many hypotheses tested
5. **Paper trade**: Test on live data before deploying capital

### Parameter Selection

```sig
// Bad: Overfit parameters
signal mom = prices / lag(prices, 17) - 1  // Why 17?

// Better: Round numbers with rationale
signal mom = prices / lag(prices, 20) - 1  // ~1 month
signal mom = prices / lag(prices, 60) - 1  // ~1 quarter
signal mom = prices / lag(prices, 252) - 1 // ~1 year
```

## Practical Example: Multi-Factor Signal

```sig
// multi_factor.sig - Combining momentum, value, and quality

data prices = load("prices.csv")
data earnings = load("earnings.csv")
data book_value = load("book.csv")
data cash_flow = load("cash_flow.csv")
data assets = load("assets.csv")

// Calculate returns and volatility
signal returns = prices / lag(prices, 1) - 1
signal vol = std(returns, 60)

// Momentum: 12-1 (skip most recent month)
signal mom_12_1 = lag(prices, 20) / lag(prices, 252) - 1
signal momentum = zscore(mom_12_1, 252)

// Value: composite
signal ep = earnings / prices  // Earnings yield
signal bp = book_value / prices  // Book yield
signal value = zscore((zscore(ep, 252) + zscore(bp, 252)) / 2, 252)

// Quality: ROE and accruals
signal roe = earnings / book_value
signal accruals = -(earnings - cash_flow) / assets
signal quality = zscore((zscore(roe, 252) + zscore(accruals, 252)) / 2, 252)

// Combine equally
signal composite = (momentum + value + quality) / 3

// Volatility-adjust
signal final_signal = composite / vol

output final_signal
```

## Key Takeaways

1. **Economic rationale first**: Understand why a signal should work
2. **Simple is robust**: Start simple, add complexity only if needed
3. **Normalize signals**: Use z-scores for comparability
4. **Combine signals**: Diversification across signals reduces risk
5. **Watch for decay**: Match trading frequency to signal decay

## Exercises

1. **Build momentum variations**: Test 20, 60, and 252-day momentum. Which performs best?

2. **Create a value signal**: Use P/E ratio to create a value signal. Compare to momentum.

3. **Combine signals**: Create a 50/50 momentum-value combination. How does it compare to each individually?

4. **Test decay**: Measure the IC of your signal at 1, 5, and 20-day forward returns.

## Next Chapter

[Chapter 4: The sigc Language](04-language.md) - Deep dive into sigc syntax and capabilities.
