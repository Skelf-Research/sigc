# Portfolio Constraints

Apply realistic constraints to portfolio weights.

## Why Constraints?

Unconstrained portfolios can have:

- Extreme positions (50%+ in one stock)
- Huge sector concentrations
- Excessive leverage
- Impractical trade sizes

Constraints make backtests realistic.

## Position Constraints

### Maximum Position Size

```sig
portfolio constrained:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    max_position = 0.05  # No position > 5%

  backtest from 2020-01-01 to 2024-12-31
```

### Minimum Position Size

```sig
constraints:
  min_position = 0.005  # No position < 0.5%
```

Small positions add costs without meaningful impact.

### Position Cap in long_short

```sig
// Built-in position cap
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
```

## Exposure Constraints

### Gross Exposure

Total long + short exposure:

```sig
constraints:
  max_gross = 2.0   # Max 200% gross (100% long + 100% short)
  min_gross = 1.5   # Min 150% gross
```

### Net Exposure

Long minus short exposure:

```sig
constraints:
  max_net = 0.1    # Max 10% net long
  min_net = -0.1   # Max 10% net short
```

### Dollar Neutral

Enforce exact dollar neutrality:

```sig
constraints:
  dollar_neutral = true  # Sum of weights = 0
```

## Sector Constraints

### Maximum Sector Weight

```sig
constraints:
  max_sector = 0.25  # No sector > 25% of portfolio
```

### Sector Neutrality

```sig
constraints:
  sector_neutral = true  # Each sector sums to 0
```

### Relative Sector Bounds

```sig
constraints:
  sector_deviation = 0.05  # Within 5% of benchmark sector weights
```

## Turnover Constraints

### Maximum Turnover

```sig
constraints:
  max_turnover = 0.25  # Max 25% turnover per rebalance
```

### Minimum Trade Size

```sig
constraints:
  min_trade = 0.01  # Don't trade if change < 1%
```

This prevents excessive small trades.

## Leverage Constraints

### Maximum Leverage

```sig
constraints:
  max_leverage = 2.0  # Max 2x leverage
```

### Long/Short Limits

```sig
constraints:
  max_long = 1.0   # Max 100% long
  max_short = 1.0  # Max 100% short
```

## Liquidity Constraints

### Minimum Volume

```sig
constraints:
  min_adv = 1000000  # Min $1M average daily volume
```

### Volume Participation

```sig
constraints:
  max_participation = 0.1  # Don't exceed 10% of daily volume
```

### Market Cap Filter

```sig
constraints:
  min_market_cap = 1000000000  # Min $1B market cap
```

## Risk Constraints

### Tracking Error

```sig
constraints:
  max_tracking_error = 0.05  # Max 5% tracking error vs benchmark
```

### Beta

```sig
constraints:
  max_beta = 1.2
  min_beta = 0.8
  # Portfolio beta between 0.8 and 1.2
```

### Volatility Target

```sig
constraints:
  target_volatility = 0.15  # Target 15% annual vol
```

## Constraint Syntax

### Full Constraints Block

```sig
portfolio constrained:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    // Position limits
    max_position = 0.05
    min_position = 0.005

    // Exposure limits
    max_gross = 2.0
    dollar_neutral = true

    // Sector limits
    max_sector = 0.25
    sector_neutral = true

    // Trading limits
    max_turnover = 0.25
    min_trade = 0.01

    // Liquidity
    min_adv = 1000000

  backtest from 2020-01-01 to 2024-12-31
```

## Constraint Priority

When constraints conflict, sigc applies them in order:

1. **Liquidity** - Filter out illiquid stocks
2. **Position limits** - Cap individual positions
3. **Sector limits** - Adjust sector exposures
4. **Exposure limits** - Adjust gross/net exposure
5. **Turnover limits** - Reduce trades if needed

## Soft vs Hard Constraints

### Hard Constraints (Strict)

Must be satisfied exactly:

```sig
constraints:
  max_position = 0.05  # Hard limit
```

### Soft Constraints (Penalties)

Preferred but can be violated:

```sig
constraints:
  soft:
    target_turnover = 0.15
    turnover_penalty = 0.01  # 1% penalty per 1% excess turnover
```

## Dynamic Constraints

Constraints that vary over time:

```sig
signal vol_regime:
  vol = rolling_std(ret(prices, 1), 60)
  high_vol = vol > quantile(vol, 0.8)
  emit high_vol

portfolio adaptive:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    // Reduce exposure in high volatility
    max_gross = where(vol_regime, 1.5, 2.0)

  backtest from 2020-01-01 to 2024-12-31
```

## Constraint Violations Report

Check constraint adherence:

```bash
sigc run strategy.sig --report constraints
```

Output:

```
Constraint Violations Report:
=============================

max_position (5%):
  Violations: 0
  Max observed: 4.8%

max_sector (25%):
  Violations: 3
  Dates: 2021-03-15 (Tech: 27%), 2021-04-12 (Tech: 26%), 2022-01-03 (Energy: 26%)

max_turnover (25%):
  Violations: 12
  Average excess: 3.2%
  Max excess: 8.5%

dollar_neutral:
  Max deviation: 0.2%
  Average deviation: 0.05%
```

## Examples

### Conservative Institutional

```sig
portfolio institutional:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  constraints:
    max_position = 0.03      # Max 3% per name
    max_sector = 0.20        # Max 20% per sector
    max_gross = 1.5          # Max 150% gross
    dollar_neutral = true    # Market neutral
    min_adv = 5000000        # Min $5M ADV
    max_turnover = 0.20      # Max 20% monthly turnover

  costs = tc.bps(10)
  backtest rebal=21 from 2015-01-01 to 2024-12-31
```

### Aggressive Hedge Fund

```sig
portfolio hedge_fund:
  weights = rank(signal).long_short(top=0.1, bottom=0.1)

  constraints:
    max_position = 0.10      # Max 10% per name
    max_gross = 3.0          # Max 300% gross
    max_net = 0.3            # Max 30% directional
    max_sector = 0.30        # Max 30% per sector

  costs = tc.bps(5) + slippage.model("square-root", coef=0.05)
  backtest rebal=5 from 2015-01-01 to 2024-12-31
```

### Long-Only with Tracking

```sig
portfolio long_only:
  weights = rank(signal).long_short(top=0.3, bottom=0)  # Long only

  constraints:
    min_position = 0.001     # Min 0.1% (avoid tiny positions)
    max_position = 0.05      # Max 5%
    max_gross = 1.0          # Fully invested (100%)
    sector_deviation = 0.05  # Within 5% of benchmark sectors
    max_tracking_error = 0.03  # Max 3% tracking error

  backtest rebal=21 benchmark=SPY from 2015-01-01 to 2024-12-31
```

## Best Practices

### 1. Start Constrained

```sig
// Better to be conservative
constraints:
  max_position = 0.03  # Not 0.10
```

### 2. Match Real-World Limits

```sig
// If you trade a $100M portfolio:
constraints:
  max_participation = 0.05  # 5% of daily volume
  min_adv = 1000000         # $1M minimum
```

### 3. Include Turnover Limits

```sig
constraints:
  max_turnover = 0.20  # Realistic for monthly rebal
```

### 4. Test Without Constraints First

Compare constrained vs unconstrained to see impact.

### 5. Document Your Constraints

```sig
// Constraints rationale:
// - max_position: Risk limit per name
// - max_sector: Diversification requirement
// - dollar_neutral: Market neutral mandate
// - min_adv: Liquidity requirement for $50M AUM
```

## Next Steps

- [Benchmark Analysis](benchmark-analysis.md) - Benchmark comparison
- [Cost Models](cost-models.md) - Transaction costs
- [Walk-Forward](walk-forward.md) - Validation with constraints
