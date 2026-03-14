# Data Quality

Ensuring your data is clean and reliable for backtesting.

## Common Data Issues

### Missing Data

- **Gaps**: Missing trading days
- **NaN values**: Missing prices or volumes
- **Incomplete symbols**: Assets that appear/disappear

### Data Errors

- **Spikes**: Erroneous price jumps
- **Zeros**: Invalid zero prices
- **Stale data**: Unchanged values when they should change

### Structural Issues

- **Survivorship bias**: Only successful companies in dataset
- **Look-ahead bias**: Using future information
- **Corporate actions**: Unadjusted splits and dividends

## Detecting Missing Data

### Check for NaN

```sig
signal check_missing:
  // Count missing values
  has_price = where(is_nan(prices), 0, 1)
  has_volume = where(is_nan(volume), 0, 1)

  // Quality score per asset
  quality = (has_price + has_volume) / 2

  emit quality
```

### Missing Data Report

```bash
sigc check data.parquet --missing
```

Output:

```
Missing Data Report:
===================
Column     | Missing | Pct Missing
-----------+---------+-----------
close      |    145  |    0.02%
volume     |    892  |    0.12%
high       |    145  |    0.02%
low        |    145  |    0.02%

Assets with >5% missing: DELISTED1, DELISTED2
```

## Handling Missing Data

### Fill with Zero

```sig
signal fill_zero:
  clean_prices = fill_nan(prices, 0)
  emit zscore(ret(clean_prices, 20))
```

### Forward Fill

```sig
signal forward_fill:
  // Use previous value
  filled = where(is_nan(prices), lag(prices, 1), prices)
  emit filled
```

### Exclude Missing

```sig
signal exclude_missing:
  valid = not(is_nan(prices))
  signal = where(valid, zscore(ret(prices, 20)), 0)
  emit signal
```

### Interpolation

```sig
signal interpolate:
  // Linear interpolation
  prev = lag(prices, 1)
  next = lead(prices, 1)
  interpolated = where(is_nan(prices), (prev + next) / 2, prices)
  emit interpolated
```

## Detecting Outliers

### Statistical Detection

```sig
signal outlier_detection:
  returns = ret(prices, 1)
  z = zscore(returns)

  // Flag returns > 5 standard deviations
  outlier = abs(z) > 5

  emit where(outlier, 1, 0)
```

### Price Spike Detection

```sig
signal spike_detection:
  returns = ret(prices, 1)

  // Large single-day moves
  spike = abs(returns) > 0.5  // > 50% move

  emit where(spike, 1, 0)
```

### Volume Anomalies

```sig
signal volume_anomaly:
  avg_volume = rolling_mean(volume, 20)
  ratio = volume / avg_volume

  // Volume > 10x average
  anomaly = ratio > 10

  emit where(anomaly, 1, 0)
```

## Handling Outliers

### Winsorization

```sig
signal winsorized:
  returns = ret(prices, 20)
  z = zscore(returns)

  // Clip at 1st and 99th percentile
  cleaned = winsor(z, p=0.01)

  emit cleaned
```

### Clipping

```sig
signal clipped:
  returns = ret(prices, 20)
  z = zscore(returns)

  // Hard clip at ±3 standard deviations
  bounded = clip(z, -3, 3)

  emit bounded
```

### Exclusion

```sig
signal exclude_outliers:
  returns = ret(prices, 20)
  z = zscore(returns)

  // Exclude extreme values
  valid = abs(z) < 5
  signal = where(valid, z, 0)

  emit signal
```

## Survivorship Bias

### The Problem

Backtests that only include currently active securities overstate returns because failed companies are excluded.

### Solution 1: Point-in-Time Data

Use a data provider that includes delisted securities:

```sig
data:
  source = "point_in_time_prices.parquet"
  format = parquet
  options:
    include_delisted = true
```

### Solution 2: Dead Stock Handling

```sig
signal handle_delisting:
  // Detect when a stock stops trading
  was_trading = lag(prices, 1) > 0
  not_trading = prices == 0 or is_nan(prices)
  delisted = was_trading and not_trading

  // Zero out signal for delisted stocks
  raw_signal = zscore(ret(prices, 60))
  signal = where(delisted, 0, raw_signal)

  emit signal
```

## Look-Ahead Bias

### The Problem

Using information that wouldn't have been available at the time of the trade.

### Common Sources

1. **Same-day data**: Using close price to make decisions before market close
2. **Reported vs actual dates**: Using data before it was actually reported
3. **Revisions**: Using revised data instead of original releases

### Solution: Lag Your Data

```sig
signal no_lookahead:
  // Use previous day's close for today's signal
  yesterday_price = lag(prices, 1)
  signal = zscore(ret(yesterday_price, 20))
  emit signal
```

### Fundamental Data

```sig
signal fundamental_signal:
  // Earnings reported with delay
  // Use 1-quarter lag to be safe
  lagged_earnings = lag(earnings, 63)  // ~3 months
  signal = zscore(lagged_earnings / prices)
  emit signal
```

## Data Validation Checklist

### Before Loading

- [ ] Check file format (CSV headers, Parquet schema)
- [ ] Verify date range is complete
- [ ] Confirm all expected symbols present
- [ ] Check for duplicate rows

### After Loading

- [ ] Count missing values per column
- [ ] Verify no future dates
- [ ] Check price ranges are reasonable
- [ ] Validate volume is non-negative

### Before Backtesting

- [ ] Confirm no look-ahead bias
- [ ] Verify corporate actions adjusted
- [ ] Check for survivorship bias
- [ ] Test on subset of data first

## Quality Metrics

### Completeness Score

```sig
signal completeness:
  // What percentage of data is present?
  has_data = where(is_nan(prices), 0, 1)
  score = rolling_mean(has_data, 252)  // Over 1 year
  emit score
```

### Price Quality

```sig
signal price_quality:
  // No zero or negative prices
  valid_price = prices > 0

  // No extreme daily returns
  ret_1d = ret(prices, 1)
  reasonable_return = abs(ret_1d) < 0.5

  quality = where(valid_price and reasonable_return, 1, 0)
  emit quality
```

### Volume Quality

```sig
signal volume_quality:
  // No zero volume on trading days
  has_volume = volume > 0

  // No extreme volume spikes
  avg_vol = rolling_mean(volume, 20)
  reasonable_vol = volume < 50 * avg_vol

  quality = where(has_volume and reasonable_vol, 1, 0)
  emit quality
```

## Automated Quality Checks

### sigc CLI

```bash
# Run all quality checks
sigc validate data.parquet

# Check specific issues
sigc validate data.parquet --check missing
sigc validate data.parquet --check outliers
sigc validate data.parquet --check duplicates
```

### In Strategy

```sig
data:
  source = "prices.parquet"
  format = parquet
  options:
    validate = true           # Enable validation
    max_missing_pct = 0.05    # Fail if >5% missing
    max_outlier_pct = 0.01    # Fail if >1% outliers
```

## Best Practices

### 1. Always Validate New Data

```bash
sigc validate new_data.parquet --strict
```

### 2. Use Quality Thresholds

```sig
signal with_quality_filter:
  // Only trade assets with good data
  quality = calculate_quality(prices, volume)
  raw_signal = zscore(ret(prices, 60))
  signal = where(quality > 0.95, raw_signal, 0)
  emit signal
```

### 3. Document Data Sources

```sig
// Data: Bloomberg adjusted prices
// Period: 2010-01-01 to 2024-12-31
// Universe: S&P 500 constituents (point-in-time)
// Corporate actions: Adjusted for splits and dividends
data:
  source = "sp500_prices.parquet"
```

### 4. Version Your Data

```
data/
├── v1/
│   └── prices.parquet
├── v2/
│   └── prices.parquet  # Fixed outliers
└── v3/
    └── prices.parquet  # Added delisted stocks
```

### 5. Keep Raw and Cleaned Data

```sig
// Load raw data
data raw:
  source = "prices_raw.parquet"

// Apply cleaning
signal clean:
  cleaned = winsor(zscore(raw.prices), p=0.01)
  emit cleaned
```

## Next Steps

- [Corporate Actions](corporate-actions.md) - Handling splits and dividends
- [CSV Format](csv.md) - Loading CSV files
- [Backtesting](../backtesting/index.md) - Running backtests
