# Sector Neutral Strategy

Long-short within sectors to eliminate sector exposure.

## Strategy Overview

Rank stocks within each sector. Go long the best in each sector, short the worst in each sector.

## Complete Strategy

```sig
data:
  source = "prices_with_sectors.parquet"
  format = parquet

// Alpha signal
signal alpha:
  momentum = zscore(ret(prices, 60))
  value = zscore(book_to_market)
  emit 0.5 * momentum + 0.5 * value

// Sector-neutral version
signal sector_neutral_alpha:
  // Rank within each sector
  within_sector_rank = group_rank(alpha, by=sectors)

  // Normalize to signal
  sector_size = group_count(prices, by=sectors)
  normalized = (within_sector_rank - sector_size / 2) / sector_size

  emit normalized

portfolio sector_neutral:
  weights = rank(sector_neutral_alpha).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    // Net exposure within each sector = 0
    sector_neutral = true

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Simple Sector Neutralization

```sig
signal simple_neutral:
  raw_signal = momentum

  // Demean within sector
  sector_mean = group_mean(raw_signal, by=sectors)
  neutral = raw_signal - sector_mean

  emit neutral
```

## Expected Results

```
Backtest Results: sector_neutral
================================
Period: 2015-01-01 to 2024-12-31

Returns:
  Annual Return: 4.8%
  Sharpe Ratio: 0.82

Sector Exposure:
  Max Sector Net: 0.5%
  Avg Sector Net: 0.1%
```

## See Also

- [Pairs Trading](pairs-trading.md)
- [Beta Neutral](beta-neutral.md)
