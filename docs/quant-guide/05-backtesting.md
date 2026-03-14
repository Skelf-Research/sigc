# Chapter 5: Backtesting Methodology

Backtesting is the process of testing a trading strategy on historical data. This chapter covers how to do it properly and avoid common mistakes.

## Why Backtest?

Backtesting serves several purposes:

1. **Validate hypotheses**: Does your idea work historically?
2. **Estimate performance**: What returns and risk to expect?
3. **Optimize parameters**: What settings work best?
4. **Build confidence**: Gain conviction before risking capital

**Warning**: A good backtest doesn't guarantee future success. Markets change, and past performance doesn't predict future results.

## Backtesting Framework

### sigc Backtesting Pipeline

```
Signal → Ranking → Portfolio Construction → Simulation → Metrics
```

1. **Signal**: Your alpha signal (from Chapter 3)
2. **Ranking**: Order assets by signal strength
3. **Portfolio Construction**: Assign weights to assets
4. **Simulation**: Simulate trades and P&L
5. **Metrics**: Evaluate performance

### Basic Backtest

```bash
# Run with default settings
sigc run strategy.sig

# Specify date range
sigc run strategy.sig --start 2015-01-01 --end 2023-12-31

# Export detailed results
sigc run strategy.sig --export results/
```

### Backtest Configuration

```sig
// strategy.sig
data prices = load("prices.csv")
signal momentum = prices / lag(prices, 60) - 1
output momentum

// Configuration in separate file or CLI
// --start 2015-01-01
// --end 2023-12-31
// --rebalance monthly
// --long-only false
// --transaction-cost 0.001
```

## Common Pitfalls

### 1. Look-Ahead Bias

Using information that wasn't available at the time.

**Example - Bad**:
```sig
// Using future prices
signal bad = lead(prices, 1) / prices - 1
```

**Example - Subtle**:
```sig
// Using point-in-time earnings data that was revised later
data earnings = load("earnings.csv")  // Contains revisions!

// Better: Use as-reported data
data earnings = load("earnings_asreported.csv")
```

**How to avoid**:
- Never use `lead()` in signals
- Use point-in-time data
- Check data timestamps carefully

### 2. Survivorship Bias

Only testing on companies that survived.

**Problem**: Your backtest doesn't include companies that went bankrupt, were delisted, or acquired. These are often the worst performers.

**Example**: Testing a value strategy only on current S&P 500 members ignores all the "cheap" companies that went bankrupt.

**How to avoid**:
- Use survivorship-bias-free databases
- Include delisted securities
- Handle corporate actions properly

### 3. Overfitting

Finding patterns that don't persist.

**Symptoms**:
- Too many parameters
- Spectacular in-sample results
- Poor out-of-sample performance
- No economic rationale

**Example - Overfit**:
```sig
// 10 parameters = 10 ways to overfit
signal overfit = sma(prices, 17) / ema(prices, 43) * rsi(prices, 11)
                 - bb(prices, 23, 2.3) + momentum(prices, 67) / vol(prices, 31)
```

**Example - Robust**:
```sig
// 2 parameters with economic rationale
signal robust = prices / lag(prices, 60) - 1  // Momentum (documented effect)
```

**How to avoid**:
- Use simple models
- Out-of-sample testing
- Multiple testing correction
- Economic rationale required

### 4. Data Snooping

Testing many hypotheses and reporting only the best.

**Problem**: If you test 100 signals, 5 will appear significant by chance at 5% level.

**How to avoid**:
- Pre-register hypotheses
- Apply Bonferroni or FDR corrections
- Use out-of-sample validation
- Report all tests, not just successes

### 5. Transaction Cost Neglect

Ignoring trading costs.

**Reality check**: A strategy that trades 100% of portfolio daily with 1 bps costs loses 2.5% annually just in costs.

```sig
// In backtest configuration
// --transaction-cost 0.001  // 10 bps round-trip
// --slippage 0.0005         // 5 bps market impact
```

### 6. Liquidity Constraints

Assuming you can trade any size at market prices.

**Problems**:
- Small caps have limited liquidity
- Large orders move prices
- Some securities can't be shorted

**How to avoid**:
- Filter by ADV (average daily volume)
- Model market impact
- Cap position sizes

## Performance Metrics

### Return Metrics

**Total Return**:
```
Total Return = (Final Value - Initial Value) / Initial Value
```

**Annualized Return**:
```
Ann Return = (1 + Total Return)^(252/days) - 1
```

**CAGR** (Compound Annual Growth Rate):
Same as annualized return, commonly used for long-term results.

### Risk Metrics

**Volatility**:
```
σ = std(daily_returns) × √252
```

**Maximum Drawdown**:
```
Max DD = max(Peak - Trough) / Peak
```

**Value at Risk (VaR)**:
Loss that won't be exceeded with X% confidence.
```
VaR_95 = percentile(returns, 5)
```

### Risk-Adjusted Metrics

**Sharpe Ratio**:
```
Sharpe = (Return - Risk Free Rate) / Volatility
```
- < 0.5: Poor
- 0.5-1.0: Acceptable
- 1.0-2.0: Good
- > 2.0: Excellent (be suspicious!)

**Sortino Ratio**:
Like Sharpe but only penalizes downside volatility:
```
Sortino = Return / Downside Deviation
```

**Calmar Ratio**:
```
Calmar = Annualized Return / Max Drawdown
```

**Information Ratio**:
```
IR = Alpha / Tracking Error
```

### Consistency Metrics

**Win Rate**:
```
Win Rate = Winning Trades / Total Trades
```

**Profit Factor**:
```
Profit Factor = Gross Profits / Gross Losses
```

**Hit Rate by Period**:
- Positive months / Total months
- Positive years / Total years

## Statistical Significance

A good Sharpe ratio might be just luck. Test for significance.

### t-Statistic

```
t = Sharpe × √(years)
```

**Rule of thumb**: t > 2 suggests significance at 5% level.

**Example**:
- Sharpe = 1.0, Years = 10 → t = 3.16 ✓
- Sharpe = 1.5, Years = 2 → t = 2.12 ✓
- Sharpe = 2.0, Years = 1 → t = 2.00 (borderline)

### Minimum Track Record

How long before a Sharpe ratio is "real"?

```
Min Years = (z_α / Sharpe)²
```

For 95% confidence (z = 1.96):
- Sharpe 0.5 → 15 years
- Sharpe 1.0 → 4 years
- Sharpe 2.0 → 1 year

### Multiple Testing Correction

When testing N strategies, adjust significance threshold:

**Bonferroni**: New threshold = α/N
- Test 100 strategies at 5% → Use 0.05% threshold

**False Discovery Rate**: Less conservative, controls proportion of false discoveries.

## Cross-Validation Methods

### Train/Test Split

Simple but wastes data.

```
|------ Train (70%) ------|-- Test (30%) --|
```

```bash
sigc run strategy.sig --train-end 2019-12-31 --test-start 2020-01-01
```

### Walk-Forward Analysis

Most realistic simulation.

```
|-- Train 1 --|-- Test 1 --|
      |-- Train 2 --|-- Test 2 --|
            |-- Train 3 --|-- Test 3 --|
```

```bash
sigc run strategy.sig --walk-forward --window 252 --step 63
```

**Advantages**:
- Uses all data
- Mimics real-world updating
- Most realistic

### K-Fold Cross-Validation

```
|-- Fold 1 --||-- Fold 2 --||-- Fold 3 --||-- Fold 4 --||-- Fold 5 --|
   Test          Train        Train        Train        Train
   Train         Test         Train        Train        Train
   ...
```

Less common in finance due to temporal nature of data.

### Combinatorial Purged Cross-Validation

Advanced method that:
- Removes overlapping data between train/test
- Tests all combinations
- Provides distribution of results

## Transaction Costs

### Components

1. **Commission**: Fixed or per-share fee
2. **Spread**: Bid-ask spread cost
3. **Market Impact**: Price movement from your order
4. **Slippage**: Difference between expected and actual price

### Modeling Costs

**Simple Model**:
```sig
// Flat cost per trade
// --transaction-cost 0.001  // 10 bps
```

**Spread Model**:
```sig
// Cost = half spread × turnover
// For 2 bps spread: cost = 1 bp per side
```

**Impact Model**:
```
Impact = σ × √(Volume / ADV)
```

Where ADV = average daily volume.

### Cost Sensitivity Analysis

Test strategy with different cost assumptions:

```bash
sigc run strategy.sig --transaction-cost 0.0005   # 5 bps
sigc run strategy.sig --transaction-cost 0.001    # 10 bps
sigc run strategy.sig --transaction-cost 0.002    # 20 bps
```

A robust strategy should remain profitable across reasonable cost assumptions.

## Realistic Simulation

### Corporate Actions

Handle splits, dividends, mergers properly:

```bash
sigc run strategy.sig --adjust-corporate-actions
```

- **Splits**: Adjust prices and volumes
- **Dividends**: Include in returns or reinvest
- **Mergers**: Handle delisting

### Market Conditions

Test across different regimes:
- Bull markets (2009-2020)
- Bear markets (2008, 2020)
- High volatility (2008, 2020)
- Low volatility (2017)
- Rising rates (2022)

### Constraints

Apply realistic constraints:

```bash
sigc run strategy.sig \
  --max-position 0.05 \      # Max 5% in any position
  --min-adv 1000000 \        # Min $1M daily volume
  --sector-neutral \         # Neutralize sector bets
  --max-turnover 2.0         # Max 200% annual turnover
```

## Interpreting Results

### What to Look For

**Good signs**:
- Consistent across time periods
- Works in multiple markets
- Robust to parameter changes
- Clear economic rationale
- t-statistic > 2

**Red flags**:
- Sensitive to exact parameters
- Only works in one period
- Incredible Sharpe (>3)
- Complex with no explanation
- Huge drawdowns

### Typical Results

| Metric | Mediocre | Good | Excellent |
|--------|----------|------|-----------|
| Sharpe Ratio | 0.5 | 1.0 | 1.5+ |
| Annual Return | 5% | 10% | 15%+ |
| Max Drawdown | -30% | -15% | -10% |
| Win Rate | 50% | 55% | 60%+ |

**Remember**: Exceptional results require exceptional scrutiny.

### Decay Analysis

Measure how quickly signal loses power:

| Horizon | Expected IC |
|---------|-------------|
| 1 day | 0.03-0.05 |
| 5 days | 0.02-0.04 |
| 20 days | 0.01-0.03 |

Fast decay = high turnover = high costs.

## Complete Backtest Example

```bash
# 1. Run initial backtest
sigc run momentum.sig --start 2010-01-01 --end 2023-12-31

# 2. Walk-forward validation
sigc run momentum.sig --walk-forward --window 756 --step 252

# 3. Cost sensitivity
sigc run momentum.sig --transaction-cost 0.001
sigc run momentum.sig --transaction-cost 0.002

# 4. Export detailed results
sigc run momentum.sig --export results/

# 5. Examine outputs
ls results/
# equity_curve.csv  monthly_returns.csv  metrics.json  trades.csv
```

### Analyzing Results

```python
import pandas as pd

# Load results
equity = pd.read_csv("results/equity_curve.csv")
monthly = pd.read_csv("results/monthly_returns.csv")

# Check consistency
positive_months = (monthly['return'] > 0).mean()
print(f"Positive months: {positive_months:.1%}")

# Check for regime sensitivity
# Split by volatility regime
high_vol = monthly[monthly['vol'] > monthly['vol'].median()]
low_vol = monthly[monthly['vol'] <= monthly['vol'].median()]
print(f"High vol Sharpe: {high_vol['return'].mean() / high_vol['return'].std() * np.sqrt(12):.2f}")
print(f"Low vol Sharpe: {low_vol['return'].mean() / low_vol['return'].std() * np.sqrt(12):.2f}")
```

## Checklist

Before deploying a strategy, verify:

- [ ] Passes walk-forward validation
- [ ] t-statistic > 2
- [ ] Works with realistic transaction costs
- [ ] Handles corporate actions
- [ ] Respects liquidity constraints
- [ ] Has clear economic rationale
- [ ] Robust to parameter variations
- [ ] Tested across market regimes
- [ ] No look-ahead bias
- [ ] Uses survivorship-free data

## Key Takeaways

1. **Backtests lie**: They're optimistic by nature
2. **Simplicity wins**: Complex models overfit
3. **Costs matter**: Include realistic friction
4. **Test significance**: Is it luck or skill?
5. **Walk-forward**: Most realistic validation
6. **Be skeptical**: Extraordinary claims require extraordinary evidence

## Exercises

1. **Basic backtest**: Run the momentum example and examine the metrics.

2. **Cost sensitivity**: How much do results degrade with 20 bps costs vs 10 bps?

3. **Walk-forward**: Run walk-forward validation. How do results compare to full-sample?

4. **Regime analysis**: Split your backtest into bull/bear markets. Does strategy work in both?

5. **Significance test**: Calculate the t-statistic for your strategy's Sharpe ratio.

## Next Chapter

[Chapter 6: Risk Management](06-risk.md) - Protect your capital.
