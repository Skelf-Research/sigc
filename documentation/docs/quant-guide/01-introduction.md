# Chapter 1: Introduction to Quantitative Trading

Welcome to the world of systematic trading.

## What is Quantitative Trading?

Quantitative trading uses mathematical models and computer programs to identify and execute trading opportunities:

```
Data → Analysis → Signal → Decision → Execution
```

Instead of relying on intuition or gut feelings, quant traders:

- Analyze historical data
- Build mathematical models
- Test strategies rigorously
- Execute systematically

## Systematic vs. Discretionary

| Aspect | Discretionary | Systematic |
|--------|--------------|------------|
| Decision Making | Human judgment | Rules-based |
| Emotions | Can interfere | Removed |
| Consistency | Variable | Consistent |
| Scalability | Limited | High |
| Speed | Slow | Fast |
| Backtesting | Difficult | Easy |

## Types of Quantitative Strategies

### 1. Statistical Arbitrage

Exploiting price relationships between related securities:

```sig
signal pairs_trade:
  // Trade the spread between related stocks
  spread = prices[GOOGL] - beta * prices[META]
  zscore = (spread - mean(spread)) / std(spread)
  emit -zscore
```

### 2. Factor Investing

Systematic exposure to return drivers:

```sig
signal multi_factor:
  momentum = zscore(ret(prices, 60))
  value = zscore(book_to_market)
  quality = zscore(roe)
  emit 0.4 * momentum + 0.3 * value + 0.3 * quality
```

### 3. Trend Following

Riding market trends:

```sig
signal trend:
  ma_fast = rolling_mean(prices, 20)
  ma_slow = rolling_mean(prices, 60)
  emit zscore(ma_fast - ma_slow)
```

### 4. Mean Reversion

Betting on prices returning to normal:

```sig
signal mean_reversion:
  z = (prices - rolling_mean(prices, 20)) / rolling_std(prices, 20)
  emit -z  // Buy oversold, short overbought
```

### 5. Market Making

Providing liquidity for a profit:

```
Not covered in this guide (requires low-latency infrastructure)
```

## The Quantitative Trading Process

### 1. Research

- Generate ideas from academic papers, market observations
- Formalize hypotheses mathematically
- Initial data exploration

### 2. Signal Development

```sig
// Transform ideas into computable signals
signal my_idea:
  // Mathematical implementation
  emit zscore(my_calculation)
```

### 3. Backtesting

```sig
// Test on historical data
portfolio test:
  weights = rank(my_idea).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

### 4. Validation

- Out-of-sample testing
- Walk-forward analysis
- Parameter stability

### 5. Production

- Deploy to live trading
- Monitor performance
- Manage risk

## Why Use sigc?

sigc provides a complete platform for quantitative trading:

### Domain-Specific Language

Write strategies naturally:

```sig
signal momentum:
  emit zscore(ret(prices, 60))
```

### Rigorous Backtesting

Test with realistic assumptions:

```sig
portfolio main:
  weights = ...
  costs = tc.bps(10)  // Transaction costs
  backtest from 2015-01-01 to 2024-12-31
```

### Production Ready

Deploy the same code to live trading:

```bash
sigc daemon start --strategy momentum.sig --live
```

### Performance

Built in Rust for speed:
- Process millions of data points
- Run parameter optimizations
- Execute in real-time

## Key Concepts Preview

### Signals

Numerical scores indicating expected returns:

```sig
signal alpha:
  // Higher score = expect higher return
  emit zscore(ret(prices, 60))
```

### Portfolios

Converting signals to positions:

```sig
portfolio main:
  weights = rank(alpha).long_short(top=0.2, bottom=0.2)
```

### Backtesting

Simulating historical performance:

```sig
backtest from 2015-01-01 to 2024-12-31
```

### Risk Management

Controlling exposure:

```sig
constraints:
  max_position = 0.03
  max_sector = 0.20
  net_exposure = 0.0
```

## Who Should Use This Guide?

### Aspiring Quants

Learn the fundamentals of systematic trading from scratch.

### Software Developers

Understand financial concepts and sigc's approach.

### Finance Professionals

Transition from Excel to programmatic trading.

### Data Scientists

Apply ML/statistics skills to financial markets.

## What You'll Learn

By the end of this guide:

1. **Understand** quant trading fundamentals
2. **Build** trading signals from data
3. **Backtest** strategies properly
4. **Manage** portfolio risk
5. **Deploy** to production
6. **Monitor** live trading

## Prerequisites

### Minimal Requirements

- Basic programming concepts
- High school mathematics
- Interest in financial markets

### No Need For

- Prior Rust knowledge
- Advanced mathematics
- Professional trading experience

## Getting Started

### Install sigc

```bash
cargo install sigc
```

### Your First Strategy

```sig
// Save as first_strategy.sig
data:
  source = "prices.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices

signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

```bash
sigc run first_strategy.sig
```

## Next Chapter

Continue to [Chapter 2: Fundamentals](02-fundamentals.md) to learn about returns, volatility, and other core concepts.
