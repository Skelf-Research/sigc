# Tutorial: Custom Functions

Create reusable functions and macros to streamline your strategy code.

## Overview

Custom functions let you:

- Avoid code duplication
- Create domain-specific abstractions
- Build reusable signal libraries
- Improve code readability

## Basic Functions

### Defining a Function

```sig
// Define a custom volatility function
fn annualized_vol(returns, window):
  rolling_std(returns, window) * sqrt(252)

// Use it
signal vol_signal:
  daily_ret = ret(prices, 1)
  vol = annualized_vol(daily_ret, 60)
  emit -zscore(vol)
```

### Function with Multiple Returns

```sig
// Bollinger Bands function
fn bollinger_bands(prices, window, num_std):
  ma = rolling_mean(prices, window)
  std = rolling_std(prices, window)
  upper = ma + num_std * std
  lower = ma - num_std * std
  return (ma, upper, lower)

signal bb_position:
  (middle, upper, lower) = bollinger_bands(prices, 20, 2)
  position = (prices - lower) / (upper - lower)
  emit position
```

## Macros

### Simple Macro

```sig
// Macro for sector-neutral z-score
macro neutral_zscore(signal):
  neutralize(zscore(signal), by=sectors)

signal momentum:
  raw = ret(prices, 60)
  emit neutral_zscore(raw)

signal value:
  raw = book_to_market
  emit neutral_zscore(raw)
```

### Parameterized Macro

```sig
// Macro with default parameters
macro momentum_signal(lookback=60, skip=0):
  ret_full = ret(prices, lookback)
  ret_skip = where(skip > 0, ret(prices, skip), 0)
  zscore(ret_full - ret_skip)

signal mom_12_1:
  emit momentum_signal(252, 21)  // 12-1 momentum

signal mom_6:
  emit momentum_signal(126)  // 6-month momentum
```

## Signal Libraries

### Creating a Factor Library

```sig
// factors.sig - Reusable factor definitions

// ============ MOMENTUM FACTORS ============

fn price_momentum(prices, lookback):
  zscore(ret(prices, lookback))

fn momentum_12_1(prices):
  ret_12m = ret(prices, 252)
  ret_1m = ret(prices, 21)
  zscore(ret_12m - ret_1m)

fn momentum_6_1(prices):
  ret_6m = ret(prices, 126)
  ret_1m = ret(prices, 21)
  zscore(ret_6m - ret_1m)

// ============ VALUE FACTORS ============

fn book_to_market_zscore(book_value, market_cap):
  zscore(book_value / market_cap)

fn earnings_yield(earnings, prices):
  zscore(earnings / prices)

fn composite_value(book_value, market_cap, earnings, prices):
  btm = book_to_market_zscore(book_value, market_cap)
  ey = earnings_yield(earnings, prices)
  0.5 * btm + 0.5 * ey

// ============ QUALITY FACTORS ============

fn profitability(roe, roa):
  0.5 * zscore(roe) + 0.5 * zscore(roa)

fn earnings_stability(earnings, lookback):
  -zscore(rolling_std(earnings, lookback))

fn quality_composite(roe, roa, earnings, lookback):
  prof = profitability(roe, roa)
  stab = earnings_stability(earnings, lookback)
  0.7 * prof + 0.3 * stab

// ============ VOLATILITY FACTORS ============

fn annualized_vol(daily_returns, window):
  rolling_std(daily_returns, window) * sqrt(252)

fn low_vol_factor(prices, window):
  vol = annualized_vol(ret(prices, 1), window)
  -zscore(vol)

fn idiosyncratic_vol(stock_returns, market_returns, window):
  residual = stock_returns - market_returns
  rolling_std(residual, window) * sqrt(252)
```

### Using the Library

```sig
import "factors.sig"

data:
  source = "prices_fundamentals.parquet"
  format = parquet

signal multi_factor:
  mom = momentum_12_1(prices)
  val = composite_value(book_value, market_cap, earnings, prices)
  qual = quality_composite(roe, roa, earnings, 8)
  lvol = low_vol_factor(prices, 60)

  combined = 0.3 * mom + 0.3 * val + 0.2 * qual + 0.2 * lvol
  emit neutralize(combined, by=sectors)

portfolio main:
  weights = rank(multi_factor).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

## Technical Indicator Library

### Common Technical Indicators

```sig
// indicators.sig - Technical analysis indicators

// ============ MOVING AVERAGES ============

fn sma(prices, window):
  rolling_mean(prices, window)

fn ema(prices, span):
  alpha = 2 / (span + 1)
  ema_calc(prices, alpha)

fn wma(prices, window):
  // Linearly weighted moving average
  weights = range(1, window + 1)
  weighted_sum = rolling_sum(prices * weights, window)
  weight_sum = sum(weights)
  weighted_sum / weight_sum

// ============ OSCILLATORS ============

fn rsi(prices, window):
  change = diff(prices, 1)
  gains = where(change > 0, change, 0)
  losses = where(change < 0, -change, 0)
  avg_gain = ema(gains, window)
  avg_loss = ema(losses, window)
  rs = avg_gain / (avg_loss + 0.0001)
  100 - (100 / (1 + rs))

fn stochastic(high, low, close, k_window, d_window):
  lowest = rolling_min(low, k_window)
  highest = rolling_max(high, k_window)
  k = 100 * (close - lowest) / (highest - lowest + 0.0001)
  d = sma(k, d_window)
  return (k, d)

fn macd(prices, fast, slow, signal_window):
  fast_ema = ema(prices, fast)
  slow_ema = ema(prices, slow)
  macd_line = fast_ema - slow_ema
  signal_line = ema(macd_line, signal_window)
  histogram = macd_line - signal_line
  return (macd_line, signal_line, histogram)

// ============ VOLATILITY ============

fn atr(high, low, close, window):
  tr1 = high - low
  tr2 = abs(high - lag(close, 1))
  tr3 = abs(low - lag(close, 1))
  true_range = max(tr1, max(tr2, tr3))
  ema(true_range, window)

fn bollinger_bands(prices, window, num_std):
  middle = sma(prices, window)
  std = rolling_std(prices, window)
  upper = middle + num_std * std
  lower = middle - num_std * std
  return (lower, middle, upper)

fn keltner_channels(high, low, close, ema_window, atr_window, multiplier):
  middle = ema(close, ema_window)
  atr_val = atr(high, low, close, atr_window)
  upper = middle + multiplier * atr_val
  lower = middle - multiplier * atr_val
  return (lower, middle, upper)

// ============ TREND ============

fn adx(high, low, close, window):
  // Simplified ADX
  plus_dm = where(diff(high, 1) > diff(low, 1) * -1 and diff(high, 1) > 0,
                  diff(high, 1), 0)
  minus_dm = where(diff(low, 1) * -1 > diff(high, 1) and diff(low, 1) < 0,
                   -diff(low, 1), 0)

  atr_val = atr(high, low, close, window)
  plus_di = 100 * ema(plus_dm, window) / atr_val
  minus_di = 100 * ema(minus_dm, window) / atr_val

  dx = 100 * abs(plus_di - minus_di) / (plus_di + minus_di + 0.0001)
  ema(dx, window)

fn aroon(high, low, window):
  bars_since_high = window - ts_argmax(high, window)
  bars_since_low = window - ts_argmin(low, window)
  aroon_up = 100 * (window - bars_since_high) / window
  aroon_down = 100 * (window - bars_since_low) / window
  return (aroon_up, aroon_down)
```

## Utility Functions

### Data Processing Utilities

```sig
// utils.sig - Utility functions

// ============ DATA CLEANING ============

fn winsorize(x, lower_pct, upper_pct):
  lower = quantile(x, lower_pct)
  upper = quantile(x, upper_pct)
  clip(x, lower, upper)

fn robust_zscore(x):
  median_val = median(x)
  mad = median(abs(x - median_val))
  (x - median_val) / (1.4826 * mad + 0.0001)

fn fill_missing(x, method):
  // Forward fill missing values
  where(is_nan(x), lag(x, 1), x)

// ============ NORMALIZATION ============

fn rank_normalize(x):
  // Convert to uniform distribution
  rank(x) / count(x)

fn percentile_rank(x, window):
  ts_rank(x, window) / window

// ============ COMBINATIONS ============

fn weighted_avg(signals, weights):
  // Combine signals with weights
  total_weight = sum(weights)
  sum(signals * weights) / total_weight

fn ensemble(signals):
  // Simple average of signals
  sum(signals) / count(signals)

// ============ RISK METRICS ============

fn sharpe_ratio(returns, window, rf_rate):
  mean_ret = rolling_mean(returns, window) * 252
  vol = rolling_std(returns, window) * sqrt(252)
  (mean_ret - rf_rate) / vol

fn max_drawdown(prices, window):
  rolling_max_price = rolling_max(prices, window)
  drawdown = (prices - rolling_max_price) / rolling_max_price
  rolling_min(drawdown, window)

fn calmar_ratio(returns, prices, window):
  annual_ret = rolling_mean(returns, window) * 252
  mdd = abs(max_drawdown(prices, window))
  annual_ret / (mdd + 0.0001)
```

## Complete Example: Custom Strategy Library

### Full Implementation

```sig
// my_strategy_lib.sig - Complete strategy library

import "factors.sig"
import "indicators.sig"
import "utils.sig"

// ============ COMPOSITE SIGNALS ============

fn trend_following_signal(prices, fast_window, slow_window):
  // Dual moving average crossover
  fast_ma = sma(prices, fast_window)
  slow_ma = sma(prices, slow_window)
  trend = (fast_ma - slow_ma) / slow_ma

  // Trend strength from ADX
  // (simplified - using volatility as proxy)
  vol = annualized_vol(ret(prices, 1), slow_window)
  strength = 1 / vol

  zscore(trend * strength)

fn mean_reversion_signal(prices, window, threshold):
  ma = sma(prices, window)
  std = rolling_std(prices, window)
  zscore_val = (prices - ma) / std

  // Only signal at extremes
  signal = where(abs(zscore_val) > threshold, -zscore_val, 0)
  signal

fn quality_momentum(prices, roe, earnings, mom_window):
  // Quality-filtered momentum
  mom = momentum_12_1(prices)
  qual = quality_composite(roe, roe, earnings, 8)  // Using roe twice as roa proxy

  // Only take momentum in quality stocks
  qual_threshold = quantile(qual, 0.5)
  where(qual > qual_threshold, mom, 0)

macro regime_adaptive(bull_signal, bear_signal, regime_indicator):
  where(regime_indicator > 0, bull_signal, bear_signal)
```

### Using the Library

```sig
import "my_strategy_lib.sig"

data:
  source = "full_data.parquet"
  format = parquet

// Detect regime
signal regime:
  ma_50 = sma(prices, 50)
  ma_200 = sma(prices, 200)
  emit where(ma_50 > ma_200, 1, -1)

// Build signals
signal trend_signal:
  emit trend_following_signal(prices, 20, 60)

signal reversion_signal:
  emit mean_reversion_signal(prices, 20, 2)

signal qual_mom_signal:
  emit quality_momentum(prices, roe, earnings, 252)

// Combine adaptively
signal combined:
  bull = 0.5 * trend_signal + 0.3 * qual_mom_signal + 0.2 * reversion_signal
  bear = 0.2 * trend_signal + 0.3 * qual_mom_signal + 0.5 * reversion_signal

  emit regime_adaptive(bull, bear, regime)

portfolio main:
  weights = rank(combined).long_short(top=0.15, bottom=0.15, cap=0.03)

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

## Best Practices

### 1. Name Functions Clearly

```sig
// Good
fn risk_adjusted_momentum(prices, lookback):
  ...

// Bad
fn ram(p, l):
  ...
```

### 2. Document Parameters

```sig
// @param prices: Asset prices
// @param window: Lookback period in days
// @param threshold: Z-score threshold for signal
fn mean_reversion(prices, window, threshold):
  ...
```

### 3. Use Default Parameters

```sig
fn momentum(prices, lookback=60, skip=0):
  ...
```

### 4. Keep Functions Focused

```sig
// Good: Single responsibility
fn calculate_volatility(returns, window):
  rolling_std(returns, window) * sqrt(252)

// Bad: Too many responsibilities
fn do_everything(prices, params):
  // 50 lines of mixed logic
```

### 5. Test Functions Independently

```sig
// Test momentum function
signal test_momentum:
  emit momentum_12_1(prices)

// Verify output before using in strategy
portfolio test:
  weights = rank(test_momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2023-01-01 to 2023-12-31
```

## Next Steps

- [Walk-Forward Optimization](walk-forward-optimization.md) - Test robustly
- [Production Deployment](production-deployment.md) - Go live
- [Python Workflow](python-workflow.md) - Integration with Python
