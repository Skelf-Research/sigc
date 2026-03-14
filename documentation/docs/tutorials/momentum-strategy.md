# Momentum Strategy Tutorial

Build a classic cross-sectional momentum strategy from scratch.

## What You'll Learn

- Loading and preparing data
- Computing momentum signals
- Constructing long-short portfolios
- Backtesting with realistic costs
- Analyzing results

## Prerequisites

- sigc installed ([Installation](../getting-started/installation.md))
- Sample data ([Sample Data](../getting-started/sample-data.md))
- Basic understanding of [Signals](../concepts/signals.md)

## Step 1: Data Setup

Create a new file `momentum.sig`:

```sig
data:
  source = "prices.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    adj_close: Numeric as prices
    volume: Numeric
```

This loads daily price data with columns for date, ticker, adjusted close price, and volume.

## Step 2: Basic Momentum Signal

Momentum is the tendency for past winners to continue winning.

```sig
signal momentum:
  // 12-month return
  ret_12m = ret(prices, 252)

  // Skip most recent month (short-term reversal)
  ret_1m = ret(prices, 21)

  // 12-1 month momentum
  mom = ret_12m - ret_1m

  // Cross-sectional z-score
  z = zscore(mom)

  emit z
```

### Why 12-1 Month?

- **12 months**: Captures medium-term trends
- **Skip 1 month**: Avoids short-term reversal effect
- This is the classic Jegadeesh-Titman momentum

## Step 3: Portfolio Construction

```sig
portfolio basic:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

This creates:
- **Long**: Top 20% of momentum stocks
- **Short**: Bottom 20% of momentum stocks
- **Dollar neutral**: Net exposure = 0

## Step 4: Run the Backtest

```bash
sigc run momentum.sig
```

Output:

```
Backtest Results: basic
=======================
Period: 2015-01-01 to 2024-12-31

Performance Metrics:
  Total Return:     125.3%
  CAGR:             8.5%
  Volatility:       14.2%
  Sharpe Ratio:     0.60
  Max Drawdown:    -28.5%
```

## Step 5: Add Realistic Costs

Momentum strategies have high turnover. Let's add costs:

```sig
portfolio with_costs:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(10)  // 10 basis points
  backtest from 2015-01-01 to 2024-12-31
```

Run again:

```bash
sigc run momentum.sig
```

Notice how returns decrease with costs:

```
Performance with costs:
  Total Return:     98.2%    (was 125.3%)
  CAGR:             7.1%     (was 8.5%)
  Sharpe Ratio:     0.50     (was 0.60)
```

## Step 6: Improve the Signal

### Add Volatility Adjustment

Higher volatility momentum might be noise:

```sig
signal momentum_vol_adj:
  // Raw momentum
  mom = ret(prices, 252) - ret(prices, 21)

  // Volatility
  vol = rolling_std(ret(prices, 1), 60)

  // Volatility-adjusted momentum
  vol_adj_mom = mom / vol

  emit zscore(vol_adj_mom)
```

### Add Volume Filter

Avoid illiquid stocks:

```sig
signal momentum_filtered:
  // Raw momentum
  mom = ret(prices, 252) - ret(prices, 21)
  z = zscore(mom)

  // Volume filter
  avg_vol = rolling_mean(volume, 20)
  liquid = avg_vol > 1000000  // $1M+ average volume

  // Only signal liquid stocks
  filtered = where(liquid, z, 0)

  emit filtered
```

## Step 7: Sector Neutralization

Remove sector bets:

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    adj_close: Numeric as prices
    volume: Numeric
    sector: String as sectors

signal sector_neutral_momentum:
  // Raw momentum
  mom = ret(prices, 252) - ret(prices, 21)
  z = zscore(mom)

  // Sector neutralize
  neutral = neutralize(z, by=sectors)

  emit neutral
```

## Step 8: Full Strategy

Putting it all together:

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    adj_close: Numeric as prices
    volume: Numeric
    sector: String as sectors

signal momentum:
  // 12-1 month momentum
  mom = ret(prices, 252) - ret(prices, 21)

  // Volatility adjustment
  vol = rolling_std(ret(prices, 1), 60)
  vol_adj = mom / vol

  // Z-score
  z = zscore(vol_adj)

  // Winsorize outliers
  clean = winsor(z, p=0.01)

  // Sector neutralize
  neutral = neutralize(clean, by=sectors)

  emit neutral

portfolio momentum_strategy:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    max_sector = 0.25

  costs = tc.bps(10) + slippage.model("square-root", coef=0.1)

  backtest rebal=21 benchmark=SPY from 2015-01-01 to 2024-12-31
```

## Step 9: Analyze Results

### Performance Report

```bash
sigc run momentum.sig --report detailed
```

### Key Metrics to Check

| Metric | Target | Why |
|--------|--------|-----|
| Sharpe Ratio | > 0.5 | Risk-adjusted return |
| Max Drawdown | < 30% | Survivable losses |
| Turnover | < 300% | Manageable costs |
| Alpha vs SPY | > 0% | Adding value |

### Monthly Returns

```bash
sigc run momentum.sig --report monthly
```

### Factor Attribution

```bash
sigc run momentum.sig --report attribution
```

Check momentum factor exposure is significant.

## Step 10: Walk-Forward Validation

Ensure the strategy is robust:

```sig
portfolio validated:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.03)
  costs = tc.bps(10)

  backtest walk_forward(
    train_years = 5,
    test_years = 2,
    step_years = 2
  ) from 2010-01-01 to 2024-12-31
```

Compare in-sample vs out-of-sample performance.

## Complete Code

```sig
// momentum.sig - Complete momentum strategy

data:
  source = "prices_with_sectors.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    adj_close: Numeric as prices
    volume: Numeric
    sector: String as sectors

signal momentum:
  // 12-1 month momentum (classic)
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  raw_mom = ret_12m - ret_1m

  // Volatility adjustment
  vol = rolling_std(ret(prices, 1), 60)
  vol_adj = raw_mom / vol

  // Normalize
  z = zscore(vol_adj)

  // Handle outliers
  clean = winsor(z, p=0.01)

  // Sector neutral
  neutral = neutralize(clean, by=sectors)

  emit neutral

portfolio momentum_strategy:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    max_sector = 0.25
    dollar_neutral = true

  costs = tc.bps(10) + slippage.model("square-root", coef=0.1)

  backtest rebal=21 benchmark=SPY from 2015-01-01 to 2024-12-31
```

## Exercises

1. **Different Lookback**: Try 6-month momentum instead of 12-1
2. **Industry Neutral**: Neutralize by industry instead of sector
3. **Combination**: Combine with value signal
4. **Different Universe**: Apply to small caps only

## Common Issues

### High Drawdown

- Add position caps
- Reduce concentration
- Add stop-loss logic

### Low Sharpe After Costs

- Reduce rebalancing frequency
- Increase position cap (reduce turnover)
- Filter to more liquid stocks

### Sector Concentration

- Add sector constraints
- Use sector neutralization

## Next Steps

- [Mean Reversion](mean-reversion.md) - Contrarian strategy
- [Multi-Factor](multi-factor.md) - Combine with other signals
- [Walk-Forward](walk-forward-optimization.md) - Proper validation
