# Chapter 2: Financial Fundamentals

Core concepts every quant trader must understand.

## Returns

### Simple Returns

The percentage change in price:

$$R_t = \frac{P_t - P_{t-1}}{P_{t-1}} = \frac{P_t}{P_{t-1}} - 1$$

```sig
signal simple_return:
  ret_1d = (prices - lag(prices, 1)) / lag(prices, 1)
  emit ret_1d
```

### Log Returns

Natural logarithm of price ratio:

$$r_t = \ln\left(\frac{P_t}{P_{t-1}}\right)$$

```sig
signal log_return:
  ret_1d = ln(prices / lag(prices, 1))
  emit ret_1d
```

### Why Log Returns?

| Property | Simple | Log |
|----------|--------|-----|
| Additive over time | No | Yes |
| Symmetric | No | Yes |
| Normal distribution | Less | More |
| Multi-period | Multiply | Add |

sigc default:
```sig
// ret() uses simple returns by default
ret_60 = ret(prices, 60)  // 60-day simple return
```

### Annualization

Convert daily to annual:

```sig
// Volatility: multiply by sqrt(252)
annual_vol = daily_vol * sqrt(252)

// Returns: multiply by 252
annual_ret = daily_ret * 252
```

## Volatility

### Standard Deviation

Most common volatility measure:

$$\sigma = \sqrt{\frac{1}{n-1}\sum_{i=1}^n(r_i - \bar{r})^2}$$

```sig
signal volatility:
  daily_ret = ret(prices, 1)
  vol = rolling_std(daily_ret, 60) * sqrt(252)  // Annualized
  emit vol
```

### Realized Volatility

Historical volatility from actual returns:

```sig
signal realized_vol:
  // Sum of squared returns
  daily_ret = ret(prices, 1)
  squared_ret = daily_ret * daily_ret
  rv = sqrt(rolling_sum(squared_ret, 21) * 252 / 21)
  emit rv
```

### Implied Volatility

Forward-looking volatility from options prices. Often proxied by VIX.

## Risk-Adjusted Returns

### Sharpe Ratio

Return per unit of risk:

$$\text{Sharpe} = \frac{R_p - R_f}{\sigma_p}$$

```sig
// In backtest results
sharpe = annual_return / annual_volatility
```

### Information Ratio

Active return per unit of active risk:

$$\text{IR} = \frac{R_p - R_b}{\sigma_{p-b}}$$

```sig
// Return vs benchmark, divided by tracking error
information_ratio = active_return / tracking_error
```

### Sortino Ratio

Uses downside deviation instead of total volatility:

$$\text{Sortino} = \frac{R_p - R_f}{\sigma_d}$$

More appropriate when return distribution is asymmetric.

## Correlation

### Pearson Correlation

Linear relationship between two variables:

$$\rho_{xy} = \frac{\text{Cov}(x,y)}{\sigma_x \sigma_y}$$

```sig
signal correlation:
  ret_aapl = ret(prices[AAPL], 1)
  ret_msft = ret(prices[MSFT], 1)
  corr = rolling_corr(ret_aapl, ret_msft, 60)
  emit corr
```

### Why Correlation Matters

- **Diversification**: Low correlation = better diversification
- **Risk**: High correlation in crisis = increased risk
- **Factor exposure**: Correlation reveals common factors

## Drawdowns

### Maximum Drawdown

Largest peak-to-trough decline:

$$\text{MaxDD} = \max_t\left(\frac{\text{Peak}_t - P_t}{\text{Peak}_t}\right)$$

```sig
signal drawdown:
  peak = rolling_max(prices, 252)
  dd = (peak - prices) / peak
  emit dd
```

### Drawdown Duration

Time underwater:
- **Drawdown start**: When price falls from peak
- **Drawdown end**: When new peak is reached
- **Recovery time**: Duration of drawdown

## Beta and Alpha

### Beta

Sensitivity to market movements:

$$\beta = \frac{\text{Cov}(R_i, R_m)}{\text{Var}(R_m)}$$

```sig
signal beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market_index, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
  emit beta
```

### Alpha

Return unexplained by market:

$$\alpha = R_i - \beta \cdot R_m$$

### CAPM

$$E[R_i] = R_f + \beta_i(E[R_m] - R_f)$$

Expected return = Risk-free rate + Beta × Market risk premium

## Statistical Concepts

### Z-Score

Standardized value (number of standard deviations from mean):

$$z = \frac{x - \mu}{\sigma}$$

```sig
signal momentum_zscore:
  raw_momentum = ret(prices, 60)
  // zscore() cross-sectionally normalizes
  emit zscore(raw_momentum)
```

### Percentile Rank

Position in distribution:

```sig
signal percentile:
  // Rank relative to history
  percentile = ts_rank(prices, 252) / 252
  emit percentile
```

### Cross-Sectional vs Time-Series

| Operation | Cross-Sectional | Time-Series |
|-----------|-----------------|-------------|
| zscore | Across assets today | Single asset over time |
| rank | Across assets today | Single asset over time |
| mean | Average of all assets | Average over time |

```sig
// Cross-sectional (default)
cs_zscore = zscore(momentum)

// Time-series
ts_zscore = (momentum - rolling_mean(momentum, 252)) / rolling_std(momentum, 252)
```

## Practical Example

### Computing Key Metrics

```sig
data:
  source = "prices.parquet"
  format = parquet

signal returns:
  emit ret(prices, 1)

signal volatility:
  daily_ret = ret(prices, 1)
  emit rolling_std(daily_ret, 60) * sqrt(252)

signal sharpe_estimate:
  daily_ret = ret(prices, 1)
  annual_ret = rolling_mean(daily_ret, 252) * 252
  annual_vol = rolling_std(daily_ret, 252) * sqrt(252)
  emit annual_ret / annual_vol

signal drawdown:
  peak = rolling_max(prices, 252)
  emit (peak - prices) / peak

signal beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  emit rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
```

## Key Takeaways

1. **Returns**: Use simple returns for most cases, log returns for compounding
2. **Volatility**: Annualize daily vol by multiplying by √252
3. **Sharpe Ratio**: Primary measure of risk-adjusted performance
4. **Correlation**: Critical for portfolio construction
5. **Drawdowns**: Maximum drawdown shows worst-case scenario
6. **Z-scores**: Standardize signals for comparison

## Exercises

1. Calculate the 60-day rolling Sharpe ratio for a stock
2. Compute the correlation between two stocks
3. Find the maximum drawdown in a price series
4. Calculate beta relative to SPY

## Next Chapter

Continue to [Chapter 3: Building Signals](03-signals.md) to learn how to construct trading signals.
