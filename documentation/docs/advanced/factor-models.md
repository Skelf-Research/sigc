# Factor Models

Build sophisticated multi-factor portfolios.

## Overview

Factor models decompose returns into systematic components:

$$R_i = \alpha_i + \beta_{1,i} F_1 + \beta_{2,i} F_2 + ... + \epsilon_i$$

## Classic Factors

### Fama-French Three Factor

```sig
signal market:
  emit ret(prices, 1)

signal smb:  // Small Minus Big
  small = market_cap < median(market_cap)
  ret_small = where(small, ret(prices, 1), 0)
  ret_big = where(not(small), ret(prices, 1), 0)
  emit mean(ret_small) - mean(ret_big)

signal hml:  // High Minus Low
  value = book_to_market
  high = value > quantile(value, 0.7)
  low = value < quantile(value, 0.3)
  ret_high = where(high, ret(prices, 1), 0)
  ret_low = where(low, ret(prices, 1), 0)
  emit mean(ret_high) - mean(ret_low)
```

### Five Factor Model

```sig
signal rmw:  // Robust Minus Weak (Profitability)
  robust = roe > quantile(roe, 0.7)
  weak = roe < quantile(roe, 0.3)
  ret_robust = where(robust, ret(prices, 1), 0)
  ret_weak = where(weak, ret(prices, 1), 0)
  emit mean(ret_robust) - mean(ret_weak)

signal cma:  // Conservative Minus Aggressive (Investment)
  conservative = asset_growth < quantile(asset_growth, 0.3)
  aggressive = asset_growth > quantile(asset_growth, 0.7)
  ret_cons = where(conservative, ret(prices, 1), 0)
  ret_agg = where(aggressive, ret(prices, 1), 0)
  emit mean(ret_cons) - mean(ret_agg)
```

## Factor Construction

### Single Factor

```sig
signal momentum_factor:
  // Raw momentum
  raw = ret(prices, 252) - ret(prices, 21)  // 12-1 month

  // Winsorize outliers
  z = zscore(raw)
  clean = winsor(z, p=0.01)

  // Sector neutralize
  neutral = neutralize(clean, by=sectors)

  emit neutral
```

### Composite Factor

```sig
signal value:
  book_value = zscore(book_to_market)
  earnings_yield = zscore(1 / pe_ratio)
  fcf_yield = zscore(fcf / market_cap)

  // Equal weight combination
  emit (book_value + earnings_yield + fcf_yield) / 3

signal quality:
  profitability = zscore(roe)
  stability = -zscore(rolling_std(roe, 4))  // Quarterly
  leverage = -zscore(debt_to_equity)

  emit (profitability + stability + leverage) / 3

signal composite:
  // Weight factors
  emit 0.4 * value + 0.4 * momentum + 0.2 * quality
```

## Factor Blending

### Static Weighting

```sig
signal static_blend:
  emit 0.25 * momentum + 0.25 * value + 0.25 * quality + 0.25 * low_vol
```

### Dynamic Weighting

```sig
signal dynamic_blend:
  // Adjust weights based on regime
  vol_regime = rolling_std(ret(market, 1), 60)
  high_vol = vol_regime > quantile(vol_regime, 0.8)

  // More defensive in high vol
  mom_weight = where(high_vol, 0.2, 0.4)
  val_weight = where(high_vol, 0.3, 0.2)
  qual_weight = where(high_vol, 0.3, 0.2)
  lowvol_weight = where(high_vol, 0.2, 0.2)

  emit mom_weight * momentum +
       val_weight * value +
       qual_weight * quality +
       lowvol_weight * low_vol
```

### Rank-Based Blending

```sig
signal rank_blend:
  // Rank each factor
  r_mom = rank(momentum)
  r_val = rank(value)
  r_qual = rank(quality)

  // Average ranks
  avg_rank = (r_mom + r_val + r_qual) / 3

  emit avg_rank
```

## Factor Neutralization

### Sector Neutral

```sig
signal sector_neutral_momentum:
  raw = zscore(ret(prices, 60))
  emit neutralize(raw, by=sectors)
```

### Industry Neutral

```sig
signal industry_neutral_value:
  raw = zscore(book_to_market)
  emit neutralize(raw, by=industry)
```

### Multi-Level Neutralization

```sig
signal double_neutral:
  raw = zscore(ret(prices, 60))

  // First sector neutral
  sector_neutral = neutralize(raw, by=sectors)

  // Then size neutral
  emit neutralize(sector_neutral, by=size_bucket)
```

### Market Cap Weighted Neutralization

```sig
signal cap_weighted_neutral:
  raw = zscore(ret(prices, 60))
  sector_mean = group_mean(raw, by=sectors, weights=market_cap)
  emit raw - sector_mean
```

## Factor Timing

### Momentum Factor Timing

```sig
signal timed_momentum:
  mom = zscore(ret(prices, 60))

  // Factor momentum (trailing 12-month factor performance)
  factor_perf = rolling_mean(mom, 252)
  factor_strong = factor_perf > 0

  // Only use momentum when factor is working
  emit where(factor_strong, mom, 0)
```

### Value Spread Timing

```sig
signal timed_value:
  value = zscore(book_to_market)

  // Value spread
  high_val = quantile(value, 0.9)
  low_val = quantile(value, 0.1)
  spread = high_val - low_val

  // Wide spread = value likely to work
  wide_spread = spread > quantile(spread, 0.7)

  emit where(wide_spread, value, 0)
```

## Factor Exposure Control

### Target Exposures

```sig
portfolio factor_targeted:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    // Target factor exposures
    factor_exposure:
      market: [0.8, 1.2]    // Beta between 0.8 and 1.2
      smb: [-0.2, 0.2]      // Small size neutral
      hml: [0.0, 0.5]       // Slight value tilt
      momentum: [0.3, 0.7]  // Moderate momentum
```

### Factor Hedging

```sig
signal hedged_alpha:
  // Raw alpha signal
  alpha = my_alpha_signal

  // Estimate factor exposures
  beta_market = rolling_corr(alpha, market, 60)
  beta_size = rolling_corr(alpha, smb, 60)

  // Hedge out exposures
  hedged = alpha - beta_market * market - beta_size * smb

  emit hedged
```

## Factor Attribution

### In Backtest

```sig
portfolio main:
  weights = rank(composite).long_short(top=0.2, bottom=0.2)

  backtest factors=[MKT, SMB, HML, MOM] from 2015-01-01 to 2024-12-31
```

### Attribution Output

```
Factor Attribution:
                                        Contribution
Factor      | Beta    | Factor Return | to Strategy
------------+---------+---------------+------------
Market      |  0.05   |    10.5%      |    0.5%
SMB         |  0.35   |     2.1%      |    0.7%
HML         |  0.42   |     3.5%      |    1.5%
MOM         |  0.28   |     5.2%      |    1.5%
------------+---------+---------------+------------
Total Factor|         |               |    4.2%
Alpha       |         |               |    8.3%
Total       |         |               |   12.5%
```

## Complete Example

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

// === Factor Definitions ===

signal momentum:
  raw = ret(prices, 252) - ret(prices, 21)
  z = zscore(raw)
  clean = winsor(z, p=0.01)
  emit neutralize(clean, by=sectors)

signal value:
  bm = zscore(book_to_market)
  ey = zscore(earnings / market_cap)
  raw = 0.5 * bm + 0.5 * ey
  clean = winsor(raw, p=0.01)
  emit neutralize(clean, by=sectors)

signal quality:
  prof = zscore(roe)
  stab = -zscore(rolling_std(earnings, 4))
  lev = -zscore(debt_to_equity)
  raw = (prof + stab + lev) / 3
  clean = winsor(raw, p=0.01)
  emit neutralize(clean, by=sectors)

signal low_volatility:
  vol = rolling_std(ret(prices, 1), 252)
  raw = -zscore(vol)  // Low vol = high signal
  emit neutralize(raw, by=sectors)

// === Composite Signal ===

signal composite:
  emit 0.30 * momentum +
       0.30 * value +
       0.25 * quality +
       0.15 * low_volatility

// === Portfolio Construction ===

portfolio multi_factor:
  weights = rank(composite).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    max_sector = 0.25
    dollar_neutral = true

  costs = tc.bps(10)

  backtest rebal=21 factors=[MKT, SMB, HML, MOM]
           from 2010-01-01 to 2024-12-31
```

## Best Practices

### 1. Neutralize Factors

Remove unintended exposures:

```sig
emit neutralize(raw, by=sectors)
```

### 2. Winsorize Outliers

```sig
clean = winsor(z, p=0.01)
```

### 3. Use Ranks for Robustness

```sig
weights = rank(composite).long_short(...)
```

### 4. Monitor Factor Exposures

Check exposures are as intended.

### 5. Diversify Across Factors

Don't rely on single factor.

## Next Steps

- [Risk Models](risk-models.md) - Risk estimation
- [Portfolio Optimization](portfolio-optimization.md) - Optimal weighting
- [Attribution](../backtesting/attribution.md) - Factor attribution
