# Chapter 8: Advanced Analytics

Factor models, attribution, and deep performance analysis.

## Factor Models

### What are Factors?

Factors are systematic drivers of returns:

```
Stock Return = Factor Returns + Idiosyncratic Return
             = β₁F₁ + β₂F₂ + ... + ε
```

### Common Factor Models

**CAPM (1 factor):**
```
R = α + β(Rm - Rf) + ε
```

**Fama-French 3 Factor:**
```
R = α + β₁(MKT) + β₂(SMB) + β₃(HML) + ε
```

**Carhart 4 Factor:**
```
R = α + β₁(MKT) + β₂(SMB) + β₃(HML) + β₄(MOM) + ε
```

### Factor Definitions

| Factor | Name | Construction |
|--------|------|--------------|
| MKT | Market | Market return - Risk-free |
| SMB | Size | Small cap - Large cap |
| HML | Value | High B/M - Low B/M |
| MOM | Momentum | Winners - Losers |
| QMJ | Quality | High quality - Low quality |

## Factor Exposure Analysis

### Calculate Factor Exposures

```sig
signal market_beta:
  stock_ret = ret(prices, 1)
  market_ret = ret(market, 1)
  beta = rolling_cov(stock_ret, market_ret, 60) / rolling_var(market_ret, 60)
  emit beta

signal size_exposure:
  // Negative of market cap (small = positive exposure)
  emit -zscore(ln(market_cap))

signal value_exposure:
  emit zscore(book_value / market_cap)

signal momentum_exposure:
  emit zscore(ret(prices, 252) - ret(prices, 21))
```

### Portfolio Factor Exposures

```bash
sigc run strategy.sig --factor-analysis
```

Output:
```
Factor Exposure Analysis
========================

Portfolio Factor Exposures:
  Market (β):    0.05 (near neutral)
  Size (SMB):   -0.12 (slight large-cap tilt)
  Value (HML):   0.28 (value exposure)
  Momentum:      0.35 (momentum exposure)
  Quality:       0.18 (quality tilt)

Factor Contribution to Return:
  Market:      +2.1%
  Size:        -0.4%
  Value:       +1.8%
  Momentum:    +2.5%
  Quality:     +0.9%
  Alpha:       +1.3%
  -------------------------
  Total:       +8.2%
```

## Performance Attribution

### Returns-Based Attribution

Decompose returns by source:

```
Total Return = Factor Returns + Selection Return
             = Σ(βᵢ × Fᵢ) + α
```

### Holdings-Based Attribution

Decompose by position:

```
Portfolio Return = Σ(wᵢ × rᵢ)
                 = Σ(contribution per stock)
```

### Sector Attribution

```bash
sigc run strategy.sig --attribution sector
```

```
Sector Attribution
==================

Sector          Weight  Return  Contribution
Technology      +0.15   +12.3%      +1.85%
Healthcare      +0.08    +8.1%      +0.65%
Financials      -0.12    +5.2%      -0.62%
Consumer Disc.  +0.05   +15.2%      +0.76%
...

Total Sector Effect: +3.2%
Selection Effect:    +4.8%
Interaction:         +0.2%
Total Return:        +8.2%
```

## Risk Attribution

### Factor Risk Decomposition

```
Total Variance = Factor Variance + Idiosyncratic Variance
               = β'Σβ + σ²ε
```

### Risk Attribution Report

```bash
sigc run strategy.sig --risk-attribution
```

```
Risk Attribution
================

Total Volatility: 9.5%

Factor Risk Contribution:
  Market:     35%  (3.3%)
  Size:        8%  (0.8%)
  Value:      12%  (1.1%)
  Momentum:   15%  (1.4%)
  Quality:     5%  (0.5%)
  -----------------------
  Total Factor: 75%

Idiosyncratic Risk: 25% (2.4%)

Risk Concentration:
  Top 10 positions: 18% of risk
  Top sector (Tech): 22% of risk
```

## Advanced Performance Metrics

### Information Coefficient

Correlation of signal with forward returns:

```python
# In Python with pysigc
import pysigc
import numpy as np

results = pysigc.run("strategy.sig")
signals = results.signals
forward_returns = results.forward_returns

ic = np.corrcoef(signals, forward_returns)[0, 1]
print(f"IC: {ic:.4f}")
```

### Turnover Analysis

```bash
sigc run strategy.sig --turnover-analysis
```

```
Turnover Analysis
=================

Annual Turnover: 420%
  Long Side: 210%
  Short Side: 210%

Average Holding Period: 58 days

Turnover by Quintile:
  Q1 (longs):   35%
  Q2:           42%
  Q3:           55%
  Q4:           48%
  Q5 (shorts):  38%

Cost Analysis:
  Gross Return: 9.8%
  Trading Costs: -1.6%
  Net Return: 8.2%
```

### Hit Rate Analysis

```
Win Rate: 54%
Average Win: +2.1%
Average Loss: -1.8%
Profit Factor: 1.3

By Market Condition:
  Bull Markets: 58% win rate
  Bear Markets: 49% win rate
  High Vol: 52% win rate
  Low Vol: 56% win rate
```

## Regime Analysis

### Performance by Regime

```sig
signal volatility_regime:
  vol = rolling_std(ret(market, 1), 20) * sqrt(252)
  long_vol = rolling_std(ret(market, 1), 60) * sqrt(252)
  emit where(vol > long_vol * 1.3, "high_vol", "normal")

signal trend_regime:
  ma_50 = rolling_mean(market, 50)
  ma_200 = rolling_mean(market, 200)
  emit where(ma_50 > ma_200, "bull", "bear")
```

```bash
sigc run strategy.sig --regime-analysis
```

```
Regime Analysis
===============

By Volatility:
  Normal Vol:   Sharpe 0.95, Return +9.2%
  High Vol:     Sharpe 0.42, Return +4.1%

By Trend:
  Bull Market:  Sharpe 1.05, Return +11.3%
  Bear Market:  Sharpe 0.28, Return +2.1%

By Combination:
  Bull + Low Vol:   Sharpe 1.15
  Bull + High Vol:  Sharpe 0.55
  Bear + Low Vol:   Sharpe 0.45
  Bear + High Vol:  Sharpe 0.15
```

## Correlation Analysis

### Strategy Correlation with Factors

```
Correlation with Common Factors:
  Market (SPY):     0.05
  Size (IWM-SPY):  -0.12
  Value (IVE-IVW):  0.35
  Momentum:         0.42
  Quality:          0.22

Strategy is most correlated with momentum factor.
```

### Strategy Correlation Over Time

```
Rolling 1-Year Correlation with SPY:
  2020: 0.15
  2021: 0.08
  2022: 0.22
  2023: 0.05
  2024: 0.03

Correlation has decreased over time (good for diversification).
```

## Stress Testing

### Historical Stress Tests

```bash
sigc run strategy.sig --stress-test
```

```
Historical Stress Tests
=======================

Event                    Date        Return    Max DD
2008 Financial Crisis    2008-09    -12.3%    -18.5%
2011 Euro Crisis         2011-08     -5.2%     -8.1%
2015 China Deval         2015-08     -3.8%     -6.2%
2018 Vol Spike           2018-02     -4.5%     -7.8%
2020 COVID Crash         2020-03    -10.2%    -15.3%
2022 Rate Hikes          2022-06     -6.8%    -11.2%

Worst Single Day:        -4.2% (2020-03-12)
Worst Single Month:      -8.5% (2020-03)
```

### Hypothetical Scenarios

```yaml
stress_scenarios:
  - name: "Market Crash"
    market_return: -20%
    volatility_multiplier: 2.5

  - name: "Rate Shock"
    yield_change: +200bps
    duration_impact: true

  - name: "Sector Rotation"
    tech_return: -15%
    financials_return: +10%
```

## Complete Analytics Example

```sig
data:
  source = "prices_fundamentals.parquet"
  format = parquet

signal momentum:
  emit neutralize(zscore(ret(prices, 60)), by=sectors)

signal value:
  emit neutralize(zscore(book_to_market), by=sectors)

signal combined:
  emit 0.5 * momentum + 0.5 * value

portfolio analyzed:
  weights = rank(combined).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20

  costs = tc.bps(10)

  backtest rebal=21 from 2010-01-01 to 2024-12-31

  // Request full analytics
  analytics:
    factor_attribution: true
    risk_attribution: true
    regime_analysis: true
    turnover_analysis: true
    stress_tests: true
```

```bash
sigc run strategy.sig --full-analytics --output reports/analysis.html
```

## Best Practices

### 1. Understand Your Factor Exposures

Know what you're betting on:
- Intended exposures (from your signals)
- Unintended exposures (check and hedge)

### 2. Monitor Attribution Over Time

Factor contributions change:
- Is momentum still working?
- Has value exposure increased?

### 3. Test Across Regimes

Ensure robustness:
- Bull and bear markets
- High and low volatility
- Different economic conditions

### 4. Regular Deep Dives

Beyond daily monitoring:
- Monthly factor analysis
- Quarterly stress tests
- Annual strategy review

## Exercises

1. Calculate factor exposures for your strategy
2. Perform returns-based attribution
3. Analyze performance across volatility regimes
4. Run historical stress tests

## Next Chapter

Continue to [Chapter 9: Deployment and Safety](09-deployment-safety.md) for production safety systems.
