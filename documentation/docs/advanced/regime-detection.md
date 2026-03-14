# Regime Detection

Identify market regimes and adapt strategies accordingly.

## Overview

Markets exhibit different regimes:

- **Bull/Bear**: Trending up or down
- **High/Low Volatility**: Calm vs turbulent
- **Risk-On/Risk-Off**: Appetite for risk
- **Correlation Regimes**: Assets moving together or apart

## Volatility Regimes

### Simple Volatility Detection

```sig
signal vol_regime:
  daily_ret = ret(prices, 1)
  vol_20 = rolling_std(daily_ret, 20) * sqrt(252)
  vol_60 = rolling_std(daily_ret, 60) * sqrt(252)

  // Compare short-term to long-term vol
  vol_ratio = vol_20 / vol_60

  // High vol regime
  high_vol = vol_ratio > 1.5

  emit high_vol
```

### VIX-Based Detection

```sig
signal vix_regime:
  // VIX levels
  low_vol = vix < 15
  normal_vol = vix >= 15 and vix < 25
  high_vol = vix >= 25
  extreme_vol = vix >= 35

  // Encode as numeric
  regime = where(extreme_vol, 4,
           where(high_vol, 3,
           where(normal_vol, 2, 1)))

  emit regime
```

### Regime-Adaptive Strategy

```sig
signal adaptive_momentum:
  // Detect regime
  vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  high_vol = vol > quantile(vol, 0.8)

  // Different signals for different regimes
  momentum = zscore(ret(prices, 60))
  reversion = -zscore(ret(prices, 5))

  // Momentum in low vol, reversion in high vol
  signal = where(high_vol, reversion, momentum)

  emit signal
```

## Trend Regimes

### Moving Average Regime

```sig
signal trend_regime:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)

  // Trend classification
  uptrend = ma_50 > ma_200
  downtrend = ma_50 < ma_200

  emit where(uptrend, 1, where(downtrend, -1, 0))
```

### ADX Trend Strength

```sig
signal trend_strength:
  // Simplified ADX
  high_change = high - lag(high, 1)
  low_change = lag(low, 1) - low

  plus_dm = where(high_change > low_change and high_change > 0, high_change, 0)
  minus_dm = where(low_change > high_change and low_change > 0, low_change, 0)

  atr_14 = atr(high, low, close, 14)

  plus_di = 100 * rolling_mean(plus_dm, 14) / atr_14
  minus_di = 100 * rolling_mean(minus_dm, 14) / atr_14

  dx = 100 * abs(plus_di - minus_di) / (plus_di + minus_di)
  adx = rolling_mean(dx, 14)

  // Strong trend > 25
  strong_trend = adx > 25

  emit strong_trend
```

### Trend-Following Adaptive

```sig
signal trend_adaptive:
  // Detect trend strength
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  trend_signal = (ma_50 - ma_200) / ma_200

  // Trend strength
  trend_strength = abs(trend_signal)
  strong_trend = trend_strength > 0.05

  // Momentum for trends, mean reversion for range
  momentum = zscore(ret(prices, 60))
  reversion = -zscore((prices - ma_50) / ma_50)

  signal = where(strong_trend, momentum, reversion)

  emit signal
```

## Correlation Regimes

### Market Correlation

```sig
signal corr_regime:
  // Average pairwise correlation
  ret_1d = ret(prices, 1)
  avg_corr = cross_sectional_correlation(ret_1d, 60)

  // High correlation = risk-off
  high_corr = avg_corr > 0.6

  emit high_corr
```

### Dispersion Trading

```sig
signal dispersion:
  ret_1d = ret(prices, 1)

  // Cross-sectional dispersion
  dispersion = std(ret_1d)  // Cross-sectional std
  avg_dispersion = rolling_mean(dispersion, 60)

  // High dispersion = stock picking opportunity
  high_dispersion = dispersion > avg_dispersion * 1.5

  emit high_dispersion
```

## Hidden Markov Models

### Two-State HMM

```yaml
regime_detection:
  method: hmm
  states: 2
  features:
    - returns
    - volatility
  lookback: 252
```

### Using HMM Output

```sig
signal hmm_adaptive:
  // hmm_state from external model (0 or 1)
  bull_regime = hmm_state == 1

  momentum = zscore(ret(prices, 60))
  defensive = zscore(1 / rolling_std(ret(prices, 1), 60))

  // Momentum in bull, defensive in bear
  signal = where(bull_regime, momentum, defensive)

  emit signal
```

## Economic Regimes

### Growth/Inflation Matrix

```sig
signal economic_regime:
  // Simplified: use proxy indicators
  growth_proxy = rolling_mean(ret(prices, 60), 60)  // Market as growth proxy
  inflation_proxy = commodity_index_return

  high_growth = growth_proxy > 0
  high_inflation = inflation_proxy > quantile(inflation_proxy, 0.7)

  // Four quadrants
  // 1: High growth, low inflation (Goldilocks)
  // 2: High growth, high inflation (Overheating)
  // 3: Low growth, low inflation (Deflation)
  // 4: Low growth, high inflation (Stagflation)

  regime = where(high_growth and not(high_inflation), 1,
           where(high_growth and high_inflation, 2,
           where(not(high_growth) and not(high_inflation), 3, 4)))

  emit regime
```

## Regime-Switching Portfolio

### Complete Example

```sig
data:
  source = "prices_with_vix.parquet"
  format = parquet

// Regime detection
signal volatility_regime:
  vol = rolling_std(ret(prices, 1), 20) * sqrt(252)
  long_vol = rolling_std(ret(prices, 1), 60) * sqrt(252)
  high_vol = vol > long_vol * 1.3
  emit high_vol

signal trend_regime:
  ma_50 = rolling_mean(prices, 50)
  ma_200 = rolling_mean(prices, 200)
  uptrend = ma_50 > ma_200
  emit uptrend

// Strategy signals
signal momentum:
  emit zscore(ret(prices, 60))

signal mean_reversion:
  z = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)
  emit -zscore(z)

signal low_volatility:
  vol = rolling_std(ret(prices, 1), 60)
  emit -zscore(vol)

// Regime-adaptive composite
signal adaptive:
  high_vol = volatility_regime
  uptrend = trend_regime

  // Strategy weights by regime
  // Bull + Low Vol: Full momentum
  // Bull + High Vol: Momentum + Low Vol
  // Bear + Low Vol: Mean reversion
  // Bear + High Vol: Defensive (low vol)

  bull_low_vol = uptrend and not(high_vol)
  bull_high_vol = uptrend and high_vol
  bear_low_vol = not(uptrend) and not(high_vol)
  bear_high_vol = not(uptrend) and high_vol

  signal = where(bull_low_vol, momentum,
           where(bull_high_vol, 0.5 * momentum + 0.5 * low_volatility,
           where(bear_low_vol, mean_reversion,
                 low_volatility)))

  emit signal

portfolio regime_switching:
  weights = rank(adaptive).long_short(top=0.2, bottom=0.2, cap=0.03)

  constraints:
    max_sector = 0.25

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31
```

## Regime Transition

### Smooth Transitions

```sig
signal smooth_regime:
  vol = rolling_std(ret(prices, 1), 20) * sqrt(252)
  vol_percentile = ts_rank(vol, 252) / 252

  // Smooth weight from 0 (low vol) to 1 (high vol)
  high_vol_weight = clip(vol_percentile, 0, 1)

  momentum = zscore(ret(prices, 60))
  defensive = -zscore(vol)

  // Blend based on regime probability
  signal = (1 - high_vol_weight) * momentum + high_vol_weight * defensive

  emit signal
```

## Best Practices

### 1. Use Multiple Indicators

Don't rely on single regime indicator:

```sig
// Combine multiple signals
vol_high = vol > threshold1
trend_down = ma_50 < ma_200
corr_high = avg_corr > threshold2

defensive_regime = vol_high or trend_down or corr_high
```

### 2. Avoid Frequent Switching

```sig
// Add persistence
regime_raw = vol > threshold
regime_smooth = rolling_mean(regime_raw, 10) > 0.5
```

### 3. Test Regime Stability

Ensure regime detection is robust out-of-sample.

### 4. Consider Transaction Costs

Regime switching increases turnover.

## Next Steps

- [Factor Models](factor-models.md) - Factor-based strategies
- [Risk Models](risk-models.md) - Risk management
- [Portfolio Optimization](portfolio-optimization.md) - Optimal allocation
