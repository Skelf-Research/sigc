# Data Section

The `data:` section declares data sources for your strategy.

## Syntax

```sig
data:
  <name>: load <format> from <path> [options]
```

## Supported Formats

### CSV

Comma-separated values:

```sig
data:
  prices: load csv from "data/prices.csv"
```

Expected format:

```csv
date,AAPL,MSFT,GOOGL
2024-01-02,185.64,374.58,140.25
2024-01-03,184.25,373.31,139.12
```

### Parquet

Apache Parquet (recommended for large datasets):

```sig
data:
  prices: load parquet from "data/prices.parquet"
```

Benefits:

- Columnar storage (efficient for analytics)
- Compression (smaller file sizes)
- Type preservation

### Arrow

Apache Arrow IPC format:

```sig
data:
  prices: load arrow from "data/prices.arrow"
```

Best for:

- Inter-process communication
- Zero-copy reads

## Data Sources

### Local Files

```sig
data:
  prices: load csv from "data/prices.csv"
  volume: load parquet from "/absolute/path/volume.parquet"
  factors: load csv from "../relative/path/factors.csv"
```

### S3

```sig
data:
  prices: load parquet from "s3://my-bucket/data/prices.parquet"
  fundamentals: load parquet from "s3://my-bucket/fundamentals.parquet"
```

Set credentials via environment variables:

```bash
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_DEFAULT_REGION=us-east-1
```

### Azure Blob Storage

```sig
data:
  prices: load parquet from "az://container/prices.parquet"
```

### Google Cloud Storage

```sig
data:
  prices: load parquet from "gs://bucket/prices.parquet"
```

## Options

### Price Adjustment

Adjust for corporate actions:

```sig
data:
  prices: load csv from "data/prices.csv" adjust=split_div
```

Options:

- `adjust=split` - Adjust for stock splits only
- `adjust=split_div` - Adjust for splits and dividends
- `adjust=none` - No adjustment (default)

### Data Type

Specify column data type:

```sig
data:
  sectors: load csv from "data/sectors.csv" dtype=category
```

Types:

- `dtype=float64` - Double precision (default for numeric)
- `dtype=float32` - Single precision
- `dtype=int64` - 64-bit integer
- `dtype=category` - Categorical data

### Date Column

Specify which column contains dates:

```sig
data:
  prices: load csv from "data/prices.csv" date_col="timestamp"
```

Default: First column is assumed to be the date.

## Multiple Data Sources

Load multiple datasets:

```sig
data:
  prices: load csv from "data/prices.csv"
  volume: load csv from "data/volume.csv"
  market_cap: load parquet from "data/fundamentals.parquet"
  sectors: load csv from "data/sectors.csv" dtype=category
```

Use them in signals:

```sig
signal composite:
  mom = zscore(ret(prices, 20))
  vol_signal = zscore(volume)
  size = -zscore(log(market_cap))
  neutral = neutralize(mom, by=sectors)
  emit neutral
```

## Data Format Requirements

### Panel Structure

Data should be in panel format (dates as rows, assets as columns):

```csv
date,AAPL,MSFT,GOOGL,AMZN
2024-01-02,185.64,374.58,140.25,151.94
2024-01-03,184.25,373.31,139.12,149.93
2024-01-04,181.91,367.94,137.98,147.44
```

### Date Formats

Supported date formats:

- `YYYY-MM-DD` (preferred): `2024-01-15`
- `YYYY/MM/DD`: `2024/01/15`
- `MM/DD/YYYY`: `01/15/2024`
- ISO 8601: `2024-01-15T00:00:00`

### Missing Values

Missing values can be:

- Empty cells
- `NaN`
- `NA`
- `null`

Handle in signals:

```sig
signal cleaned:
  filled = fill_nan(prices, 0)
  emit zscore(filled)
```

## Examples

### Basic Usage

```sig
data:
  prices: load csv from "data/prices.csv"

signal example:
  emit zscore(ret(prices, 20))
```

### Multiple Sources

```sig
data:
  prices: load csv from "data/prices.csv"
  book_value: load csv from "data/book_value.csv"
  earnings: load parquet from "data/earnings.parquet"

signal value:
  pb = prices / book_value
  pe = prices / earnings
  emit -zscore(pb + pe)  // Lower is better for value
```

### S3 with Options

```sig
data:
  prices: load parquet from "s3://quant-data/prices.parquet" adjust=split_div
  sectors: load parquet from "s3://quant-data/sectors.parquet" dtype=category

signal sector_neutral_mom:
  raw = zscore(ret(prices, 60))
  neutral = neutralize(raw, by=sectors)
  emit neutral
```

### Fundamental Data

```sig
data:
  prices: load csv from "data/prices.csv"
  roe: load csv from "data/roe.csv"
  debt_equity: load csv from "data/debt_equity.csv"
  revenue_growth: load csv from "data/revenue_growth.csv"

signal quality:
  roe_z = zscore(roe)
  leverage_z = -zscore(debt_equity)  // Lower debt is better
  growth_z = zscore(revenue_growth)
  emit (roe_z + leverage_z + growth_z) / 3
```

## Troubleshooting

### File Not Found

```
Error: Failed to load data from "data/prices.csv"
  File not found
```

Check:

- File path is correct
- Working directory is correct
- File has read permissions

### Parse Error

```
Error: Failed to parse CSV
  Invalid number at row 5, column 3
```

Check:

- Data format is correct
- No text in numeric columns
- Consistent delimiters

### Date Parse Error

```
Error: Failed to parse date "01-15-2024"
```

Use supported format: `2024-01-15`

### S3 Access Denied

```
Error: Access denied to s3://bucket/file.parquet
```

Check:

- AWS credentials are set
- IAM permissions allow read access
- Bucket policy allows access

## Next Steps

- [Params Section](params-section.md) - Define parameters
- [Data Loading Guide](../data/index.md) - Advanced data loading
- [Corporate Actions](../data/corporate-actions.md) - Handling splits/dividends
