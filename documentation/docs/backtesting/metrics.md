# Performance Metrics

Comprehensive metrics for evaluating strategy performance.

## Return Metrics

### Total Return

Cumulative return over the backtest period:

$$\text{Total Return} = \frac{V_{end} - V_{start}}{V_{start}}$$

```
Total Return: 85.2%
```

### CAGR (Compound Annual Growth Rate)

Annualized return accounting for compounding:

$$\text{CAGR} = \left(\frac{V_{end}}{V_{start}}\right)^{\frac{1}{years}} - 1$$

```
CAGR: 13.1%
```

### Monthly Returns

Returns by calendar month:

```
Monthly Returns:
         Jan    Feb    Mar    Apr    May    Jun    Jul    Aug    Sep    Oct    Nov    Dec   Year
2020   -1.2%   2.3%  -4.1%   5.2%   3.1%   1.8%  -0.5%   4.2%  -1.3%   2.1%   5.3%   2.8%  21.2%
2021    2.1%   1.8%   3.2%   2.1%  -1.5%   2.3%   1.8%   2.5%  -2.1%   3.2%   1.5%   2.3%  21.5%
2022   -3.2%  -2.1%   1.5%  -4.2%  -1.8%  -3.5%   2.1%  -1.5%  -4.3%   3.2%   2.8%  -1.5% -12.8%
2023    3.2%   1.5%   2.1%   1.8%   2.3%   3.1%   2.5%   1.2%  -0.8%  -1.2%   4.2%   3.5%  25.8%
2024    1.8%   2.3%   2.1%   3.2%   1.5%   2.8%   ...
```

### Rolling Returns

Returns over rolling windows:

| Period | 1M | 3M | 6M | 1Y | 3Y |
|--------|----|----|----|----|-----|
| Return | 2.8% | 7.2% | 12.1% | 18.5% | 52.3% |

## Risk Metrics

### Volatility

Annualized standard deviation of returns:

$$\text{Volatility} = \sigma_{daily} \times \sqrt{252}$$

```
Annualized Volatility: 15.2%
```

### Downside Volatility

Standard deviation of negative returns only:

$$\text{Downside Vol} = \sqrt{\frac{\sum_{r_i < 0} r_i^2}{n}} \times \sqrt{252}$$

```
Downside Volatility: 10.8%
```

### Maximum Drawdown

Largest peak-to-trough decline:

$$\text{MDD} = \max_t \left(\frac{\max_{s \leq t} V_s - V_t}{\max_{s \leq t} V_s}\right)$$

```
Max Drawdown: -18.5%
Max Drawdown Duration: 145 days
Max Drawdown Start: 2022-01-03
Max Drawdown End: 2022-10-12
Recovery Date: 2023-02-15
```

### Value at Risk (VaR)

Maximum expected loss at confidence level:

```
VaR (95%): -2.1% (daily)
           -4.5% (weekly)
           -9.2% (monthly)

VaR (99%): -3.5% (daily)
```

Interpretation: 95% of days, losses won't exceed 2.1%.

### Conditional VaR (CVaR / Expected Shortfall)

Average loss beyond VaR:

```
CVaR (95%): -3.2% (daily)
```

Interpretation: When losses exceed VaR, average loss is 3.2%.

## Risk-Adjusted Metrics

### Sharpe Ratio

Return per unit of risk:

$$\text{Sharpe} = \frac{R_p - R_f}{\sigma_p}$$

```
Sharpe Ratio: 0.86
```

Interpretation:

| Sharpe | Quality |
|--------|---------|
| < 0.5 | Poor |
| 0.5-1.0 | Average |
| 1.0-1.5 | Good |
| > 1.5 | Excellent |

### Sortino Ratio

Return per unit of downside risk:

$$\text{Sortino} = \frac{R_p - R_f}{\sigma_{downside}}$$

```
Sortino Ratio: 1.21
```

### Calmar Ratio

CAGR relative to maximum drawdown:

$$\text{Calmar} = \frac{\text{CAGR}}{|\text{Max Drawdown}|}$$

```
Calmar Ratio: 0.71
```

### Information Ratio

Excess return per unit of tracking error:

$$\text{IR} = \frac{R_p - R_b}{\text{TE}}$$

```
Information Ratio: 0.45
```

### Omega Ratio

Probability-weighted gains vs losses:

$$\Omega = \frac{\int_\theta^\infty (1 - F(r)) dr}{\int_{-\infty}^\theta F(r) dr}$$

```
Omega Ratio (0%): 1.35
```

## Benchmark Comparison

### Alpha and Beta

CAPM regression coefficients:

$$R_p - R_f = \alpha + \beta (R_m - R_f) + \epsilon$$

```
Alpha: 4.2% (annualized)
Beta: 0.72
```

### Tracking Error

Volatility of excess returns:

```
Tracking Error: 9.3%
```

### Up/Down Capture

Performance in up vs down markets:

```
Up Capture: 85%    (captures 85% of benchmark gains)
Down Capture: 62%  (captures 62% of benchmark losses)
Capture Ratio: 1.37  (85/62)
```

### Correlation

```
Correlation with SPY: 0.78
Correlation with QQQ: 0.65
```

## Distribution Metrics

### Skewness

Asymmetry of returns:

```
Skewness: -0.15
```

- Negative: More extreme losses than gains
- Positive: More extreme gains than losses

### Kurtosis

Tail heaviness:

```
Kurtosis: 3.8 (excess: 0.8)
```

- 3.0: Normal distribution
- \>3.0: Fat tails (more extreme events)
- <3.0: Thin tails

### Win Rate

Percentage of positive return periods:

```
Win Rate (daily): 52.3%
Win Rate (weekly): 55.8%
Win Rate (monthly): 58.3%
```

### Profit Factor

Gross profits / Gross losses:

```
Profit Factor: 1.28
```

## Trading Metrics

### Turnover

Portfolio turnover rate:

```
Annual Turnover: 245%
Monthly Turnover: 20.4%
Per-Rebalance Turnover: 18.5%
```

### Trade Count

```
Total Trades: 4,523
Avg Trades per Rebalance: 85
```

### Position Count

```
Avg Long Positions: 42
Avg Short Positions: 38
Max Positions: 95
Min Positions: 35
```

### Concentration

```
Top 10 Positions: 35% of portfolio
HHI (Herfindahl): 0.02
Effective N: 48 (1/HHI)
```

## Period Analysis

### Best/Worst Periods

```
Best Day: +4.2% (2020-11-09)
Worst Day: -3.8% (2020-03-16)

Best Month: +8.5% (2020-11)
Worst Month: -8.2% (2020-03)

Best Year: +25.8% (2023)
Worst Year: -12.8% (2022)
```

### Drawdown Table

```
Top 5 Drawdowns:
Rank | Start      | End        | Recovery   | Depth    | Duration
-----+------------+------------+------------+----------+---------
1    | 2022-01-03 | 2022-10-12 | 2023-02-15 | -18.5%   | 282 days
2    | 2020-02-19 | 2020-03-23 | 2020-06-08 | -12.3%   | 110 days
3    | 2023-07-31 | 2023-10-27 | 2023-12-08 |  -8.7%   | 130 days
4    | 2021-09-02 | 2021-10-04 | 2021-11-08 |  -6.2%   |  67 days
5    | 2024-04-01 | 2024-04-19 | 2024-05-15 |  -5.1%   |  44 days
```

## Accessing Metrics in Code

### CLI Output

```bash
sigc run strategy.sig --metrics all
```

### JSON Export

```bash
sigc run strategy.sig --output metrics.json --format json
```

```json
{
  "total_return": 0.852,
  "cagr": 0.131,
  "volatility": 0.152,
  "sharpe_ratio": 0.86,
  "max_drawdown": -0.185,
  "calmar_ratio": 0.71,
  "alpha": 0.042,
  "beta": 0.72
}
```

### Python Integration

```python
import pysigc

results = pysigc.run("strategy.sig")
print(f"Sharpe: {results.sharpe_ratio:.2f}")
print(f"Max DD: {results.max_drawdown:.1%}")
```

## Metric Interpretation Guide

### What Makes a Good Strategy?

| Metric | Poor | Acceptable | Good | Excellent |
|--------|------|------------|------|-----------|
| Sharpe | <0.5 | 0.5-0.8 | 0.8-1.2 | >1.2 |
| Max DD | >30% | 20-30% | 10-20% | <10% |
| Calmar | <0.5 | 0.5-0.8 | 0.8-1.2 | >1.2 |
| Win Rate | <50% | 50-55% | 55-60% | >60% |

### Red Flags

- Sharpe > 2.5 without explanation (likely overfit)
- Max DD recovering immediately (data snooping)
- No down years in 5+ year backtest
- Turnover > 1000% annually (cost-prohibitive)

## Next Steps

- [Cost Models](cost-models.md) - Impact of trading costs
- [Benchmark Analysis](benchmark-analysis.md) - Detailed benchmark comparison
- [Walk-Forward](walk-forward.md) - Out-of-sample validation
