# Portfolio Section

The `portfolio` section constructs portfolios from signals and runs backtests.

## Syntax

```sig
portfolio <name>:
  weights = <weight_expression>
  [costs = <cost_expression>]
  backtest [options] from <date> to <date>
```

## Basic Usage

```sig
signal momentum:
  emit zscore(ret(prices, 20))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Weight Construction

### Long-Short Portfolio

The most common construction: long winners, short losers.

```sig
portfolio long_short:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

Parameters:

- `top`: Fraction of assets to long (default: 0.2)
- `bottom`: Fraction of assets to short (default: 0.2)
- `cap`: Maximum position size (optional)

### With Position Cap

Limit individual position sizes:

```sig
portfolio capped:
  weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
  backtest from 2024-01-01 to 2024-12-31
```

With `cap=0.05`, no position exceeds 5% of portfolio.

### Asymmetric Long-Short

Different top/bottom percentages:

```sig
portfolio asymmetric:
  // More longs than shorts
  weights = rank(signal).long_short(top=0.3, bottom=0.1)
  backtest from 2024-01-01 to 2024-12-31
```

## Backtest Options

### Date Range

Required: specify start and end dates.

```sig
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

### Rebalancing Frequency

Default is daily. Specify different frequencies:

```sig
// Weekly (every 5 trading days)
portfolio weekly:
  weights = ...
  backtest rebal=5 from 2024-01-01 to 2024-12-31

// Monthly (every 21 trading days)
portfolio monthly:
  weights = ...
  backtest rebal=21 from 2024-01-01 to 2024-12-31

// Quarterly (every 63 trading days)
portfolio quarterly:
  weights = ...
  backtest rebal=63 from 2024-01-01 to 2024-12-31
```

### Benchmark

Specify a benchmark for relative metrics:

```sig
portfolio hedged:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest benchmark=SPY from 2024-01-01 to 2024-12-31
```

Enables calculation of:

- Alpha
- Beta
- Information Ratio
- Tracking Error

## Transaction Costs

### Basic Commission

```sig
portfolio with_commission:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(5)  // 5 basis points per trade
  backtest from 2024-01-01 to 2024-12-31
```

### Commission + Slippage

```sig
portfolio institutional:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(5) + slippage.model("linear", coef=0.5)
  backtest from 2024-01-01 to 2024-12-31
```

### Advanced Cost Model

```sig
portfolio realistic:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  costs = tc.bps(3) + slippage.model("square-root", coef=0.1)
  backtest from 2024-01-01 to 2024-12-31
```

Slippage models:

- `"linear"`: Cost proportional to trade size
- `"square-root"`: Cost proportional to sqrt(size)
- `"almgren-chriss"`: Full market impact model

See [Transaction Costs](../backtesting/cost-models.md) for details.

## Multiple Portfolios

Define multiple portfolios for comparison:

```sig
signal momentum:
  emit zscore(ret(prices, 60))

// Different top/bottom percentages
portfolio conservative:
  weights = rank(momentum).long_short(top=0.1, bottom=0.1)
  backtest from 2024-01-01 to 2024-12-31

portfolio aggressive:
  weights = rank(momentum).long_short(top=0.3, bottom=0.3)
  backtest from 2024-01-01 to 2024-12-31

// Different rebalancing
portfolio daily:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31

portfolio weekly:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest rebal=5 from 2024-01-01 to 2024-12-31
```

Compare with:

```bash
sigc diff conservative.sig aggressive.sig
```

## Weight Expressions

### Using Rank

```sig
portfolio ranked:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
```

`rank()` converts scores to uniform ranks (0 to 1), making portfolios robust to outliers.

### Using Signal Directly

```sig
portfolio signal_weighted:
  // Weights proportional to signal strength
  // Be careful with outliers!
  scaled = scale(abs(signal))
  weights = where(signal > 0, scaled, -scaled)
```

### Combined Signals

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal reversal:
  emit -zscore(ret(prices, 5))

portfolio combined:
  combined_score = 0.7 * momentum + 0.3 * reversal
  weights = rank(combined_score).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-12-31
```

## Backtest Results

Running a backtest produces:

```
=== Backtest Results ===
Total Return:         45.23%
Annualized Return:    10.12%
Sharpe Ratio:          1.23
Max Drawdown:         12.45%
Turnover:            285.00%
```

### With Benchmark

```
=== Backtest Results ===
Total Return:         45.23%
Annualized Return:    10.12%
Sharpe Ratio:          1.23
Max Drawdown:         12.45%
Turnover:            285.00%

--- Benchmark-Relative ---
Alpha:                 5.67%
Beta:                  0.35
Information Ratio:     1.45
Tracking Error:        8.20%
```

### Export Results

```bash
sigc run strategy.sig --output results.json
```

## Examples

### Basic Momentum

```sig
data:
  prices: load csv from "data/prices.csv"

params:
  lookback = 60
  top_pct = 0.2

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio main:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)
  backtest from 2020-01-01 to 2024-12-31
```

### With All Options

```sig
data:
  prices: load parquet from "s3://bucket/prices.parquet"
  sectors: load csv from "data/sectors.csv" dtype=category

params:
  lookback = 60
  vol_window = 252
  top_pct = 0.2
  max_weight = 0.05
  commission_bps = 5

signal sector_neutral_momentum:
  raw = zscore(ret(prices, lookback))
  neutral = neutralize(raw, by=sectors)
  vol = rolling_std(ret(prices, 1), vol_window)
  vol_adj = neutral / vol
  emit winsor(zscore(vol_adj), 0.01)

portfolio institutional:
  weights = rank(sector_neutral_momentum).long_short(
    top=top_pct,
    bottom=top_pct,
    cap=max_weight
  )
  costs = tc.bps(commission_bps) + slippage.model("square-root", coef=0.1)
  backtest rebal=21 benchmark=SPY from 2020-01-01 to 2024-12-31
```

### Multi-Strategy Portfolio

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal quality:
  emit zscore(roe)

// Strategy 1: Momentum only
portfolio momentum_only:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31

// Strategy 2: Value only
portfolio value_only:
  weights = rank(value).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31

// Strategy 3: Combined
portfolio multi_factor:
  combined = 0.4 * momentum + 0.4 * value + 0.2 * quality
  weights = rank(combined).long_short(top=0.2, bottom=0.2)
  backtest from 2020-01-01 to 2024-12-31
```

## Best Practices

### 1. Use Ranking

```sig
// Good: Robust to outliers
weights = rank(signal).long_short(top=0.2, bottom=0.2)

// Risky: Signal outliers affect weights
weights = signal.long_short(top=0.2, bottom=0.2)
```

### 2. Set Position Caps

```sig
// Prevent concentration
weights = rank(signal).long_short(top=0.2, bottom=0.2, cap=0.05)
```

### 3. Include Transaction Costs

```sig
// Realistic performance estimate
costs = tc.bps(5) + slippage.model("square-root", coef=0.1)
```

### 4. Match Rebalancing to Signal

- Fast signals → frequent rebalancing
- Slow signals → infrequent rebalancing

```sig
// Fast momentum: rebalance more frequently
portfolio fast:
  weights = rank(fast_signal).long_short(top=0.2, bottom=0.2)
  backtest rebal=5 from 2024-01-01 to 2024-12-31

// Slow value: rebalance less frequently
portfolio slow:
  weights = rank(slow_signal).long_short(top=0.2, bottom=0.2)
  backtest rebal=21 from 2024-01-01 to 2024-12-31
```

### 5. Use Benchmark for Context

```sig
backtest benchmark=SPY from 2024-01-01 to 2024-12-31
```

## Next Steps

- [Backtesting](../backtesting/index.md) - Detailed backtest guide
- [Transaction Costs](../backtesting/cost-models.md) - Cost modeling
- [Constraints](../backtesting/constraints.md) - Position limits
