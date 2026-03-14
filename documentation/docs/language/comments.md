# Comments

Comments document your code and are ignored by the compiler.

## Comment Styles

### C-Style Comments

```sig
// This is a single-line comment
signal example:
  x = ret(prices, 20)  // Inline comment
  emit x
```

### Shell-Style Comments

```sig
# This is also a single-line comment
signal example:
  x = ret(prices, 20)  # Inline comment
  emit x
```

Both styles are equivalent. Choose one and be consistent.

## Placement

### File Header

```sig
// Volatility-Adjusted Momentum Strategy
// Based on Moskowitz, Ooi, Pedersen (2012)
// Author: Quant Team
// Last Updated: 2024-01-15

data:
  prices: load parquet from "data/prices.parquet"
  ...
```

### Section Comments

```sig
// ==========================================
// Data Sources
// ==========================================

data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"

// ==========================================
// Parameters
// ==========================================

params:
  lookback = 60
  vol_window = 252

// ==========================================
// Signals
// ==========================================

signal momentum:
  ...
```

### Block Comments

```sig
// Compute volatility-adjusted momentum:
// 1. Calculate raw 60-day returns
// 2. Estimate 252-day volatility
// 3. Divide returns by volatility
// 4. Cross-sectionally normalize
// 5. Winsorize to remove outliers

signal vol_adj_momentum:
  returns = ret(prices, 60)
  vol = rolling_std(ret(prices, 1), 252)
  vol_adj = returns / vol
  normalized = zscore(vol_adj)
  emit winsor(normalized, p=0.01)
```

### Inline Comments

```sig
signal documented:
  returns = ret(prices, 60)           // 60-day returns (~3 months)
  vol = rolling_std(ret(prices, 1), 252)  // 252-day vol (~1 year)
  vol_adj = returns / vol             // Scale by volatility
  normalized = zscore(vol_adj)        // Cross-sectional normalization
  emit winsor(normalized, p=0.01)     // Remove outliers
```

## Documentation Conventions

### Signal Documentation

```sig
// Momentum Signal
//
// Computes 12-1 month momentum with volatility adjustment.
// Skips the most recent month to avoid short-term reversal.
//
// Parameters used:
//   lookback: Return computation window (default: 252)
//   skip_days: Days to skip (default: 21)
//   vol_window: Volatility estimation window (default: 60)
//
// References:
//   Jegadeesh & Titman (1993) "Returns to Buying Winners"
//   Moskowitz, Ooi, Pedersen (2012) "Time Series Momentum"
//
signal momentum:
  total = ret(prices, lookback)
  recent = ret(prices, skip_days)
  raw = total - recent
  vol = rolling_std(ret(prices, 1), vol_window)
  emit zscore(raw / vol)
```

### Parameter Documentation

```sig
params:
  // Signal parameters
  lookback = 252        // Return lookback in trading days (~1 year)
  skip_days = 21        // Days to skip for reversal effect (~1 month)

  // Volatility parameters
  vol_window = 60       // Volatility estimation window (~3 months)

  // Cleaning parameters
  winsor_pct = 0.01     // Winsorization percentile (1st/99th)

  // Portfolio parameters
  top_pct = 0.2         // Fraction of assets to long (20%)
  bottom_pct = 0.2      // Fraction of assets to short (20%)
  max_weight = 0.05     // Maximum position size (5%)
```

### Function/Macro Documentation

```sig
// Compute rolling Sharpe ratio
// Args:
//   returns: Daily or periodic returns
//   window: Rolling window size (default: 252 for annual)
// Returns:
//   Annualized Sharpe ratio
fn sharpe(returns, window=252):
  rolling_mean(returns, window) / rolling_std(returns, window) * sqrt(252)

// Sector-neutral momentum signal
// Args:
//   px: Price data
//   sectors: Sector classifications
//   lookback: Return window (default: 60)
//   vol_window: Volatility window (default: 252)
// Returns:
//   Cleaned, sector-neutral momentum score
macro sector_neutral_momentum(px: expr, sectors: expr, lookback: number = 60, vol_window: number = 252):
  let raw = ret(px, lookback)
  let vol = rolling_std(ret(px, 1), vol_window)
  let vol_adj = raw / vol
  let neutral = neutralize(vol_adj, by=sectors)
  emit winsor(zscore(neutral), p=0.01)
```

## Comment Best Practices

### 1. Explain "Why", Not "What"

```sig
// BAD: Explains what (obvious from code)
returns = ret(prices, 20)  // Compute 20-day returns

// GOOD: Explains why
returns = ret(prices, 20)  // 20 days ~ 1 month, standard momentum horizon
```

### 2. Keep Comments Current

Update comments when code changes. Stale comments are worse than no comments.

### 3. Use Consistent Style

Pick one comment style and use it throughout:

```sig
// Preferred: C-style throughout
signal consistent:
  // Step 1
  x = ...
  // Step 2
  y = ...

// Avoid: Mixed styles
signal inconsistent:
  // Step 1
  x = ...
  # Step 2
  y = ...
```

### 4. Document Non-Obvious Choices

```sig
signal documented_choices:
  // Skip 21 days (1 month) to avoid short-term reversal effect
  // per Jegadeesh & Titman (1993)
  total = ret(prices, 252)
  recent = ret(prices, 21)
  momentum = total - recent

  // Use 60-day vol to balance noise reduction with responsiveness
  vol = rolling_std(ret(prices, 1), 60)

  emit zscore(momentum / vol)
```

### 5. Group Related Code

```sig
signal well_organized:
  // ----- Momentum Computation -----
  total_return = ret(prices, 252)
  recent_return = ret(prices, 21)
  raw_momentum = total_return - recent_return

  // ----- Volatility Adjustment -----
  daily_returns = ret(prices, 1)
  volatility = rolling_std(daily_returns, 60)
  vol_adj_momentum = raw_momentum / volatility

  // ----- Normalization & Cleaning -----
  normalized = zscore(vol_adj_momentum)
  cleaned = winsor(normalized, p=0.01)

  emit cleaned
```

## Example: Well-Documented Strategy

```sig
// ============================================================================
// Volatility-Adjusted Momentum Strategy
// ============================================================================
//
// DESCRIPTION:
//   Implements 12-1 month momentum with volatility adjustment and
//   sector neutralization. Based on academic research showing momentum
//   effect is stronger when adjusted for volatility.
//
// METHODOLOGY:
//   1. Compute 12-month (252-day) returns
//   2. Subtract recent 1-month (21-day) to avoid reversal
//   3. Divide by trailing volatility to normalize
//   4. Remove sector bias via neutralization
//   5. Cross-sectionally standardize and winsorize
//
// REFERENCES:
//   - Jegadeesh & Titman (1993) "Returns to Buying Winners and Selling Losers"
//   - Moskowitz, Ooi, Pedersen (2012) "Time Series Momentum"
//   - Barroso & Santa-Clara (2015) "Momentum has its Moments"
//
// PARAMETERS:
//   lookback = 252    : Total return window (~1 year)
//   skip_days = 21    : Recent days to exclude (~1 month)
//   vol_window = 60   : Volatility estimation window (~3 months)
//   winsor_pct = 0.01 : Outlier clipping percentile
//   top_pct = 0.2     : Long position percentage
//   bottom_pct = 0.2  : Short position percentage
//   max_weight = 0.05 : Maximum single position
//
// EXPECTED PERFORMANCE:
//   Sharpe: 1.0-1.5 (depending on market conditions)
//   Turnover: 200-400% annually
//
// ============================================================================

data:
  prices: load parquet from "s3://data/prices.parquet" adjust=split_div
  sectors: load csv from "data/sectors.csv" dtype=category

params:
  // Return calculation
  lookback = 252          // 1 year lookback
  skip_days = 21          // Skip 1 month (reversal avoidance)

  // Volatility
  vol_window = 60         // 3 month vol estimation

  // Cleaning
  winsor_pct = 0.01       // Clip at 1st/99th percentile

  // Portfolio construction
  top_pct = 0.2           // Long top 20%
  bottom_pct = 0.2        // Short bottom 20%
  max_weight = 0.05       // Max 5% per position

// Helper function for volatility
fn vol(x, window):
  rolling_std(ret(x, 1), window)

signal momentum:
  // Compute momentum (12-1 month)
  total_return = ret(prices, lookback)
  recent_return = ret(prices, skip_days)
  raw_momentum = total_return - recent_return

  // Volatility adjustment (Barroso & Santa-Clara)
  volatility = vol(prices, vol_window)
  vol_adj = raw_momentum / volatility

  // Sector neutralization
  neutral = neutralize(vol_adj, by=sectors)

  // Final cleaning
  normalized = zscore(neutral)
  cleaned = winsor(normalized, winsor_pct)

  emit cleaned

portfolio main:
  weights = rank(momentum).long_short(
    top=top_pct,
    bottom=bottom_pct,
    cap=max_weight
  )
  costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest rebal=21 benchmark=SPY from 2015-01-01 to 2024-12-31
```

## Next Steps

- [Syntax Overview](syntax.md) - Complete syntax reference
- [Signal Section](signal-section.md) - Writing signals
- [Best Practices](../tutorials/index.md) - Strategy development
