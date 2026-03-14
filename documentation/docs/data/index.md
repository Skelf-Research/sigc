# Data Loading

sigc supports multiple data sources and formats for loading market data.

## Supported Formats

| Format | Extension | Best For |
|--------|-----------|----------|
| [CSV](csv.md) | `.csv` | Small datasets, debugging |
| [Parquet](parquet.md) | `.parquet` | Large datasets, production |
| [S3](s3.md) | `s3://` | Cloud storage |
| [PostgreSQL](postgresql.md) | `postgresql://` | Database integration |

## Quick Examples

### CSV

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

### Parquet

```sig
data:
  source = "market_data.parquet"
  format = parquet
```

### S3

```sig
data:
  source = "s3://bucket/data/prices.parquet"
  format = parquet
```

### PostgreSQL

```sig
data:
  source = "postgresql://localhost/marketdb"
  query = "SELECT date, ticker, close, volume FROM daily_prices"
```

## Data Section Structure

```sig
data:
  source = "..."           # Required: file path, URL, or connection string
  format = csv | parquet   # Required for files
  columns:                 # Column definitions (required for CSV)
    column_name: Type [as alias]
  options:                 # Optional: format-specific settings
    ...
```

## Column Types

| Type | Description | Example |
|------|-------------|---------|
| `Date` | Date column (index) | `2024-01-15` |
| `Symbol` | Asset identifier | `AAPL`, `MSFT` |
| `Numeric` | Price, volume, etc. | `150.25` |
| `String` | Text data | `"Technology"` |

## Column Aliasing

Rename columns for cleaner code:

```sig
data:
  source = "raw_data.csv"
  format = csv
  columns:
    trade_date: Date
    symbol: Symbol
    adj_close: Numeric as prices      # Use 'prices' in signals
    shares_traded: Numeric as volume  # Use 'volume' in signals
```

## Multiple Data Sources

Combine data from multiple files:

```sig
data prices:
  source = "prices.parquet"
  format = parquet

data fundamentals:
  source = "fundamentals.parquet"
  format = parquet

signal combined:
  momentum = zscore(ret(prices.close, 60))
  value = zscore(fundamentals.book_to_market)
  emit 0.5 * momentum + 0.5 * value
```

## Data Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ CSV/Parquet │───▶│   Parser    │───▶│   Panel     │
│    File     │    │  (Polars)   │    │ (dates×sym) │
└─────────────┘    └─────────────┘    └─────────────┘
                          │
                          ▼
              ┌─────────────────────┐
              │   Type Validation   │
              │  - Column types     │
              │  - Date parsing     │
              │  - Symbol mapping   │
              └─────────────────────┘
```

## Best Practices

### 1. Use Parquet for Production

```sig
// Parquet is 5-10x faster than CSV for large datasets
data:
  source = "prices.parquet"
  format = parquet
```

### 2. Define Types Explicitly

```sig
// Good: explicit types
columns:
  date: Date
  ticker: Symbol
  close: Numeric as prices

// Bad: relying on inference
```

### 3. Handle Missing Data

```sig
signal robust:
  // Fill missing prices before computation
  clean = fill_nan(prices, 0)
  emit zscore(ret(clean, 20))
```

### 4. Use Consistent Naming

```sig
// Alias to standard names
columns:
  px_last: Numeric as prices
  px_volume: Numeric as volume
  px_high: Numeric as high
  px_low: Numeric as low
```

## Data Quality

See [Data Quality](data-quality.md) for:

- Missing data detection
- Outlier handling
- Corporate action adjustments
- Survivorship bias

## Next Steps

- [CSV Format](csv.md) - CSV file loading
- [Parquet Format](parquet.md) - Parquet file loading
- [Data Quality](data-quality.md) - Data validation
- [Corporate Actions](corporate-actions.md) - Handling splits and dividends
