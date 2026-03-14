# Chapter 2: Mathematical Fundamentals

This chapter covers the essential mathematical concepts you'll use throughout your quant journey. Don't worry if some concepts seem abstract at first - they'll become intuitive with practice.

## Returns

Returns are the fundamental measure of performance in finance.

### Simple Returns

The simple return is the percentage change in price:

```
r = (P_t - P_{t-1}) / P_{t-1} = P_t / P_{t-1} - 1
```

In sigc:

```sig
data prices = load("prices.csv")
signal returns = (prices - lag(prices, 1)) / lag(prices, 1)
```

**Properties**:
- Intuitive interpretation
- Asset-additive (portfolio return = weighted sum of asset returns)
- Cannot be time-aggregated by simple addition

### Log Returns

Log returns (continuously compounded) are the natural log of price ratio:

```
r_log = ln(P_t / P_{t-1})
```

In sigc:

```sig
signal log_returns = log(prices / lag(prices, 1))
```

**Properties**:
- Time-additive (multi-period return = sum of single-period returns)
- Better for statistical modeling (more normally distributed)
- Approximately equal to simple returns for small values

**When to use which?**
- Use log returns for statistical analysis and modeling
- Use simple returns for performance reporting and portfolio aggregation

### Multi-Period Returns

For simple returns, compounding:
```
r_{t:t+n} = (1 + r_t)(1 + r_{t+1})...(1 + r_{t+n}) - 1
```

For log returns, just add:
```
r_log_{t:t+n} = r_log_t + r_log_{t+1} + ... + r_log_{t+n}
```

In sigc:
```sig
// 20-day cumulative log return
signal cum_ret = sum(log_returns, 20)

// 20-day simple return
signal ret_20d = prices / lag(prices, 20) - 1
```

## Volatility

Volatility measures the dispersion of returns. It's the standard unit of risk in finance.

### Standard Deviation

The most common volatility measure:

```sig
// Realized volatility (20-day)
signal volatility = std(returns, 20)
```

### Annualization

Volatility scales with the square root of time:

```
σ_annual = σ_daily × √252
```

Why 252? There are approximately 252 trading days in a year.

```sig
// Annualized volatility
signal ann_vol = std(returns, 20) * sqrt(252)
```

### Other Volatility Measures

**Exponentially Weighted**:
```sig
// More weight on recent observations
signal ewm_vol = ewm_std(returns, 20)
```

**Parkinson (High-Low)**:
```sig
// Uses intraday range
signal parkinson = sqrt(log(high / low) ** 2 / (4 * log(2)))
```

**Garman-Klass**:
```sig
// Uses OHLC data
signal gk_vol = sqrt(0.5 * log(high/low)**2 - (2*log(2)-1) * log(close/open)**2)
```

### Why Volatility Matters

1. **Risk measurement**: Higher volatility = more risk
2. **Position sizing**: Size positions inversely to volatility
3. **Signal normalization**: Standardize signals by volatility
4. **Option pricing**: Directly determines option values

## Correlation and Covariance

Understanding relationships between assets is crucial for portfolio construction.

### Correlation

Correlation measures linear relationship between two variables (-1 to +1):

```
ρ_{X,Y} = Cov(X, Y) / (σ_X × σ_Y)
```

- ρ = +1: Perfect positive correlation
- ρ = 0: No linear relationship
- ρ = -1: Perfect negative correlation

### Covariance

Covariance measures how two variables move together:

```
Cov(X, Y) = E[(X - μ_X)(Y - μ_Y)]
```

Unlike correlation, covariance is unbounded.

### Why Correlation Matters

**Diversification**: Assets with low correlation provide diversification benefits.

Consider two assets with:
- Expected return: 10%
- Volatility: 20%

| Correlation | Portfolio Volatility |
|-------------|---------------------|
| +1.0        | 20%                |
| +0.5        | 17.3%              |
| 0.0         | 14.1%              |
| -0.5        | 10%                |
| -1.0        | 0%                 |

Negative correlation is rare but valuable!

### Correlation Pitfalls

**Non-stationarity**: Correlations change over time, especially in crises.

**Non-linearity**: Correlation only measures linear relationships.

**Tail dependence**: Assets may be uncorrelated normally but highly correlated in crashes.

## Time Series Concepts

Financial data is time series data. Understanding its properties is essential.

### Stationarity

A stationary series has constant statistical properties over time:
- Constant mean
- Constant variance
- Constant autocorrelation structure

**Why it matters**: Most statistical methods assume stationarity. Price series are non-stationary, but return series are (approximately) stationary.

### Autocorrelation

Autocorrelation measures correlation of a series with its lagged self:

```
ρ_k = Cor(r_t, r_{t-k})
```

**In returns**:
- Short-term: Slight negative (mean reversion)
- Medium-term: Slight positive (momentum)
- Long-term: Near zero (efficiency)

### Mean Reversion vs Momentum

Two fundamental market dynamics:

**Mean Reversion**: What goes up must come down
```sig
// Mean reversion signal: buy when below average
signal deviation = prices - sma(prices, 20)
signal mean_rev = -zscore(deviation, 60)
```

**Momentum**: Trend continuation
```sig
// Momentum signal: buy what's going up
signal momentum = prices / lag(prices, 60) - 1
```

These effects operate on different timescales:
- Very short-term (days): Mean reversion
- Medium-term (3-12 months): Momentum
- Long-term (3-5 years): Mean reversion

### Seasonality

Regular patterns at fixed intervals:
- Day-of-week effects
- Month-of-year effects (January effect)
- Quarter-end rebalancing

## Distribution of Returns

Understanding the distribution of returns helps with risk management.

### Normal Distribution

Returns are often assumed to be normally distributed for simplicity. However:

**Reality check**:
- Returns have **fat tails** (more extreme events than normal)
- Returns are often **negatively skewed** (bigger down moves)
- Returns exhibit **volatility clustering** (calm periods and turbulent periods)

### Skewness

Skewness measures asymmetry:
- Positive skew: Right tail is longer
- Negative skew: Left tail is longer

Stock returns typically have negative skewness - crashes are more severe than rallies.

### Kurtosis

Kurtosis measures tail thickness:
- Normal distribution: kurtosis = 3
- Fat tails: kurtosis > 3 (leptokurtic)

Stock returns have kurtosis of 5-10, meaning extreme events are much more common than a normal distribution suggests.

### Implications

1. **Don't trust 3-sigma**: In a normal distribution, 3-sigma events are rare (0.3%). In markets, they happen much more often.

2. **Stress test**: Test your strategy against historical crashes and hypothetical scenarios.

3. **Manage tail risk**: Consider options or other hedges for extreme events.

## Practical Examples in sigc

### Complete Statistical Analysis

```sig
data prices = load("prices.csv")

// Basic returns
signal returns = prices / lag(prices, 1) - 1
signal log_ret = log(prices / lag(prices, 1))

// Volatility measures
signal vol_20 = std(returns, 20) * sqrt(252)
signal vol_60 = std(returns, 60) * sqrt(252)

// Trend measures
signal sma_20 = sma(prices, 20)
signal sma_60 = sma(prices, 60)
signal trend = sma_20 / sma_60 - 1

// Mean reversion
signal zscore_20 = (prices - sma_20) / std(prices, 20)

// Momentum
signal mom_20 = prices / lag(prices, 20) - 1
signal mom_60 = prices / lag(prices, 60) - 1

// Volatility-adjusted momentum
signal sharpe_mom = mom_20 / vol_20

output sharpe_mom
```

### Volatility Targeting

Normalize signals by volatility for consistent risk:

```sig
data prices = load("prices.csv")

signal returns = prices / lag(prices, 1) - 1
signal vol = std(returns, 20)

// Raw signal
signal momentum = prices / lag(prices, 20) - 1

// Volatility-targeted signal
// Higher vol = lower signal (less position)
signal vol_target = 0.10  // 10% target volatility
signal scaled_signal = momentum * (vol_target / (vol * sqrt(252)))

output scaled_signal
```

## Key Takeaways

1. **Use log returns for analysis**, simple returns for reporting
2. **Volatility is not constant** - use rolling windows
3. **Correlations change** - especially in crises
4. **Returns are not normal** - have fat tails and skewness
5. **Normalize signals** by volatility for consistent risk

## Exercises

1. **Compare returns**: Load a price series and calculate both simple and log returns. Plot them and observe the difference during large moves.

2. **Rolling volatility**: Calculate 20-day and 60-day volatility. When do they diverge?

3. **Correlation regimes**: Calculate rolling correlation between two assets. Does it increase during market stress?

4. **Test normality**: Calculate the kurtosis of daily returns. How does it compare to 3 (normal)?

## Next Chapter

[Chapter 3: Building Trading Signals](03-signals.md) - Apply these concepts to build predictive signals.
