# Transaction Cost Models

Model realistic trading costs for accurate backtests.

## Cost Components

| Component | Description | Typical Range |
|-----------|-------------|---------------|
| Commission | Broker fees | 0.5-5 bps |
| Slippage | Bid-ask spread | 1-10 bps |
| Market Impact | Price movement from trading | Varies |
| Borrow Cost | Short selling fees | 0.3-2% annual |

## Built-in Models

### Zero Cost (Testing)

```rust
use sig_runtime::CostModel;

let model = CostModel::zero();
```

### Institutional

Low-cost for large AUM:

```rust
let model = CostModel::institutional();
// commission: 0.5 bps
// slippage: 1 bp
// impact: sqrt model, 0.05 coefficient
// borrow: 0.3% annual
```

### Retail

Higher costs for smaller accounts:

```rust
let model = CostModel::retail();
// commission: 5 bps
// slippage: 5 bps
// impact: linear, 0.5 coefficient
// borrow: 2% annual
```

## Custom Models

```rust
use sig_runtime::{CostModel, ImpactModel};

let model = CostModel::new()
    .with_commission(1.0)      // 1 bp
    .with_slippage(2.0)        // 2 bps
    .with_borrow_cost(0.005)   // 0.5% annual
    .with_impact(ImpactModel::SquareRoot { coefficient: 0.1 });
```

## Impact Models

### None

No market impact:
```rust
ImpactModel::None
```

### Linear

Cost proportional to participation:
```rust
ImpactModel::Linear { coefficient: 0.1 }
// impact = coefficient * (trade_size / ADV)
```

### Square Root

Diminishing marginal impact:
```rust
ImpactModel::SquareRoot { coefficient: 0.1 }
// impact = coefficient * sqrt(trade_size / ADV)
```

### Almgren-Chriss

Academic model with temporary and permanent impact:
```rust
ImpactModel::AlmgrenChriss { eta: 0.1, gamma: 0.05 }
```

## Calculating Costs

```rust
let model = CostModel::institutional();

// Calculate cost for a trade
let cost = model.calculate_cost(
    100_000.0,           // notional
    Some(1_000_000.0),   // average daily volume
    false,               // is_short
    21.0                 // holding period (days)
);

println!("Total cost: ${:.2}", cost.total);
println!("Commission: ${:.2}", cost.commission);
println!("Slippage: ${:.2}", cost.slippage);
println!("Impact: ${:.2}", cost.impact);
```

## Portfolio-Level Costs

```rust
use sig_runtime::PortfolioCostCalculator;

let calculator = PortfolioCostCalculator::new(CostModel::institutional());

let trades = vec![
    ("AAPL".to_string(), 50_000.0, Some(500_000_000.0), false),
    ("MSFT".to_string(), -30_000.0, Some(400_000_000.0), true),
];

let cost = calculator.calculate_rebalance_cost(&trades, 21.0);
println!("Total cost: ${:.2}", cost.total_cost);
println!("Average: {:.2} bps", cost.avg_cost_bps);
```

## Impact on Strategy

High turnover strategies are more sensitive to costs:

| Turnover | Cost Impact |
|----------|-------------|
| 100% | ~5-10 bps annual |
| 500% | ~25-50 bps annual |
| 1000% | ~50-100 bps annual |

A strategy with 1.0 Sharpe gross might have 0.7 Sharpe net of costs.
