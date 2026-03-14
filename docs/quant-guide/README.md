# The Quant Guide

A comprehensive educational resource for quantitative finance with sigc.

## Who This Guide Is For

- **Aspiring Quants**: Developers transitioning into quantitative finance
- **Finance Professionals**: Analysts looking to automate their strategies
- **Researchers**: Academics exploring systematic trading approaches
- **Hobbyists**: Anyone interested in data-driven investing

## Prerequisites

- Basic programming knowledge (any language)
- Familiarity with statistics (mean, standard deviation)
- Understanding of financial markets (stocks, prices, returns)

## Table of Contents

### Part I: Foundations

1. **[Introduction to Quantitative Finance](01-introduction.md)**
   - What is quantitative finance?
   - The role of systematic trading
   - Overview of sigc and its philosophy

2. **[Mathematical Fundamentals](02-fundamentals.md)**
   - Returns and log returns
   - Volatility and risk
   - Correlation and covariance
   - Time series basics

### Part II: Signal Development

3. **[Building Trading Signals](03-signals.md)**
   - What makes a good signal
   - Technical indicators
   - Fundamental factors
   - Alternative data signals
   - Signal combination and blending

4. **[The sigc Language](04-language.md)**
   - Syntax and semantics
   - Data operations
   - Built-in functions
   - Common patterns

### Part III: Validation

5. **[Backtesting Methodology](05-backtesting.md)**
   - Backtesting fundamentals
   - Common pitfalls and biases
   - Transaction costs and slippage
   - Performance metrics
   - Statistical significance

6. **[Risk Management](06-risk.md)**
   - Position sizing
   - Portfolio construction
   - Drawdown control
   - Tail risk and stress testing

### Part IV: Production

7. **[Going Live](07-production.md)**
   - Infrastructure setup
   - Data pipelines
   - Monitoring and alerts
   - Continuous improvement

### Part V: Advanced Topics

8. **[Advanced Analytics](08-advanced-analytics.md)**
   - Factor models (Fama-French, Barra)
   - Risk models (VaR, CVaR, stress testing)
   - Regime detection (HMM, clustering)
   - Portfolio optimization (mean-variance, risk parity, Black-Litterman)

9. **[Deployment & Safety](09-deployment-safety.md)**
   - Trading safety systems (kill switch, circuit breakers)
   - Position limits and order validation
   - Docker containerization
   - Monitoring and health checks

10. **[Language Enhancements](10-language-enhancements.md)**
    - Custom function definitions
    - Macro system for reusable patterns
    - Type inference and operator signatures
    - VS Code extension and LSP server
    - Improved error messages with diagnostics

11. **[External Integrations](11-integrations.md)**
    - Market data providers (Yahoo Finance)
    - Broker APIs (Alpaca)
    - WebSocket streaming
    - Custom provider framework

## How to Use This Guide

**Linear Learning**: Work through chapters 1-7 sequentially for a complete education.

**Reference**: Jump to specific topics as needed when building strategies.

**Hands-On**: Each chapter includes examples you can run with sigc.

## Quick Start

```bash
# Install sigc
cargo install sigc

# Run your first backtest
sigc run examples/momentum.sig

# Explore the output
sigc run examples/momentum.sig --export results/
```

## Example Strategy

```sig
// Simple momentum strategy
data prices = load("data/prices.csv")

// Calculate 20-day momentum
signal momentum = (prices / lag(prices, 20)) - 1

// Output the signal
output momentum
```

## Additional Resources

- [API Reference](../reference/)
- [Example Strategies](../examples/)
- [Production Features](../advanced/production-features.md)
- [Performance Tuning](../advanced/)

## Contributing

Found an error or want to improve the guide? Contributions welcome!

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
