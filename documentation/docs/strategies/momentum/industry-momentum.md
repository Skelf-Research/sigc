# Industry Momentum Strategy

Sector rotation based on industry performance.

## Strategy Overview

Invest in industries with strong recent performance, avoid weak industries. This captures macro trends and sector rotation.

## The Signal

```sig
signal industry_momentum:
  // Industry-level momentum
  // Group stocks by sector
  industry_ret = group_mean(ret(prices, 60), by=sectors)

  // Assign industry score to each stock
  emit zscore(industry_ret)
```

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Industry-level momentum
signal industry_momentum:
  // Calculate sector returns
  stock_ret = ret(prices, 60)
  sector_ret = group_mean(stock_ret, by=sectors)

  // Z-score across sectors
  emit zscore(sector_ret)

// Stock selection within industries
signal within_industry:
  stock_ret = ret(prices, 60)
  sector_ret = group_mean(stock_ret, by=sectors)

  // Relative to sector
  relative = stock_ret - sector_ret

  emit zscore(relative)

// Combined: industry bet + stock selection
signal combined:
  emit 0.7 * industry_momentum + 0.3 * within_industry

portfolio industry_momentum:
  weights = rank(combined).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    // Allow sector concentration (that's the point)
    max_sector = 0.40

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Sector-Level Implementation

### Pure Sector Rotation

```sig
signal sector_rotation:
  // Rank sectors by momentum
  sector_ret_60 = group_mean(ret(prices, 60), by=sectors)
  sector_rank = rank(sector_ret_60)

  // Long top sectors, short bottom sectors
  top_sectors = sector_rank >= quantile(sector_rank, 0.7)
  bottom_sectors = sector_rank <= quantile(sector_rank, 0.3)

  signal = where(top_sectors, 1,
           where(bottom_sectors, -1, 0))

  emit signal
```

### With Economic Regime

```sig
signal regime_aware_sector:
  // Detect economic regime
  growth_proxy = rolling_mean(ret(market, 60), 60)
  high_growth = growth_proxy > 0

  // Sector momentum
  sector_ret = group_mean(ret(prices, 60), by=sectors)

  // Cyclicals in growth, defensives in contraction
  cyclical_sectors = sectors in ["Technology", "Consumer Discretionary", "Financials"]
  defensive_sectors = sectors in ["Utilities", "Healthcare", "Consumer Staples"]

  base_signal = zscore(sector_ret)

  // Tilt by regime
  regime_tilt = where(high_growth and cyclical_sectors, 0.2,
                where(not(high_growth) and defensive_sectors, 0.2, 0))

  emit base_signal + regime_tilt
```

## Relative Strength

### Industry vs Market

```sig
signal industry_relative_strength:
  // Sector return vs market
  sector_ret = group_mean(ret(prices, 60), by=sectors)
  market_ret = ret(market, 60)

  relative = sector_ret - market_ret

  emit zscore(relative)
```

### Industry Rotation Speed

```sig
signal fast_rotation:
  // Short-term industry momentum (faster rotation)
  sector_ret_20 = group_mean(ret(prices, 20), by=sectors)
  emit zscore(sector_ret_20)

signal slow_rotation:
  // Long-term industry momentum (slower rotation)
  sector_ret_120 = group_mean(ret(prices, 120), by=sectors)
  emit zscore(sector_ret_120)

signal adaptive_rotation:
  // Use fast in trending, slow in choppy markets
  trend_strength = abs(rolling_mean(ret(market, 1), 60))
  strong_trend = trend_strength > quantile(trend_strength, 0.7)

  emit where(strong_trend, fast_rotation, slow_rotation)
```

## Expected Results

```
Backtest Results: industry_momentum
===================================
Period: 2010-01-01 to 2024-12-31

Returns:
  Total Return: 142%
  Annual Return: 6.4%
  Annual Volatility: 11.5%
  Sharpe Ratio: 0.56

Sector Exposure:
  Avg Long Sectors: 3.2
  Avg Short Sectors: 3.1
  Max Single Sector: 35%

Turnover:
  Annual Turnover: 180%
```

## Risk Considerations

### Sector Concentration

By design, this strategy has sector concentration:

```sig
// Add within-sector diversification
constraints:
  min_positions_per_sector = 5
```

### Regime Dependence

Sector rotation works best in trending markets:

```sig
// Add trend filter
signal trend_confirmed:
  industry_mom = industry_momentum
  market_trend = ma_50 > ma_200

  // Reduce exposure in choppy markets
  emit where(market_trend, industry_mom, industry_mom * 0.6)
```

## Variations

### Fundamentals-Weighted

```sig
signal fundamental_sector:
  // Weight by sector valuations
  sector_pe = group_mean(pe_ratio, by=sectors)
  sector_growth = group_mean(earnings_growth, by=sectors)

  // Cheap + growing sectors
  value = -zscore(sector_pe)
  growth = zscore(sector_growth)

  emit 0.5 * industry_momentum + 0.3 * value + 0.2 * growth
```

## See Also

- [Price Momentum](price-momentum.md)
- [Trend Following](trend-following.md)
