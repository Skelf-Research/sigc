# Chapter 1: Introduction to Quantitative Finance

## What is Quantitative Finance?

Quantitative finance applies mathematical models, statistical analysis, and computational techniques to financial markets. Instead of relying on intuition or fundamental analysis alone, quants use data to make systematic, repeatable investment decisions.

### The Quant Approach

Traditional investing often relies on:
- Gut feelings
- News interpretation
- Analyst recommendations
- Qualitative company analysis

Quantitative investing instead uses:
- Historical data analysis
- Statistical models
- Systematic rules
- Automated execution

Neither approach is inherently superior. The best practitioners often combine both, using quantitative methods to validate or refine qualitative insights.

## Why Systematic Trading?

### Advantages

**Discipline**: Rules-based systems remove emotional decision-making. You won't panic sell at the bottom or greed-buy at the top.

**Scalability**: A good signal can be applied across thousands of securities simultaneously.

**Reproducibility**: Strategies can be tested, validated, and improved systematically.

**Speed**: Computers can process information and act faster than humans.

### Challenges

**Overfitting**: The biggest risk in quant finance. A model that perfectly fits historical data may fail catastrophically on new data.

**Data Quality**: Garbage in, garbage out. Bad data leads to bad decisions.

**Regime Changes**: Markets evolve. What worked in 2010 may not work in 2024.

**Competition**: Markets are competitive. Edge gets arbitraged away.

## The Signal-Based Framework

sigc is built around the concept of **signals** - numerical scores that predict future asset performance.

```
Raw Data → Signal Generation → Portfolio Construction → Execution → Returns
```

### What is a Signal?

A signal is a time series of scores for each asset. Higher scores indicate expected outperformance.

Example signals:
- **Momentum**: Assets that went up tend to keep going up
- **Value**: Cheap assets tend to outperform expensive ones
- **Quality**: Profitable companies tend to outperform unprofitable ones
- **Low Volatility**: Less volatile assets tend to have better risk-adjusted returns

### From Signal to Portfolio

1. **Generate signals** for each asset at each time point
2. **Rank assets** by signal strength
3. **Construct portfolio** (e.g., long top decile, short bottom decile)
4. **Backtest** to evaluate historical performance
5. **Deploy** if results are robust

## sigc Philosophy

sigc was designed with several principles:

### Simplicity

The DSL focuses on expressing signals clearly:

```sig
data prices = load("prices.csv")
signal momentum = (prices / lag(prices, 20)) - 1
output momentum
```

No boilerplate, no complex frameworks. Express your idea directly.

### Performance

Backtesting millions of data points should be fast:
- SIMD-optimized computations
- Memory-mapped data loading
- Incremental computation

### Production-Ready

Research code shouldn't be thrown away for production:
- Configuration management
- Alerting and monitoring
- Audit logging
- Data quality validation

### Correctness

Avoiding common pitfalls:
- No look-ahead bias in data alignment
- Proper handling of corporate actions
- Realistic transaction cost modeling

## Your First Strategy

Let's build a simple momentum strategy to understand the workflow.

### The Hypothesis

**Momentum Effect**: Assets that have performed well recently tend to continue performing well in the short term.

This is one of the most robust phenomena in finance, documented across asset classes and time periods.

### The Implementation

```sig
// momentum.sig - A simple 20-day momentum strategy

// Load price data
data prices = load("data/prices.csv")

// Calculate 20-day return (momentum signal)
signal returns_20d = (prices - lag(prices, 20)) / lag(prices, 20)

// Smooth with 5-day average to reduce noise
signal momentum = sma(returns_20d, 5)

// Output for portfolio construction
output momentum
```

### Running the Backtest

```bash
# Compile and run
sigc run momentum.sig

# View detailed results
sigc run momentum.sig --export results/

# Examine the output
ls results/
# equity_curve.csv  trades.csv  metrics.json
```

### Interpreting Results

```
Backtest Results
================
Total Return:      15.23%
Annualized Return: 12.45%
Sharpe Ratio:      1.32
Max Drawdown:      -8.45%
Win Rate:          54.2%
```

**What do these mean?**

- **Total Return**: How much money did we make?
- **Annualized Return**: Yearly average return
- **Sharpe Ratio**: Return per unit of risk (>1 is good, >2 is excellent)
- **Max Drawdown**: Worst peak-to-trough decline
- **Win Rate**: Percentage of profitable trades

## Common Misconceptions

### "More complexity = Better performance"

False. Simple strategies are often more robust. Complex models overfit more easily.

### "Past performance predicts future results"

Not directly. But properly validated strategies with economic rationale have better odds.

### "You need expensive data"

Not to start. Many effective signals use just price and volume data, which is freely available.

### "You need a PhD in math"

Helpful but not required. Basic statistics and programming skills are sufficient to begin.

## The Journey Ahead

In this guide, you'll learn:

1. **Mathematical foundations** needed for signal development
2. **How to build signals** from various data sources
3. **The sigc language** for expressing strategies
4. **Backtesting methodology** to validate ideas
5. **Risk management** to protect capital
6. **Production deployment** to run strategies live

Each chapter builds on the previous, with practical examples you can run and modify.

## Exercises

1. **Run the example**: Execute `sigc run examples/momentum.sig` and examine the output.

2. **Modify the lookback**: Change the 20-day lookback to 60 days. How do results change?

3. **Think about hypothesis**: Why might momentum work? What could cause it to stop working?

## Next Chapter

[Chapter 2: Mathematical Fundamentals](02-fundamentals.md) - The mathematical building blocks you'll need.
