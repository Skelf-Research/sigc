# CSV Format

Loading data from CSV files.

## Basic Usage

```sig
data:
  source = "prices.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric
```

## Expected File Format

### Long Format (Recommended)

One row per date/symbol combination:

```csv
date,ticker,close,volume
2024-01-02,AAPL,185.64,45678900
2024-01-02,MSFT,374.51,23456789
2024-01-02,GOOGL,140.21,12345678
2024-01-03,AAPL,184.25,43210987
2024-01-03,MSFT,372.89,21098765
2024-01-03,GOOGL,139.75,11234567
```

### Wide Format

One row per date, symbols as columns:

```csv
date,AAPL,MSFT,GOOGL
2024-01-02,185.64,374.51,140.21
2024-01-03,184.25,372.89,139.75
```

Load with pivot option:

```sig
data:
  source = "prices_wide.csv"
  format = csv
  options:
    pivot = true
    date_column = "date"
```

## Column Definitions

### Required Columns

```sig
columns:
  date: Date      # Index column
  ticker: Symbol  # Asset identifier
```

### Data Columns

```sig
columns:
  close: Numeric as prices    # Closing price
  volume: Numeric             # Trading volume
  high: Numeric               # Daily high
  low: Numeric                # Daily low
  open: Numeric               # Opening price
```

### Optional Metadata

```sig
columns:
  sector: String              # Sector classification
  industry: String            # Industry classification
  market_cap: Numeric         # Market capitalization
```

## Column Aliasing

Rename columns for consistency:

```sig
data:
  source = "bloomberg_data.csv"
  format = csv
  columns:
    PX_LAST: Numeric as prices
    PX_VOLUME: Numeric as volume
    PX_HIGH: Numeric as high
    PX_LOW: Numeric as low
```

Use in signals:

```sig
signal example:
  // Use aliased name 'prices' instead of 'PX_LAST'
  emit zscore(ret(prices, 20))
```

## CSV Options

### Date Parsing

```sig
data:
  source = "data.csv"
  format = csv
  columns:
    date: Date
    ...
  options:
    date_format = "%Y-%m-%d"       # ISO format (default)
    # date_format = "%m/%d/%Y"     # US format
    # date_format = "%d/%m/%Y"     # European format
```

### Delimiter

```sig
data:
  source = "data.tsv"
  format = csv
  options:
    delimiter = "\t"   # Tab-separated
    # delimiter = ";"  # Semicolon
```

### Header

```sig
data:
  source = "no_header.csv"
  format = csv
  options:
    has_header = false
  columns:
    col1: Date
    col2: Symbol
    col3: Numeric as prices
```

### Encoding

```sig
data:
  source = "data.csv"
  format = csv
  options:
    encoding = "utf-8"   # Default
    # encoding = "latin1"
```

### Skip Rows

```sig
data:
  source = "data_with_notes.csv"
  format = csv
  options:
    skip_rows = 3   # Skip first 3 rows
```

## Type Inference

sigc can infer types, but explicit is better:

```sig
// Explicit (recommended)
columns:
  date: Date
  ticker: Symbol
  close: Numeric as prices
  volume: Numeric

// Inferred (less reliable)
data:
  source = "data.csv"
  format = csv
  options:
    infer_types = true
```

## Multiple CSV Files

### Single Directory

```sig
data:
  source = "data/*.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
```

### Explicit List

```sig
data:
  source = ["prices_2023.csv", "prices_2024.csv"]
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
```

## Handling Missing Data

### In CSV

```csv
date,ticker,close,volume
2024-01-02,AAPL,185.64,45678900
2024-01-02,MSFT,,23456789         # Missing price
2024-01-03,AAPL,184.25,           # Missing volume
```

### In sigc

```sig
signal handle_missing:
  // Option 1: Fill with value
  clean_prices = fill_nan(prices, 0)

  // Option 2: Forward fill
  filled = where(is_nan(prices), lag(prices, 1), prices)

  // Option 3: Exclude
  valid = not(is_nan(prices))
  filtered = where(valid, prices, 0)

  emit zscore(ret(clean_prices, 20))
```

## Performance Considerations

### File Size Guidelines

| Size | Rows | Recommendation |
|------|------|----------------|
| Small | < 100K | CSV is fine |
| Medium | 100K - 1M | Consider Parquet |
| Large | > 1M | Use Parquet |

### Optimization Tips

```sig
// 1. Use specific columns (don't load unused data)
columns:
  date: Date
  ticker: Symbol
  close: Numeric as prices  # Only load what you need

// 2. Convert to Parquet for repeated use
// Run once: sigc convert data.csv data.parquet

// 3. Use date filtering
options:
  start_date = "2020-01-01"
  end_date = "2024-12-31"
```

## Common Issues

### Date Parsing Errors

```
Error: Failed to parse date '01/15/2024'
```

Fix: Specify date format:

```sig
options:
  date_format = "%m/%d/%Y"
```

### Missing Column

```
Error: Column 'prices' not found
```

Fix: Check column names or add alias:

```sig
columns:
  close: Numeric as prices  # 'close' in CSV, 'prices' in code
```

### Encoding Issues

```
Error: Invalid UTF-8 sequence
```

Fix: Specify encoding:

```sig
options:
  encoding = "latin1"
```

## Example Files

### OHLCV Data

```csv
date,ticker,open,high,low,close,volume
2024-01-02,AAPL,185.00,186.50,184.00,185.64,45678900
2024-01-02,MSFT,373.00,375.00,372.00,374.51,23456789
```

```sig
data:
  source = "ohlcv.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    open: Numeric
    high: Numeric
    low: Numeric
    close: Numeric as prices
    volume: Numeric
```

### With Fundamentals

```csv
date,ticker,close,pe_ratio,book_value,market_cap
2024-01-02,AAPL,185.64,28.5,4.25,2850000000000
2024-01-02,MSFT,374.51,35.2,24.50,2780000000000
```

```sig
data:
  source = "with_fundamentals.csv"
  format = csv
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    pe_ratio: Numeric
    book_value: Numeric
    market_cap: Numeric
```

## Next Steps

- [Parquet Format](parquet.md) - Faster loading for large datasets
- [Data Quality](data-quality.md) - Validating your data
- [Signal Section](../language/signal-section.md) - Using loaded data
