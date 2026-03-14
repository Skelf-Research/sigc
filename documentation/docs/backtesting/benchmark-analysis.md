# Benchmark Analysis

Compare strategy performance against market benchmarks.

## Specifying a Benchmark

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest rebal=21 benchmark=SPY from 2020-01-01 to 2024-12-31
```

## Common Benchmarks

| Benchmark | Description | Use For |
|-----------|-------------|---------|
| `SPY` | S&P 500 ETF | US large cap |
| `QQQ` | Nasdaq 100 ETF | US tech/growth |
| `IWM` | Russell 2000 ETF | US small cap |
| `VTI` | Total US Market | Broad US |
| `EFA` | EAFE ETF | International developed |
| `EEM` | Emerging Markets ETF | Emerging markets |
| `AGG` | US Aggregate Bond | Bonds |
| `cash` | Risk-free rate | Absolute return |

## Benchmark Metrics

### Alpha

Excess return after adjusting for market exposure:

$$\alpha = R_p - (R_f + \beta (R_m - R_f))$$

```
Alpha: 4.2% (annualized)
```

Interpretation: Strategy generated 4.2% above what beta exposure would predict.

### Beta

Sensitivity to benchmark:

$$\beta = \frac{\text{Cov}(R_p, R_m)}{\text{Var}(R_m)}$$

```
Beta: 0.72
```

Interpretation: When benchmark moves 1%, strategy moves ~0.72%.

### Tracking Error

Volatility of excess returns:

$$\text{TE} = \sigma(R_p - R_b)$$

```
Tracking Error: 9.3% (annualized)
```

### Information Ratio

Alpha per unit of tracking error:

$$\text{IR} = \frac{\alpha}{\text{TE}}$$

```
Information Ratio: 0.45
```

| IR | Quality |
|-----|---------|
| < 0.25 | Poor |
| 0.25-0.5 | Average |
| 0.5-0.75 | Good |
| > 0.75 | Excellent |

### Active Share

How different is the portfolio from benchmark:

```
Active Share: 85%
```

- < 20%: Closet indexer
- 20-60%: Moderate active
- > 60%: Truly active

## Up/Down Capture

### Up Capture Ratio

Performance when benchmark is up:

$$\text{Up Capture} = \frac{\text{Avg Strategy Return (up months)}}{\text{Avg Benchmark Return (up months)}}$$

```
Up Capture: 85%
```

### Down Capture Ratio

Performance when benchmark is down:

$$\text{Down Capture} = \frac{\text{Avg Strategy Return (down months)}}{\text{Avg Benchmark Return (down months)}}$$

```
Down Capture: 62%
```

### Capture Ratio

```
Capture Ratio = Up Capture / Down Capture = 85% / 62% = 1.37
```

> 1.0 means strategy captures more upside than downside.

## Relative Performance

### Cumulative Excess Return

```
Period           | Strategy | Benchmark | Excess
-----------------+----------+-----------+-------
2020             |   21.2%  |   18.4%   | +2.8%
2021             |   18.5%  |   28.7%   | -10.2%
2022             |   -8.2%  |  -18.1%   | +9.9%
2023             |   25.8%  |   26.3%   | -0.5%
2024 (YTD)       |   12.3%  |   15.2%   | -2.9%
-----------------+----------+-----------+-------
Cumulative       |   85.2%  |   75.4%   | +9.8%
```

### Rolling Excess Return

```bash
sigc run strategy.sig --report rolling-alpha
```

Shows 12-month rolling alpha over time.

## Benchmark Attribution

### Single-Factor Model

```
Strategy Return = Alpha + Beta × Benchmark Return + Residual

Example:
  Strategy: +12%
  Benchmark: +10%
  Beta: 0.8

  Expected: 0.8 × 10% = 8%
  Alpha: 12% - 8% = 4%
```

### Multi-Factor Model

```sig
portfolio main:
  backtest benchmark=SPY factors=[SMB, HML, MOM] from ...
```

Output:

```
Factor Attribution:
Factor    | Exposure | Factor Return | Contribution
----------+----------+---------------+-------------
Market    |    0.72  |      10.5%    |      7.6%
SMB       |    0.35  |       2.1%    |      0.7%
HML       |   -0.15  |      -1.5%    |      0.2%
MOM       |    0.45  |       5.2%    |      2.3%
----------+----------+---------------+-------------
Factor Sum|          |               |     10.8%
Alpha     |          |               |      3.4%
Total     |          |               |     14.2%
```

## Correlation Analysis

### Rolling Correlation

```bash
sigc run strategy.sig --report correlation
```

```
Rolling 60-day Correlation with SPY:
  Min: 0.45
  Max: 0.92
  Mean: 0.72
  Current: 0.68
```

### Correlation Matrix

```
           Strategy  SPY    QQQ    IWM
Strategy     1.00   0.72   0.65   0.58
SPY          0.72   1.00   0.88   0.82
QQQ          0.65   0.88   1.00   0.72
IWM          0.58   0.82   0.72   1.00
```

## Regime Analysis

Performance in different market environments:

### By Market Direction

```
Market Regime    | # Months | Strategy | Benchmark | Excess
-----------------+----------+----------+-----------+-------
Strong Up (>5%)  |    18    |   4.2%   |    8.5%   | -4.3%
Mild Up (0-5%)   |    32    |   2.8%   |    2.1%   | +0.7%
Mild Down (-5-0) |    25    |  -1.5%   |   -2.3%   | +0.8%
Strong Down(<-5%)|    10    |  -3.2%   |   -8.5%   | +5.3%
```

### By Volatility Regime

```
VIX Regime      | # Months | Strategy | Benchmark | Excess
----------------+----------+----------+-----------+-------
Low (<15)       |    28    |   1.8%   |    1.5%   | +0.3%
Medium (15-25)  |    42    |   0.9%   |    0.5%   | +0.4%
High (>25)      |    15    |  -0.5%   |   -2.1%   | +1.6%
```

## Benchmark Selection

### Match Your Universe

```sig
// US large cap strategy
backtest benchmark=SPY ...

// Small cap strategy
backtest benchmark=IWM ...

// Tech focus
backtest benchmark=QQQ ...
```

### Long-Short Strategies

For market-neutral strategies:

```sig
// Compare to cash (risk-free rate)
backtest benchmark=cash ...

// Or no benchmark
backtest from ...  // Absolute return
```

### Multi-Asset

```sig
// 60/40 portfolio benchmark
backtest benchmark=[SPY:0.6, AGG:0.4] ...
```

## Custom Benchmarks

### Equal-Weight Benchmark

```sig
data benchmark:
  source = "benchmark_weights.csv"

portfolio main:
  backtest benchmark=benchmark from ...
```

### Factor Benchmark

```sig
// Your own factor portfolio as benchmark
signal benchmark_signal:
  emit zscore(ret(prices, 60))

portfolio benchmark_port:
  weights = rank(benchmark_signal).long_short(top=0.5, bottom=0.5)

portfolio main:
  backtest benchmark=benchmark_port from ...
```

## Best Practices

### 1. Use Appropriate Benchmark

Match benchmark to strategy universe and style.

### 2. Consider Multiple Benchmarks

```sig
portfolio main:
  backtest benchmark=[SPY, QQQ, IWM] from ...
```

### 3. Report Both Absolute and Relative

```
Absolute Performance:
  CAGR: 13.1%
  Sharpe: 0.86

Relative Performance (vs SPY):
  Alpha: 4.2%
  Information Ratio: 0.45
```

### 4. Analyze Regime Dependence

Understand when strategy outperforms vs underperforms.

### 5. Check Benchmark Fit

```
R-squared: 0.65
```

Low R² means benchmark may not be appropriate.

## Example: Complete Benchmark Report

```bash
sigc run strategy.sig --report benchmark
```

```
Benchmark Analysis Report
=========================
Strategy: momentum_strategy
Benchmark: SPY
Period: 2020-01-01 to 2024-12-31

Return Comparison:
                 | Strategy | Benchmark | Difference
-----------------+----------+-----------+-----------
Total Return     |   85.2%  |   75.4%   |   +9.8%
CAGR             |   13.1%  |   11.9%   |   +1.2%
Volatility       |   15.2%  |   18.5%   |   -3.3%
Sharpe Ratio     |    0.86  |    0.64   |   +0.22
Max Drawdown     |  -18.5%  |  -24.5%   |   +6.0%

Risk-Adjusted Relative:
  Alpha:              4.2%
  Beta:               0.72
  Tracking Error:     9.3%
  Information Ratio:  0.45

Capture Analysis:
  Up Capture:    85%
  Down Capture:  62%
  Capture Ratio: 1.37

Correlation:
  Full Period: 0.72
  Rolling Min: 0.45
  Rolling Max: 0.92

Active Share: 85%

Best Relative Months:
  2022-10: Strategy -2.1%, Benchmark -9.3% (+7.2%)
  2020-03: Strategy -8.5%, Benchmark -12.5% (+4.0%)

Worst Relative Months:
  2021-11: Strategy +1.2%, Benchmark +6.8% (-5.6%)
  2023-12: Strategy +2.1%, Benchmark +4.5% (-2.4%)
```

## Next Steps

- [Attribution](attribution.md) - Detailed return attribution
- [Metrics](metrics.md) - All performance metrics
- [Walk-Forward](walk-forward.md) - Validation methodology
