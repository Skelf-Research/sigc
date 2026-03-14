# Cost Models

Model transaction costs realistically for accurate backtests.

## Why Costs Matter

Transaction costs can significantly impact returns:

| Annual Turnover | Cost (10 bps) | Cost (25 bps) |
|-----------------|---------------|---------------|
| 100% | 0.1% | 0.25% |
| 250% | 0.25% | 0.625% |
| 500% | 0.5% | 1.25% |
| 1000% | 1.0% | 2.5% |

A strategy with 15% gross returns and 500% turnover might only deliver 13.75% net returns.

## Basic Cost Model

### Fixed Basis Points

```sig
portfolio main:
  weights = ...
  costs = tc.bps(5)  # 5 basis points per trade
  backtest from 2020-01-01 to 2024-12-31
```

This applies 0.05% cost to each dollar traded.

### Example

```
Trade: Sell 2% of AAPL, Buy 2% of MSFT
Total traded: 4% of portfolio
Cost: 4% × 0.05% = 0.002% of portfolio
```

## Commission Models

### Per-Share Commission

```sig
portfolio main:
  weights = ...
  costs = tc.per_share(0.005)  # $0.005 per share
  backtest ...
```

### Per-Trade Commission

```sig
portfolio main:
  weights = ...
  costs = tc.per_trade(5.00)  # $5 per trade
  backtest ...
```

### Tiered Commission

```sig
portfolio main:
  weights = ...
  costs = tc.tiered(
    tiers = [
      { volume = 1000000, rate = 0.0035 },
      { volume = 10000000, rate = 0.0020 },
      { default_rate = 0.0010 }
    ]
  )
  backtest ...
```

## Slippage Models

Slippage captures market impact and execution quality.

### Fixed Slippage

```sig
portfolio main:
  weights = ...
  costs = slippage.bps(5)  # 5 bps slippage
  backtest ...
```

### Volume-Based Slippage

Cost increases with trade size relative to volume:

```sig
portfolio main:
  weights = ...
  costs = slippage.volume_pct(coef=0.1)
  // Slippage = 0.1 × (trade_value / daily_volume)
  backtest ...
```

### Square-Root Model

Standard market impact model:

$$\text{Impact} = \sigma \cdot \text{coef} \cdot \sqrt{\frac{\text{trade\_size}}{\text{ADV}}}$$

```sig
portfolio main:
  weights = ...
  costs = slippage.model("square-root", coef=0.1)
  backtest ...
```

### Linear Model

```sig
portfolio main:
  weights = ...
  costs = slippage.model("linear", coef=0.05)
  backtest ...
```

## Combined Costs

Combine multiple cost components:

```sig
portfolio main:
  weights = ...
  costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
  backtest ...
```

### Full Cost Model

```sig
portfolio main:
  weights = ...
  costs = (
    tc.bps(3) +                           # Bid-ask spread
    tc.per_share(0.001) +                 # Broker commission
    slippage.model("square-root", coef=0.05) +  # Market impact
    tc.bps(1)                             # Exchange fees
  )
  backtest ...
```

## Asymmetric Costs

Different costs for buying vs selling:

```sig
portfolio main:
  weights = ...
  costs = tc.asymmetric(
    buy = tc.bps(5),
    sell = tc.bps(7)  # Short-selling costs more
  )
  backtest ...
```

## Short-Selling Costs

### Borrow Cost

```sig
portfolio main:
  weights = ...
  costs = tc.bps(5) + borrow.rate(0.005)  # 0.5% annual borrow rate
  backtest ...
```

### Hard-to-Borrow

```sig
portfolio main:
  weights = ...
  costs = tc.bps(5) + borrow.htb(
    default_rate = 0.005,
    htb_rate = 0.20,     # 20% for hard-to-borrow
    htb_list = "htb_stocks.csv"
  )
  backtest ...
```

## Realistic Cost Estimates

### Institutional Trading

| Component | Large Cap | Small Cap |
|-----------|-----------|-----------|
| Bid-Ask Spread | 1-3 bps | 5-15 bps |
| Commission | 1-2 bps | 1-2 bps |
| Market Impact | 2-10 bps | 10-50 bps |
| **Total** | **5-15 bps** | **20-70 bps** |

### Retail Trading

| Component | Commission-Free | Traditional |
|-----------|-----------------|-------------|
| Bid-Ask Spread | 2-5 bps | 2-5 bps |
| Commission | 0 bps | 5-10 bps |
| Payment for Order Flow | 1-3 bps | 0 bps |
| **Total** | **3-8 bps** | **7-15 bps** |

## Cost-Aware Strategy Design

### Check Breakeven Turnover

```bash
sigc analyze strategy.sig --breakeven-turnover --cost-bps 10
```

Output:

```
Gross Return: 15.2%
Cost per 100% Turnover: 0.2%
Breakeven Turnover: 7600%
Current Turnover: 250%
Net Return: 14.7%
```

### Turnover Reduction

Add turnover penalty:

```sig
portfolio low_turnover:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)

  // Don't trade small changes
  constraints:
    min_trade = 0.01  # Don't trade if change < 1%

  backtest ...
```

### Rebalancing Frequency

Less frequent rebalancing reduces costs:

```sig
// High turnover
portfolio daily:
  backtest rebal=1 ...  // High costs

// Lower turnover
portfolio monthly:
  backtest rebal=21 ...  // Lower costs
```

## Sensitivity Analysis

Test different cost assumptions:

```sig
portfolio low_cost:
  costs = tc.bps(5)
  backtest ...

portfolio mid_cost:
  costs = tc.bps(15)
  backtest ...

portfolio high_cost:
  costs = tc.bps(30)
  backtest ...
```

### CLI Analysis

```bash
sigc run strategy.sig --cost-sensitivity 5,10,15,20,25,30
```

Output:

```
Cost Sensitivity Analysis:
Cost (bps) | Net Return | Sharpe | Breakeven
-----------+------------+--------+----------
5          | 14.7%      | 0.95   | Yes
10         | 14.2%      | 0.92   | Yes
15         | 13.7%      | 0.89   | Yes
20         | 13.2%      | 0.86   | Yes
25         | 12.7%      | 0.82   | Yes
30         | 12.2%      | 0.79   | Yes
```

## Best Practices

### 1. Be Conservative

```sig
// Assume higher costs than expected
costs = tc.bps(15)  // Not tc.bps(5)
```

### 2. Include All Components

```sig
// Don't just use commission
costs = (
  tc.bps(3) +           // Spread
  tc.per_share(0.001) + // Commission
  slippage.model("square-root", coef=0.1) +  // Impact
  borrow.rate(0.005)    // Short costs
)
```

### 3. Test Sensitivity

```sig
// Strategy should work across reasonable cost assumptions
portfolio conservative:
  costs = tc.bps(20)  // Double expected costs
  backtest ...
```

### 4. Consider Capacity

Large positions have higher impact:

```sig
// Add position cap to manage impact
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.03)
```

### 5. Account for Liquidity

```sig
// Higher costs for illiquid stocks
costs = slippage.model("square-root",
  coef=0.1,
  volume_threshold=1000000  // Higher impact for low-volume stocks
)
```

## Example: Full Cost Model

```sig
data:
  source = "prices.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric

signal momentum:
  emit zscore(ret(prices, 60))

portfolio realistic:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2, cap=0.03)

  // Realistic institutional cost model
  costs = (
    tc.bps(2) +                              // Half spread
    tc.per_share(0.001) +                    // Commission
    slippage.model("square-root", coef=0.1) +  // Market impact
    borrow.rate(0.003)                       // Borrow cost
  )

  // Turnover control
  constraints:
    min_trade = 0.005  // Don't trade < 0.5% changes

  backtest rebal=21 from 2020-01-01 to 2024-12-31
```

## Next Steps

- [Metrics](metrics.md) - Understanding return metrics
- [Walk-Forward](walk-forward.md) - Out-of-sample testing
- [Constraints](constraints.md) - Portfolio constraints
