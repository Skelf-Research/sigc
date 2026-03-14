# Quantitative Trading Guide

A comprehensive introduction to quantitative trading with sigc.

## Overview

This guide takes you from fundamentals to production trading:

| Chapter | Topic | Level |
|---------|-------|-------|
| [1. Introduction](01-introduction.md) | What is quant trading? | Beginner |
| [2. Fundamentals](02-fundamentals.md) | Market data and returns | Beginner |
| [3. Signals](03-signals.md) | Building alpha signals | Beginner |
| [4. Language](04-language.md) | sigc DSL mastery | Intermediate |
| [5. Backtesting](05-backtesting.md) | Proper validation | Intermediate |
| [6. Risk](06-risk.md) | Risk management | Intermediate |
| [7. Production](07-production.md) | Going live | Advanced |
| [8. Advanced Analytics](08-advanced-analytics.md) | Factor models | Advanced |
| [9. Deployment](09-deployment-safety.md) | Safety and ops | Advanced |

## Who This Is For

### Quant Researchers

- Learn to implement trading signals
- Understand backtesting best practices
- Build robust research workflows

### Software Developers

- Understand quant finance concepts
- Learn the sigc DSL
- Deploy production systems

### Finance Professionals

- Move from Excel to programmatic trading
- Automate research workflows
- Build systematic strategies

## Learning Path

### Week 1-2: Foundations

1. **[Introduction](01-introduction.md)** - Understand quant trading landscape
2. **[Fundamentals](02-fundamentals.md)** - Learn about returns, volatility, correlation
3. **[Installation](../getting-started/installation.md)** - Set up your environment

### Week 3-4: Building Signals

4. **[Signals](03-signals.md)** - Construct trading signals
5. **[Language](04-language.md)** - Master sigc syntax
6. **[Operators](../operators/index.md)** - Learn available tools

### Week 5-6: Testing

7. **[Backtesting](05-backtesting.md)** - Validate strategies properly
8. **[Risk](06-risk.md)** - Manage portfolio risk
9. **[Tutorials](../tutorials/index.md)** - Hands-on practice

### Week 7-8: Production

10. **[Production](07-production.md)** - Deploy trading systems
11. **[Advanced Analytics](08-advanced-analytics.md)** - Factor models
12. **[Deployment](09-deployment-safety.md)** - Safety and monitoring

## Key Concepts Preview

### What is Quantitative Trading?

Using mathematical models to identify trading opportunities:

```
Data → Analysis → Signal → Portfolio → Execution
```

### The sigc Approach

```sig
// 1. Load data
data:
  source = "prices.parquet"
  format = parquet

// 2. Compute signal
signal momentum:
  emit zscore(ret(prices, 60))

// 3. Construct portfolio
portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

### Why Systematic Trading?

| Aspect | Discretionary | Systematic |
|--------|--------------|------------|
| Emotions | Affected | Removed |
| Consistency | Variable | Consistent |
| Scalability | Limited | High |
| Backtesting | Difficult | Easy |
| Speed | Slow | Fast |

## Prerequisites

### Technical

- Basic programming knowledge
- Command line familiarity
- Text editor proficiency

### Finance

- Understanding of stocks/bonds
- Basic statistics (mean, std, correlation)
- Interest in markets

### No Prerequisites

- No Rust knowledge needed
- No advanced math required
- No prior quant experience needed

## Tools You'll Need

| Tool | Purpose |
|------|---------|
| sigc | Strategy development |
| Text editor/VSCode | Code editing |
| Terminal | Running sigc |
| Sample data | Testing strategies |

## Quick Start

```bash
# Install sigc
cargo install sigc

# Create first strategy
cat > momentum.sig << 'EOF'
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
EOF

# Run backtest
sigc run momentum.sig
```

## What You'll Build

By the end of this guide, you'll be able to:

1. **Research** - Build and test trading signals
2. **Backtest** - Validate strategies properly
3. **Risk Manage** - Control portfolio risk
4. **Deploy** - Run strategies in production
5. **Monitor** - Track and manage live trading

## Next Steps

Start with [Chapter 1: Introduction](01-introduction.md) to begin your quant trading journey.
