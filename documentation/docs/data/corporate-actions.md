# Corporate Actions

Handling stock splits, dividends, and other corporate actions in your data.

## Overview

Corporate actions affect price and volume data:

| Action | Effect on Price | Effect on Volume |
|--------|-----------------|------------------|
| Stock Split | Divided | Multiplied |
| Reverse Split | Multiplied | Divided |
| Dividend | Reduced (ex-date) | No change |
| Spin-off | Reduced | No change |
| Merger | Converted/Delisted | Varies |

## Adjusted vs Unadjusted Prices

### Unadjusted (Raw) Prices

What actually traded on each day:

```
Date       | AAPL Price
2014-06-06 | $645.57    <- Before 7:1 split
2014-06-09 | $93.70     <- After split
```

### Adjusted Prices

Retroactively adjusted for corporate actions:

```
Date       | AAPL Adjusted Price
2014-06-06 | $92.22     <- Adjusted back
2014-06-09 | $93.70     <- Same
```

**Always use adjusted prices for backtesting.**

## Using Adjusted Prices

### Data Provider

Most data providers offer adjusted prices:

```sig
data:
  source = "adjusted_prices.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    adj_close: Numeric as prices  # Use adjusted close
    volume: Numeric
```

### Manual Adjustment

If you have split data:

```sig
signal adjust_for_splits:
  // Cumulative adjustment factor
  adj_factor = cumprod(split_ratio)

  // Adjust prices backward
  adjusted_price = prices / adj_factor

  emit adjusted_price
```

## Stock Splits

### Detection

Detect splits in unadjusted data:

```sig
signal detect_splits:
  daily_return = ret(prices, 1)

  // Large drop (potential split)
  potential_split = daily_return < -0.4

  // Check if round ratio
  ratio = lag(prices, 1) / prices
  is_round = abs(round(ratio) - ratio) < 0.01

  split_detected = potential_split and is_round

  emit where(split_detected, ratio, 1)
```

### Common Split Ratios

| Ratio | Description |
|-------|-------------|
| 2:1 | Price halves |
| 3:1 | Price divides by 3 |
| 4:1 | Price divides by 4 |
| 7:1 | Apple 2014 split |
| 10:1 | NVIDIA 2024 split |
| 20:1 | Google/Amazon 2022 splits |

### Reverse Splits

```sig
signal detect_reverse_split:
  daily_return = ret(prices, 1)

  // Large jump (potential reverse split)
  potential_reverse = daily_return > 1.0

  // Check if round ratio
  ratio = prices / lag(prices, 1)
  is_round = abs(round(ratio) - ratio) < 0.01

  reverse_detected = potential_reverse and is_round

  emit where(reverse_detected, ratio, 1)
```

## Dividends

### Dividend Adjustment

Prices drop by dividend amount on ex-date:

```sig
signal adjust_for_dividends:
  // Dividend yield on ex-date
  div_yield = dividend / lag(prices, 1)

  // Cumulative adjustment factor
  adj_factor = cumprod(1 - div_yield)

  // Adjust prices backward
  adjusted_price = prices / adj_factor

  emit adjusted_price
```

### Total Return

Include dividends in return calculation:

```sig
signal total_return:
  price_return = ret(prices, 1)
  div_return = dividend / lag(prices, 1)
  total = price_return + div_return
  emit total
```

## Spin-offs

When a company splits into two:

```sig
signal handle_spinoff:
  // Parent company price drops
  // Spinoff creates new security

  // Track combined value
  parent_value = parent_price * parent_shares
  spinoff_value = spinoff_price * spinoff_shares
  total_value = parent_value + spinoff_value

  emit total_value
```

## Mergers and Acquisitions

### Cash Acquisition

Target is acquired for cash:

```sig
signal handle_cash_acquisition:
  // Stock delists at acquisition price
  was_trading = lag(prices, 1) > 0
  not_trading = prices == 0 or is_nan(prices)
  acquired = was_trading and not_trading

  // Last price should be ~acquisition price
  emit where(acquired, 0, prices)
```

### Stock-for-Stock Merger

Target converted to acquirer shares:

```sig
signal handle_stock_merger:
  // Target shareholders receive exchange_ratio shares of acquirer
  target_value = target_price * exchange_ratio

  emit target_value
```

## Data Quality with Corporate Actions

### Validation

```sig
signal validate_corporate_actions:
  daily_return = ret(prices, 1)

  // Flag suspicious large moves
  large_move = abs(daily_return) > 0.3

  // Check if explained by corporate action
  has_action = corporate_action_flag > 0

  // Unexplained moves are data errors
  suspicious = large_move and not(has_action)

  emit where(suspicious, 1, 0)
```

### Volume Adjustment

When prices are adjusted, volume should be adjusted inversely:

```sig
// Adjusted volume (for split-adjusted prices)
adjusted_volume = volume * split_ratio
```

## Best Practices

### 1. Use Pre-Adjusted Data

Most data providers handle adjustments:

```sig
data:
  source = "bloomberg_adjusted.parquet"  # Pre-adjusted
  format = parquet
```

### 2. Verify Adjustments

```sig
signal verify_adjustments:
  daily_ret = ret(prices, 1)

  // No return > 100% (unadjusted splits)
  no_splits = abs(daily_ret) < 1.0

  // No returns exactly -50%, -67%, etc. (unhandled splits)
  common_splits = [-0.5, -0.667, -0.75, -0.8, -0.857, -0.9, -0.95]
  // Check distance from common split returns
  not_near_split = min(abs(daily_ret - common_splits)) > 0.01

  emit where(no_splits and not_near_split, 1, 0)
```

### 3. Track Adjustment Factors

Keep adjustment factors for audit:

```sig
data:
  source = "prices.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as raw_prices
    adj_close: Numeric as prices
    adj_factor: Numeric  # For debugging
```

### 4. Handle Delistings

```sig
signal handle_delisting:
  // Zero out signals for delisted stocks
  is_trading = prices > 0 and not(is_nan(prices))
  signal = where(is_trading, raw_signal, 0)
  emit signal
```

### 5. Document Your Adjustments

```sig
// Data adjustments:
// - Split adjusted back to 2000
// - Dividend adjusted (total return)
// - Delisted stocks included with final prices
// - Adjusted volume for splits
data:
  source = "fully_adjusted.parquet"
```

## Common Issues

### Missing Split Adjustment

```
Symptom: -50%, -67% returns on split dates
Solution: Use adjusted prices or apply split factors
```

### Missing Dividend Adjustment

```
Symptom: Small drops on ex-dividend dates
Solution: Use dividend-adjusted prices or total return prices
```

### Inconsistent Adjustment

```
Symptom: Some prices adjusted, others not
Solution: Verify entire dataset uses same adjustment method
```

### Volume Not Adjusted

```
Symptom: Volume spikes/drops on split dates
Solution: Adjust volume inversely to price adjustment
```

## Adjustment Timeline

```
Original Data:
2024-01-15: $200 (before 2:1 split)
2024-01-16: $100 (after split)

Adjusted Data (looking back):
2024-01-15: $100 (adjusted)
2024-01-16: $100 (actual)

When loading in 2024:
- All historical prices adjusted to current shares outstanding
- Returns are meaningful across corporate actions
```

## Next Steps

- [Data Quality](data-quality.md) - General data validation
- [Backtesting](../backtesting/index.md) - Using clean data for backtests
- [CSV Format](csv.md) - Loading data files
