# Chapter 6: Risk Management

Risk management is what separates surviving traders from failed ones. This chapter covers how to size positions, construct portfolios, and protect capital.

## The Nature of Risk

### Types of Risk

**Market Risk**: Exposure to overall market movements
- Beta risk
- Systematic risk
- Undiversifiable

**Specific Risk**: Individual security risk
- Company-specific events
- Idiosyncratic
- Diversifiable

**Factor Risk**: Exposure to systematic factors
- Value, momentum, size
- Industry, country
- Interest rates, inflation

**Liquidity Risk**: Inability to exit positions
- Low volume securities
- Market stress periods
- Position concentration

**Model Risk**: Your model is wrong
- Overfitting
- Regime changes
- Parameter instability

### Risk vs Uncertainty

**Risk**: Known unknowns - can be estimated from history
**Uncertainty**: Unknown unknowns - can't be estimated

Markets have both. Manage risk, respect uncertainty.

## Position Sizing

Position sizing determines how much capital to allocate to each trade. It's often more important than signal selection.

### Equal Weight

Simplest approach:
```
Weight_i = 1/N
```

**Pros**: Simple, diversified
**Cons**: Ignores signal strength and risk

### Signal-Proportional

Weight by signal strength:
```
Weight_i = Signal_i / Σ|Signal_j|
```

**Pros**: Larger bets on stronger signals
**Cons**: Risk not controlled

### Volatility-Adjusted (Risk Parity)

Equal risk contribution:
```
Weight_i = (1/σ_i) / Σ(1/σ_j)
```

**Pros**: Balanced risk
**Cons**: High weight to low-vol assets (bonds)

### Volatility Targeting

Scale positions to target portfolio volatility:

```sig
data prices = load("prices.csv")
signal returns = prices / lag(prices, 1) - 1
signal vol = std(returns, 20)

// Raw signal
signal momentum = prices / lag(prices, 60) - 1

// Volatility target (e.g., 10% annual)
param target_vol = 0.10
signal vol_scalar = target_vol / (vol * sqrt(252))

// Scaled signal
signal sized_signal = momentum * vol_scalar

output sized_signal
```

**Why it works**: Keeps risk constant even when volatility changes.

### Kelly Criterion

Optimal sizing based on edge and odds:
```
f* = (p × b - q) / b
```

Where:
- p = probability of win
- q = 1 - p
- b = win/loss ratio

**In practice**: Use fractional Kelly (1/2 or 1/4) for safety.

**Example**:
- Win rate: 55%
- Win/loss ratio: 1.0
- Kelly: (0.55 × 1.0 - 0.45) / 1.0 = 10%
- Half Kelly: 5%

## Portfolio Construction

### Long-Short Construction

From signal to portfolio:

1. **Rank assets** by signal
2. **Go long** top N (or top decile)
3. **Go short** bottom N (or bottom decile)
4. **Weight** equally or by signal strength

```sig
// sigc handles this automatically from your signal output
signal momentum = prices / lag(prices, 60) - 1
output momentum

// Configuration for construction
// --construction long-short
// --top-n 20
// --bottom-n 20
```

### Sector Neutrality

Eliminate sector bets:

```sig
// Demean within sectors
signal sector_neutral = momentum - sector_mean(momentum)
```

**Why**: Reduces unintended factor exposure.

### Factor Neutrality

Neutralize common factor exposures:

```sig
// Remove market beta
signal beta = rolling_beta(returns, market_returns, 60)
signal market_neutral = momentum - beta * market_signal

// Remove multiple factors
signal neutral = momentum - beta_mkt * market - beta_smb * smb - beta_hml * hml
```

### Optimization

Mean-variance optimization:
```
max: w'μ - (λ/2) w'Σw
subject to: constraints
```

**Constraints** to consider:
- Long-only or allow shorts
- Max position size
- Sector limits
- Turnover limits

**Warning**: Optimizers are sensitive to inputs. Small changes in expected returns → large changes in weights.

## Constraints

### Position Limits

```bash
sigc run strategy.sig --max-position 0.05  # Max 5% per position
```

**Why**: Prevent concentration risk.

### Sector/Industry Limits

```bash
sigc run strategy.sig --max-sector 0.30  # Max 30% in any sector
```

**Why**: Prevent sector concentration.

### Turnover Limits

```bash
sigc run strategy.sig --max-turnover 2.0  # Max 200% annual turnover
```

**Why**: Control transaction costs.

### Liquidity Constraints

```bash
sigc run strategy.sig --min-adv 1000000  # Min $1M daily volume
```

**Why**: Ensure tradability.

### Short-Selling Constraints

```bash
sigc run strategy.sig --long-only  # No shorting
```

Some accounts can't short or have borrowing constraints.

## Drawdown Management

Drawdowns destroy capital and psychology. Managing them is crucial.

### Maximum Drawdown Limits

```bash
sigc run strategy.sig --max-drawdown 0.15  # Stop at 15% drawdown
```

### Drawdown Reduction Strategies

**1. Volatility Scaling**:
Reduce exposure when volatility rises:
```sig
signal vol_scale = min(target_vol / realized_vol, 1.0)
signal adjusted = raw_signal * vol_scale
```

**2. Trend Following**:
Reduce exposure when in drawdown:
```sig
signal equity_trend = portfolio_value / sma(portfolio_value, 20)
signal scale = if(equity_trend < 1.0, equity_trend, 1.0)
```

**3. Stop-Loss**:
Exit positions at loss threshold:
```sig
// Exit if position loses 5%
signal stop_loss = position_return < -0.05
```

### Recovery Time

Time to recover from drawdown:
```
Recovery Time = -Drawdown / Average Return
```

Example:
- 20% drawdown, 10% annual return → 2 years to recover

This is why limiting drawdowns is critical.

## Tail Risk Management

### Value at Risk (VaR)

Loss that won't be exceeded with X% confidence.

```
VaR_95 = -μ + 1.645σ  (assuming normal)
```

**Problem**: Assumes normality, underestimates tail risk.

### Expected Shortfall (CVaR)

Average loss when loss exceeds VaR.

```
CVaR_95 = E[Loss | Loss > VaR_95]
```

Better than VaR for fat-tailed distributions.

### Stress Testing

Test against historical scenarios:
- 2008 Financial Crisis
- 2020 COVID Crash
- 2022 Rate Shock

```bash
sigc run strategy.sig --stress-test 2008
```

### Hedging Strategies

**1. Options**:
Buy puts for downside protection.

**2. VIX exposure**:
Long volatility as crash hedge.

**3. Trend following overlay**:
CTA-style strategy as tail hedge.

**4. Risk-off triggers**:
Reduce exposure on specific signals (VIX spike, credit spread widening).

## Risk Monitoring

### Daily Monitoring

Check these daily:
- Portfolio P&L
- Position-level P&L
- Risk metrics (VaR, volatility)
- Factor exposures
- Concentration

### Alerting

Set up alerts for:
```bash
sigc alerts config \
  --drawdown-warning 0.05 \
  --drawdown-critical 0.10 \
  --vol-warning 1.5 \       # 1.5x normal vol
  --concentration-warning 0.10
```

### Risk Dashboards

Track over time:
- Rolling Sharpe
- Rolling volatility
- Factor exposures
- Correlation to market

## Practical Implementation

### Complete Risk-Managed Strategy

```sig
// risk_managed.sig - Momentum with risk controls

data prices = load("prices.csv")
data market = load("market.csv")

// Calculate returns
signal returns = prices / lag(prices, 1) - 1
signal mkt_returns = market / lag(market, 1) - 1

// Volatility
signal vol = std(returns, 20)
signal mkt_vol = std(mkt_returns, 20)

// Raw momentum signal
signal raw_momentum = prices / lag(prices, 60) - 1

// Step 1: Volatility normalize
signal vol_norm = raw_momentum / vol

// Step 2: Cross-sectional standardize
signal zscore = cs_zscore(vol_norm)

// Step 3: Market neutralize
signal beta = rolling_beta(returns, mkt_returns, 60)
signal market_neutral = zscore - beta * cs_mean(zscore)

// Step 4: Volatility target
param target_vol = 0.10
signal portfolio_vol = std(zscore, 20) * sqrt(252)
signal vol_scale = target_vol / portfolio_vol

// Final signal
signal final_signal = market_neutral * min(vol_scale, 2.0)  // Cap scale at 2x

output final_signal
```

### Risk Configuration

```bash
sigc run risk_managed.sig \
  --start 2015-01-01 \
  --end 2023-12-31 \
  --max-position 0.03 \
  --max-sector 0.25 \
  --max-turnover 4.0 \
  --max-drawdown 0.15 \
  --transaction-cost 0.001
```

## Common Mistakes

### 1. Ignoring Correlation

Two 10% positions with 0.8 correlation ≈ one 18% position.

**Solution**: Consider correlation when sizing.

### 2. Fighting the Market

Doubling down on losing positions.

**Solution**: Respect stop-losses. Let winners run, cut losers.

### 3. Over-Leverage

Leverage amplifies losses too.

**Solution**: Start with 1x. Add leverage slowly.

### 4. Model Overconfidence

Trusting backtests too much.

**Solution**: Add safety margins. Use fractional Kelly.

### 5. Ignoring Tail Events

Rare events happen more than models suggest.

**Solution**: Stress test. Have hedges. Keep some cash.

## Risk Budgeting

### Capital Allocation

Divide capital among strategies by risk contribution:

| Strategy | Expected Sharpe | Vol Target | Capital |
|----------|----------------|------------|---------|
| Momentum | 1.0 | 10% | 50% |
| Value | 0.8 | 8% | 30% |
| Quality | 0.6 | 6% | 20% |

### Risk Parity Across Strategies

Equal risk contribution from each strategy:
```
RiskContribution_i = w_i × σ_i × (Σw_j × cov_ij) / σ_p
```

## Key Takeaways

1. **Size matters**: Position sizing often matters more than signal selection
2. **Volatility target**: Keep risk constant as markets change
3. **Diversify risks**: Across positions, sectors, factors, strategies
4. **Limit drawdowns**: They're hard to recover from
5. **Expect tails**: Plan for events worse than history
6. **Monitor continuously**: Risk changes, stay vigilant

## Exercises

1. **Volatility targeting**: Implement a 10% vol target. How does it change results?

2. **Sector neutrality**: Compare sector-neutral vs unconstrained. What changes?

3. **Drawdown analysis**: Find the max drawdown in your backtest. How long to recover?

4. **Stress test**: Test your strategy in 2008 and 2020. Does it survive?

5. **Correlation impact**: If your top 2 positions have 0.7 correlation, what's the combined risk?

## Next Chapter

[Chapter 7: Going Live](07-production.md) - Take your strategy to production.
