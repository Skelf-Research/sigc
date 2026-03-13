# Operators Reference

Complete reference of all sigc operators organized by category.

## Arithmetic

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `+` | `a + b` | Addition | `prices + 1` |
| `-` | `a - b` | Subtraction | `high - low` |
| `*` | `a * b` | Multiplication | `price * volume` |
| `/` | `a / b` | Division | `profit / capital` |
| `abs` | `abs(x)` | Absolute value | `abs(returns)` |
| `sign` | `sign(x)` | Sign (-1, 0, 1) | `sign(momentum)` |
| `log` | `log(x)` | Natural logarithm | `log(prices)` |
| `exp` | `exp(x)` | Exponential | `exp(rate)` |
| `pow` | `pow(x, n)` | Power | `pow(vol, 2)` |
| `sqrt` | `sqrt(x)` | Square root | `sqrt(variance)` |
| `floor` | `floor(x)` | Round down | `floor(score)` |
| `ceil` | `ceil(x)` | Round up | `ceil(score)` |
| `round` | `round(x)` | Round to nearest | `round(weight)` |
| `min` | `min(a, b)` | Minimum of two | `min(price, cap)` |
| `max` | `max(a, b)` | Maximum of two | `max(score, 0)` |
| `clip` | `clip(x, lo, hi)` | Clamp to range | `clip(weight, -0.1, 0.1)` |

## Comparison

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `>` | `a > b` | Greater than | `volume > avg_vol` |
| `<` | `a < b` | Less than | `price < target` |
| `>=` | `a >= b` | Greater or equal | `score >= threshold` |
| `<=` | `a <= b` | Less or equal | `risk <= limit` |
| `==` | `a == b` | Equal | `sector == tech` |
| `!=` | `a != b` | Not equal | `status != halted` |

## Logical

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `and` | `a and b` | Logical AND | `bullish and volume_up` |
| `or` | `a or b` | Logical OR | `oversold or reversal` |
| `not` | `not(x)` | Logical NOT | `not(excluded)` |
| `where` | `where(cond, a, b)` | Conditional | `where(bull, long, short)` |

## Data Handling

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `is_nan` | `is_nan(x)` | Check for NaN | `is_nan(price)` |
| `fill_nan` | `fill_nan(x, val)` | Replace NaN | `fill_nan(score, 0)` |
| `coalesce` | `coalesce(a, b)` | First non-NaN | `coalesce(est, actual)` |
| `cumsum` | `cumsum(x)` | Cumulative sum | `cumsum(returns)` |
| `cumprod` | `cumprod(x)` | Cumulative product | `cumprod(1 + r)` |
| `cummax` | `cummax(x)` | Cumulative max | `cummax(price)` |
| `cummin` | `cummin(x)` | Cumulative min | `cummin(drawdown)` |

## Time-Series

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `lag` | `lag(x, n)` | Shift by n periods | `lag(price, 1)` |
| `ret` | `ret(x, n)` | n-period return | `ret(price, 20)` |
| `delta` | `delta(x, n)` | n-period difference | `delta(volume, 5)` |
| `rolling_mean` | `rolling_mean(x, n)` | Moving average | `rolling_mean(price, 20)` |
| `rolling_std` | `rolling_std(x, n)` | Moving std dev | `rolling_std(ret, 20)` |
| `rolling_sum` | `rolling_sum(x, n)` | Moving sum | `rolling_sum(volume, 10)` |
| `rolling_min` | `rolling_min(x, n)` | Moving minimum | `rolling_min(price, 52)` |
| `rolling_max` | `rolling_max(x, n)` | Moving maximum | `rolling_max(price, 52)` |
| `rolling_corr` | `rolling_corr(x, y, n)` | Moving correlation | `rolling_corr(a, b, 60)` |
| `ema` | `ema(x, span)` | Exponential MA | `ema(price, 20)` |
| `decay_linear` | `decay_linear(x, n)` | Linear decay avg | `decay_linear(signal, 10)` |
| `ts_argmax` | `ts_argmax(x, n)` | Index of max | `ts_argmax(price, 20)` |
| `ts_argmin` | `ts_argmin(x, n)` | Index of min | `ts_argmin(price, 20)` |
| `ts_rank` | `ts_rank(x, n)` | Time-series rank | `ts_rank(ret, 20)` |
| `ts_skew` | `ts_skew(x, n)` | Rolling skewness | `ts_skew(ret, 60)` |
| `ts_kurt` | `ts_kurt(x, n)` | Rolling kurtosis | `ts_kurt(ret, 60)` |
| `ts_product` | `ts_product(x, n)` | Rolling product | `ts_product(1+r, 20)` |
| `ts_zscore` | `ts_zscore(x, n)` | Time-series zscore | `ts_zscore(ret, 60)` |

## Cross-Sectional

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `zscore` | `zscore(x)` | Cross-sectional z-score | `zscore(returns)` |
| `rank` | `rank(x)` | Cross-sectional rank | `rank(momentum)` |
| `rank_pct` | `rank_pct(x)` | Percentile rank (0-1) | `rank_pct(score)` |
| `scale` | `scale(x)` | Scale to sum=1 | `scale(abs(signal))` |
| `demean` | `demean(x)` | Subtract mean | `demean(beta)` |
| `winsor` | `winsor(x, p)` | Winsorize at percentile | `winsor(score, 0.01)` |
| `neutralize` | `neutralize(x, g)` | Group neutralize | `neutralize(alpha, sector)` |
| `quantile` | `quantile(x, q)` | Quantile value | `quantile(ret, 0.75)` |
| `bucket` | `bucket(x, n)` | Assign to n buckets | `bucket(score, 5)` |
| `median` | `median(x)` | Cross-sectional median | `median(returns)` |
| `mad` | `mad(x)` | Median abs deviation | `mad(returns)` |

## Portfolio Construction

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `long_short` | `rank(x).long_short(top, bottom)` | Long/short weights | `rank(sig).long_short(0.2, 0.2)` |

## Technical Indicators

| Operator | Syntax | Description | Example |
|----------|--------|-------------|---------|
| `rsi` | `rsi(x, n)` | Relative Strength Index | `rsi(price, 14)` |
| `macd` | `macd(x, fast, slow, sig)` | MACD indicator | `macd(price, 12, 26, 9)` |
| `atr` | `atr(h, l, c, n)` | Average True Range | `atr(high, low, close, 14)` |
| `vwap` | `vwap(p, v)` | Volume-weighted price | `vwap(price, volume)` |

## Usage Examples

### Momentum Factor
```
signal momentum:
  r20 = ret(prices, 20)
  r5 = ret(prices, 5)
  mom = r20 - r5
  emit zscore(mom)
```

### Mean Reversion
```
signal mean_reversion:
  ma = rolling_mean(prices, 20)
  std = rolling_std(prices, 20)
  z = (prices - ma) / std
  emit -zscore(z)
```

### Volatility-Adjusted
```
signal vol_adj:
  ret = ret(prices, 1)
  vol = rolling_std(ret, 20)
  score = ret / vol
  emit winsor(score, 0.01)
```

### Multi-Factor
```
signal combo:
  mom = zscore(ret(prices, 20))
  rev = -zscore(ret(prices, 5))
  combined = 0.6 * mom + 0.4 * rev
  emit combined
```
