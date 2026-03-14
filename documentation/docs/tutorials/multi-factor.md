# Tutorial: Multi-Factor Strategy

Build a robust strategy combining multiple alpha factors.

## Overview

Multi-factor strategies:

- Combine diverse signals for more robust alpha
- Reduce reliance on single factors
- Smooth returns across market regimes

## The Strategy

We'll build a strategy combining:

1. **Momentum** - Trend following
2. **Value** - Cheap stocks
3. **Quality** - Strong fundamentals
4. **Low Volatility** - Defensive positioning

## Step 1: Individual Factors

### Momentum Factor

```sig
signal momentum:
  // 12-month return, excluding last month
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  momentum = ret_12m - ret_1m  // 12-1 momentum

  emit zscore(momentum)
```

### Value Factor

```sig
signal value:
  // Multiple value metrics
  book_to_market = book_value / market_cap
  earnings_yield = earnings / prices

  // Combine value signals
  value = 0.5 * zscore(book_to_market) + 0.5 * zscore(earnings_yield)

  emit value
```

### Quality Factor

```sig
signal quality:
  // Profitability and stability
  roe = net_income / equity
  roa = net_income / assets
  margin_stability = -rolling_std(gross_margin, 12)

  // Combine quality signals
  quality = 0.4 * zscore(roe) + 0.4 * zscore(roa) + 0.2 * zscore(margin_stability)

  emit quality
```

### Low Volatility Factor

```sig
signal low_volatility:
  // Inverse volatility
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)

  emit -zscore(vol)
```

## Step 2: Simple Equal Weighting

### Basic Combination

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal quality:
  emit zscore(roe)

signal low_vol:
  emit -zscore(rolling_std(ret(prices, 1), 60))

// Equal-weighted combination
signal multi_factor:
  combined = 0.25 * momentum + 0.25 * value + 0.25 * quality + 0.25 * low_vol
  emit combined

portfolio equal_weighted:
  weights = rank(multi_factor).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Step 3: Factor Timing

### Adjust Weights by Regime

```sig
signal market_regime:
  // Detect market regime
  market_ret = rolling_mean(ret(prices, 1), 20)
  market_vol = rolling_std(ret(prices, 1), 20) * sqrt(252)

  // Bull: positive return, low vol
  // Bear: negative return, high vol
  bull_score = zscore(market_ret) - zscore(market_vol)

  emit bull_score

signal timed_multi_factor:
  // Base factors
  mom = momentum
  val = value
  qual = quality
  lvol = low_vol

  // Regime-dependent weights
  regime = market_regime
  bull = regime > 0

  // Bull market: favor momentum
  // Bear market: favor quality and low vol
  w_mom = where(bull, 0.35, 0.15)
  w_val = 0.25
  w_qual = where(bull, 0.20, 0.35)
  w_lvol = where(bull, 0.20, 0.25)

  combined = w_mom * mom + w_val * val + w_qual * qual + w_lvol * lvol

  emit combined

portfolio factor_timed:
  weights = rank(timed_multi_factor).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Step 4: Optimized Factor Weights

### Find Optimal Weights via Backtest

```sig
params:
  w_mom: range(0.1, 0.5, 0.1)
  w_val: range(0.1, 0.5, 0.1)
  w_qual: range(0.1, 0.5, 0.1)
  // w_lvol = 1 - others (implied)

signal multi_factor_optimized:
  w_lvol = 1.0 - w_mom - w_val - w_qual

  combined = w_mom * momentum + w_val * value + w_qual * quality + w_lvol * low_vol

  emit where(w_lvol >= 0, combined, 0)  // Valid combinations only

portfolio optimized:
  weights = rank(multi_factor_optimized).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

```bash
sigc run strategy.sig --optimize --metric sharpe
```

## Step 5: Z-Score Combination Methods

### Different Combination Approaches

```sig
// Method 1: Simple average of z-scores
signal zscore_avg:
  emit (momentum + value + quality + low_vol) / 4

// Method 2: Rank average
signal rank_avg:
  r_mom = rank(momentum)
  r_val = rank(value)
  r_qual = rank(quality)
  r_lvol = rank(low_vol)
  emit (r_mom + r_val + r_qual + r_lvol) / 4

// Method 3: IC-weighted
// Weight by factor's predictive power
signal ic_weighted:
  // Hypothetical IC values (would be calculated historically)
  ic_mom = 0.04
  ic_val = 0.03
  ic_qual = 0.02
  ic_lvol = 0.02
  total_ic = ic_mom + ic_val + ic_qual + ic_lvol

  combined = (ic_mom * momentum + ic_val * value + ic_qual * quality + ic_lvol * low_vol) / total_ic
  emit combined
```

## Step 6: Sector-Neutral Factors

### Neutralize Within Sectors

```sig
signal sector_neutral_factors:
  // Neutralize each factor by sector
  mom_neutral = neutralize(momentum, by=sectors)
  val_neutral = neutralize(value, by=sectors)
  qual_neutral = neutralize(quality, by=sectors)
  lvol_neutral = neutralize(low_vol, by=sectors)

  // Combine neutralized factors
  combined = 0.3 * mom_neutral + 0.3 * val_neutral + 0.2 * qual_neutral + 0.2 * lvol_neutral

  emit combined

portfolio sector_neutral:
  weights = rank(sector_neutral_factors).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    max_sector = 0.20  // Limit sector exposure
    net_exposure = 0.0

  backtest from 2015-01-01 to 2024-12-31
```

## Step 7: Complete Multi-Factor Strategy

### Production-Ready Implementation

```sig
data:
  source = "full_dataset.parquet"
  format = parquet

// ============ INDIVIDUAL FACTORS ============

// Momentum: 12-1 month returns
signal momentum:
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  raw = ret_12m - ret_1m
  emit neutralize(zscore(raw), by=sectors)

// Value: Book-to-market and earnings yield
signal value:
  btm = zscore(book_to_market)
  ey = zscore(earnings / prices)
  raw = 0.5 * btm + 0.5 * ey
  emit neutralize(raw, by=sectors)

// Quality: ROE, ROA, and earnings stability
signal quality:
  prof = 0.4 * zscore(roe) + 0.4 * zscore(roa)
  stability = zscore(-rolling_std(earnings, 8))
  raw = 0.7 * prof + 0.3 * stability
  emit neutralize(raw, by=sectors)

// Low Volatility: Inverse realized vol
signal low_volatility:
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  raw = -zscore(vol)
  emit neutralize(raw, by=sectors)

// Size: Small cap premium
signal size:
  raw = -zscore(market_cap)
  emit neutralize(raw, by=sectors)

// ============ REGIME DETECTION ============

signal volatility_regime:
  market_vol = rolling_std(ret(prices, 1), 20) * sqrt(252)
  long_vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  high_vol = market_vol > long_vol * 1.3
  emit high_vol

signal trend_regime:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  uptrend = ma_50 > ma_200
  emit uptrend

// ============ FACTOR COMBINATION ============

signal multi_factor:
  // Get regime indicators
  high_vol = volatility_regime
  uptrend = trend_regime

  // Dynamic factor weights
  // Normal: Balanced
  // High Vol: Favor quality and low vol
  // Downtrend: Reduce momentum

  w_mom = where(uptrend, 0.30, 0.15)
  w_mom = where(high_vol, w_mom * 0.7, w_mom)

  w_val = 0.25

  w_qual = where(high_vol, 0.30, 0.20)

  w_lvol = where(high_vol, 0.25, 0.15)

  w_size = 0.10

  // Normalize weights
  total_w = w_mom + w_val + w_qual + w_lvol + w_size

  combined = (w_mom * momentum + w_val * value + w_qual * quality +
              w_lvol * low_volatility + w_size * size) / total_w

  emit combined

// ============ PORTFOLIO ============

portfolio multi_factor:
  weights = rank(multi_factor).long_short(
    top = 0.15,
    bottom = 0.15,
    cap = 0.025
  )

  constraints:
    // Exposure limits
    gross_exposure = 2.0
    net_exposure: [-0.1, 0.1]  // Close to dollar neutral

    // Position limits
    max_sector = 0.20
    max_position = 0.025

    // Turnover control
    max_turnover = 0.30

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Step 8: Run and Analyze

### Execute Backtest

```bash
sigc run multi_factor.sig --output results/
```

### Factor Attribution

```bash
sigc run multi_factor.sig --attribution
```

Output:
```
Factor Attribution Analysis
===========================

Factor Contributions to Return:
  Momentum:       +2.8%
  Value:          +1.9%
  Quality:        +1.5%
  Low Volatility: +0.9%
  Size:           +0.4%
  Specific:       +1.2%
  -------------------------
  Total:          +8.7%

Factor Correlations:
           Mom    Val   Qual   LVol   Size
Momentum   1.00  -0.15  0.08  -0.22   0.05
Value     -0.15   1.00  0.12   0.18  -0.10
Quality    0.08   0.12  1.00   0.25  -0.15
Low Vol   -0.22   0.18  0.25   1.00   0.08
Size       0.05  -0.10 -0.15   0.08   1.00
```

## Step 9: Walk-Forward Validation

### Robust Testing

```sig
portfolio validated:
  weights = rank(multi_factor).long_short(top=0.15, bottom=0.15)

  backtest walk_forward(
    train_years = 5,
    test_years = 1,
    step_years = 1
  ) from 2010-01-01 to 2024-12-31
```

## Key Insights

### Factor Selection Criteria

1. **Economic rationale** - Why should it work?
2. **Empirical evidence** - Academic and industry research
3. **Low correlation** - Diversification benefit
4. **Implementability** - Can you actually trade it?

### Common Pitfalls

1. **Over-fitting** - Too many factors
2. **Factor crowding** - Everyone using same factors
3. **Data mining** - Finding spurious patterns
4. **Ignoring costs** - High turnover destroys alpha

### Best Practices

1. **Use robust estimation** - Shrinkage, winsorization
2. **Sector neutralize** - Avoid sector bets
3. **Control turnover** - Balance signal decay vs costs
4. **Regular revalidation** - Monitor factor decay

## Next Steps

- [Volatility Strategy](volatility-strategy.md) - Trade volatility
- [Walk-Forward Optimization](walk-forward-optimization.md) - Robust testing
- [Production Deployment](production-deployment.md) - Go live
