# Return Attribution

Decompose strategy returns to understand what's driving performance.

## Overview

Attribution answers: "Where did returns come from?"

```
Total Return
├── Factor Returns
│   ├── Market (Beta)
│   ├── Size (SMB)
│   ├── Value (HML)
│   └── Momentum (MOM)
├── Sector Returns
│   ├── Allocation Effect
│   └── Selection Effect
└── Idiosyncratic (Alpha)
```

## Factor Attribution

### Single-Factor Model

Decompose returns vs market:

```sig
portfolio main:
  backtest benchmark=SPY from 2020-01-01 to 2024-12-31
```

```
Single-Factor Attribution:
  Total Return: 14.2%
  Market Contribution (β=0.72): 7.6%
  Alpha: 6.6%
```

### Multi-Factor Model (Fama-French)

```sig
portfolio main:
  backtest factors=[MKT, SMB, HML, MOM] from 2020-01-01 to 2024-12-31
```

Output:

```
Multi-Factor Attribution:
                                        Contribution
Factor      | Beta    | Factor Return | to Strategy
------------+---------+---------------+------------
Market      |  0.72   |    10.5%      |    7.6%
SMB (Size)  |  0.35   |     2.1%      |    0.7%
HML (Value) | -0.15   |    -1.5%      |    0.2%
MOM         |  0.45   |     5.2%      |    2.3%
------------+---------+---------------+------------
Total Factor|         |               |   10.8%
Alpha       |         |               |    3.4%
Total       |         |               |   14.2%
```

### Factor Exposure Over Time

```bash
sigc run strategy.sig --report factor-exposures
```

Shows how factor betas change over time:

```
Rolling 60-Day Factor Exposures:

        Market  SMB     HML     MOM
2020Q1  0.85    0.25   -0.10    0.55
2020Q2  0.78    0.30   -0.05    0.48
2020Q3  0.72    0.35   -0.12    0.42
...
2024Q3  0.68    0.38   -0.18    0.52
```

## Sector Attribution

### Brinson Attribution

Decompose returns by sector allocation and selection:

```sig
portfolio main:
  backtest benchmark=SPY sectors=gics from 2020-01-01 to 2024-12-31
```

```
Brinson Attribution:

Sector       |Port Wt|Bench Wt|Port Ret|Bench Ret|Alloc |Select|Total
-------------+-------+--------+--------+---------+------+------+-----
Technology   | 28%   | 25%    | 18.5%  | 15.2%   |+0.46%|+0.92%|+1.38%
Healthcare   | 15%   | 13%    | 12.1%  | 10.5%   |+0.21%|+0.24%|+0.45%
Financials   | 10%   | 12%    |  8.5%  | 11.2%   |-0.22%|-0.27%|-0.49%
Consumer Disc| 12%   | 10%    | 14.2%  | 12.1%   |+0.24%|+0.25%|+0.49%
Industrials  |  8%   | 10%    |  9.8%  |  8.5%   |-0.17%|+0.10%|-0.07%
...          |       |        |        |         |      |      |
-------------+-------+--------+--------+---------+------+------+-----
Total        |100%   |100%    | 13.1%  | 11.9%   |+0.35%|+0.85%|+1.20%
```

### Attribution Effects

**Allocation Effect**: Return from over/underweighting sectors

$$\text{Allocation} = (w_p - w_b) \times (R_b^{sector} - R_b^{total})$$

**Selection Effect**: Return from stock selection within sectors

$$\text{Selection} = w_p \times (R_p^{sector} - R_b^{sector})$$

**Interaction Effect**: Combined allocation and selection

$$\text{Interaction} = (w_p - w_b) \times (R_p^{sector} - R_b^{sector})$$

## Position-Level Attribution

### Top Contributors

```
Top 10 Contributors:
Rank | Ticker | Avg Weight | Return | Contribution
-----+--------+------------+--------+-------------
1    | NVDA   |    3.2%    | +125%  |    +4.0%
2    | META   |    2.8%    |  +85%  |    +2.4%
3    | MSFT   |    2.5%    |  +45%  |    +1.1%
4    | AMZN   |    2.3%    |  +52%  |    +1.2%
5    | GOOGL  |    2.1%    |  +38%  |    +0.8%
...
```

### Top Detractors

```
Top 10 Detractors:
Rank | Ticker | Avg Weight | Return | Contribution
-----+--------+------------+--------+-------------
1    | PYPL   |   -2.5%    | -25%   |    -0.6%
2    | INTC   |   -1.8%    | -35%   |    -0.6%
3    | DIS    |   -2.0%    | -18%   |    -0.4%
4    | BA     |   -1.5%    | -22%   |    -0.3%
...
```

## Time-Based Attribution

### Monthly Attribution

```
Monthly Return Attribution:

Month    |Market |Size   |Value  |Mom    |Alpha  |Total
---------+-------+-------+-------+-------+-------+------
2024-01  | +1.2% | +0.2% | -0.1% | +0.5% | +0.3% | +2.1%
2024-02  | +1.5% | +0.1% | +0.0% | +0.3% | +0.2% | +2.1%
2024-03  | +0.8% | +0.3% | -0.2% | +0.2% | +0.4% | +1.5%
...
```

### Cumulative Attribution

```
Cumulative Attribution (2020-2024):

Category    | Contribution
------------+-------------
Market      |   38.2%
Size        |    8.5%
Value       |   -2.1%
Momentum    |   12.3%
Alpha       |   28.3%
------------+-------------
Total       |   85.2%
```

## Risk Attribution

### Contribution to Volatility

```
Risk Attribution:

Factor      | Risk Contribution | % of Total
------------+-------------------+-----------
Market      |      10.2%        |    67%
Size        |       2.1%        |    14%
Value       |       0.8%        |     5%
Momentum    |       1.5%        |    10%
Idiosync.   |       0.6%        |     4%
------------+-------------------+-----------
Total Vol   |      15.2%        |   100%
```

### Marginal Risk Contribution

```
Marginal Contribution to Risk (MCTR):

Sector      | Weight | MCTR   | Contribution
------------+--------+--------+-------------
Technology  |   28%  |  1.25  |    35.0%
Healthcare  |   15%  |  0.85  |    12.8%
Financials  |   10%  |  1.10  |    11.0%
...
```

## Attribution Report

Generate comprehensive attribution:

```bash
sigc run strategy.sig --report attribution
```

```
Return Attribution Report
=========================
Period: 2020-01-01 to 2024-12-31
Total Return: 85.2%

Factor Attribution (Fama-French + Momentum):
-------------------------------------------
Factor      | Exposure | Contribution | % of Return
------------+----------+--------------+------------
Market      |    0.72  |     38.2%    |    44.8%
SMB         |    0.35  |      8.5%    |    10.0%
HML         |   -0.15  |     -2.1%    |    -2.5%
MOM         |    0.45  |     12.3%    |    14.4%
Alpha       |          |     28.3%    |    33.2%
------------+----------+--------------+------------
Total       |          |     85.2%    |   100.0%

Sector Attribution (Brinson):
-----------------------------
Allocation Effect:   +3.5%
Selection Effect:    +8.5%
Interaction Effect:  +0.2%
Total Active:       +12.2%

Top Contributors:
-----------------
1. NVDA (+4.0%)
2. META (+2.4%)
3. AMZN (+1.2%)

Top Detractors:
---------------
1. PYPL (-0.6%)
2. INTC (-0.6%)
3. DIS (-0.4%)

Attribution Over Time:
----------------------
[Charts showing cumulative contribution by factor]
```

## Custom Attribution

### Define Custom Factors

```sig
signal my_quality_factor:
  emit zscore(roe)

signal my_growth_factor:
  emit zscore(revenue_growth)

portfolio main:
  backtest factors=[MKT, my_quality_factor, my_growth_factor] from ...
```

### Industry Attribution

```sig
portfolio main:
  backtest sectors=sic_industry from ...
```

### Country Attribution

```sig
portfolio main:
  backtest sectors=country from ...
```

## Interpreting Attribution

### What Good Attribution Looks Like

```
Diversified factor contributions:
- Multiple factors contributing positively
- No single factor dominating (>80%)
- Positive alpha indicating skill

Good: Market 30%, Size 15%, Value 10%, Mom 20%, Alpha 25%
Bad:  Market 95%, Alpha 5% (just beta, no skill)
```

### Red Flags

1. **Alpha > 50% of return**: May be unexplained risk
2. **Single factor dominance**: Strategy is just factor exposure
3. **Unstable exposures**: Betas change wildly over time
4. **Negative alpha**: Strategy destroys value vs factors

## Best Practices

### 1. Use Appropriate Factor Model

```sig
// For equity long-short
factors=[MKT, SMB, HML, MOM, QMJ]

// For momentum strategy
factors=[MKT, MOM, STR]  // Include short-term reversal
```

### 2. Check Factor Stability

Factor exposures should be relatively stable unless designed otherwise.

### 3. Validate Alpha

True alpha should:
- Persist out-of-sample
- Not be explained by known factors
- Survive transaction costs

### 4. Report Multiple Views

```bash
# Factor view
sigc run strategy.sig --report factor-attribution

# Sector view
sigc run strategy.sig --report sector-attribution

# Position view
sigc run strategy.sig --report position-attribution
```

### 5. Attribution Should Sum

```
Factor contributions + Alpha = Total Return
```

Always verify attribution sums correctly.

## Next Steps

- [Metrics](metrics.md) - Performance metrics
- [Benchmark Analysis](benchmark-analysis.md) - Benchmark comparison
- [Walk-Forward](walk-forward.md) - Out-of-sample validation
