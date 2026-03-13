# Data Loading Guide

How to load and connect data sources in sigc.

## Supported Formats

| Format | Extension | Declaration |
|--------|-----------|-------------|
| CSV | `.csv` | `load csv from "path"` |
| Parquet | `.parquet` | `load parquet from "path"` |
| S3 | `s3://` | `load parquet from "s3://bucket/key"` |

## CSV Files

### Basic Usage

```
data:
  prices: load csv from "data/prices.csv"
```

### Expected Format

CSV must have:
- First column: `date` (YYYY-MM-DD format)
- Remaining columns: asset prices

```csv
date,AAPL,MSFT,GOOGL
2024-01-02,185.64,374.58,140.25
2024-01-03,184.25,373.31,139.12
```

### Multiple Files

```
data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"
```

## Parquet Files

More efficient for large datasets:

```
data:
  prices: load parquet from "data/prices.parquet"
```

## S3 Sources

### Configuration

Set AWS credentials:
```bash
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_REGION=us-east-1
```

### Usage

```
data:
  prices: load parquet from "s3://my-bucket/data/prices.parquet"
```

## Date Filtering

Dates in the `portfolio` block filter loaded data:

```
portfolio main:
  weights = rank(signal).long_short(top=0.2, bottom=0.2)
  backtest from 2024-01-01 to 2024-06-30
```

Only data between these dates is used.

## Data Requirements

### Minimum Data

- At least 2 assets (columns)
- Enough history for your lookback periods
- No gaps in dates (business days)

### Handling Missing Data

sigc handles NaN values:
- Operations propagate NaN
- Use `fill_nan()` to replace
- Use `coalesce()` for fallbacks

```
signal clean:
  raw = ret(prices, 20)
  cleaned = fill_nan(raw, 0)
  emit cleaned
```

## Troubleshooting

### "Failed to load CSV"

- Check file path is correct
- Verify file exists: `ls -la data/prices.csv`
- Check CSV has proper headers

### "No data in date range"

- Ensure data covers your backtest dates
- Check date format is YYYY-MM-DD

### S3 Timeout

- Verify AWS credentials
- Check bucket permissions
- Ensure region is correct
