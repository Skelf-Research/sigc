# Parquet Format

Apache Parquet is the recommended format for production data in sigc.

## Why Parquet?

| Feature | CSV | Parquet |
|---------|-----|---------|
| Read Speed | Slow | 5-10x faster |
| File Size | Large | 50-90% smaller |
| Type Safety | No | Yes |
| Column Pruning | No | Yes |
| Predicate Pushdown | No | Yes |

## Basic Usage

```sig
data:
  source = "prices.parquet"
  format = parquet
```

Types are automatically inferred from Parquet schema.

## Column Selection

Only load needed columns:

```sig
data:
  source = "market_data.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric
  # Other columns in file are ignored
```

## Parquet Options

### Row Groups

```sig
data:
  source = "large_file.parquet"
  format = parquet
  options:
    row_group_size = 100000  # Rows per group
```

### Compression

sigc reads all standard compressions:

- Snappy (default, fast)
- Gzip (smaller files)
- LZ4 (fastest)
- Zstd (best compression ratio)

### Memory Mapping

```sig
data:
  source = "huge_file.parquet"
  format = parquet
  options:
    memory_map = true  # Memory-map instead of loading
```

## Creating Parquet Files

### From CSV with sigc

```bash
# Convert CSV to Parquet
sigc convert prices.csv prices.parquet
```

### From Python

```python
import pandas as pd

# Read CSV
df = pd.read_csv('prices.csv', parse_dates=['date'])

# Write Parquet
df.to_parquet('prices.parquet', index=False)
```

### From Polars

```python
import polars as pl

# Read and write
df = pl.read_csv('prices.csv')
df.write_parquet('prices.parquet')
```

## Schema Requirements

### Required Columns

Your Parquet file should have:

```
date: Date (or Timestamp)
ticker/symbol: String
[price columns]: Float64
```

### Example Schema

```
message schema {
  required int32 date (DATE);
  required binary ticker (STRING);
  required double close;
  required double volume;
  optional double high;
  optional double low;
  optional double open;
}
```

## Partitioned Data

### Hive-Style Partitioning

```
data/
├── year=2023/
│   ├── month=01/
│   │   └── data.parquet
│   └── month=02/
│       └── data.parquet
└── year=2024/
    └── month=01/
        └── data.parquet
```

```sig
data:
  source = "data/"
  format = parquet
  options:
    partitioned = true
```

### Date Filtering with Partitions

```sig
data:
  source = "data/"
  format = parquet
  options:
    partitioned = true
    start_date = "2023-01-01"  # Only load needed partitions
    end_date = "2024-12-31"
```

## Multiple Parquet Files

### Directory

```sig
data:
  source = "data/*.parquet"
  format = parquet
```

### Explicit List

```sig
data:
  source = ["prices_2023.parquet", "prices_2024.parquet"]
  format = parquet
```

## Column Aliasing

```sig
data:
  source = "data.parquet"
  format = parquet
  columns:
    px_last: Numeric as prices
    px_volume: Numeric as volume
```

## Type Mapping

| Parquet Type | sigc Type |
|--------------|-----------|
| INT32 (DATE) | Date |
| INT64 (TIMESTAMP) | Date |
| STRING | Symbol, String |
| FLOAT, DOUBLE | Numeric |
| BOOLEAN | Boolean |

## Performance Tips

### 1. Column Pruning

```sig
// Only load required columns
data:
  source = "large_file.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices  # Only loads 'close', not all columns
```

### 2. Predicate Pushdown

```sig
data:
  source = "data.parquet"
  format = parquet
  options:
    start_date = "2020-01-01"  # Filter pushed to reader
    symbols = ["AAPL", "MSFT", "GOOGL"]
```

### 3. Row Group Statistics

Parquet stores min/max per row group, enabling fast filtering.

### 4. Memory Mapping for Large Files

```sig
data:
  source = "100gb_file.parquet"
  format = parquet
  options:
    memory_map = true  # Don't load entire file
```

## Compression Comparison

| Compression | Read Speed | Write Speed | Ratio |
|-------------|------------|-------------|-------|
| None | Fastest | Fastest | 1.0x |
| Snappy | Fast | Fast | 2-3x |
| LZ4 | Fast | Fast | 2-3x |
| Gzip | Medium | Slow | 4-6x |
| Zstd | Medium | Medium | 5-8x |

Recommendation: Use **Snappy** for most cases (good balance).

## Example: Converting a Large Dataset

```python
import polars as pl

# Read CSV in chunks
df = pl.scan_csv('huge_prices.csv')

# Write as partitioned Parquet
df.sink_parquet(
    'prices/',
    compression='snappy',
    row_group_size=100_000,
    partition_by=['year', 'month']
)
```

## Troubleshooting

### Schema Mismatch

```
Error: Schema mismatch in file 'data2.parquet'
```

Ensure all Parquet files have identical schemas when loading multiple files.

### Memory Issues

```
Error: Out of memory loading file
```

Solutions:

1. Use memory mapping: `memory_map = true`
2. Select only needed columns
3. Use date filtering
4. Partition large files

### Missing Columns

```
Error: Column 'prices' not found in Parquet schema
```

Check the actual schema:

```bash
# Using Python
python -c "import pyarrow.parquet as pq; print(pq.read_schema('file.parquet'))"

# Using sigc
sigc inspect file.parquet
```

## Best Practices

### 1. Use Snappy Compression

```python
df.to_parquet('data.parquet', compression='snappy')
```

### 2. Choose Appropriate Row Group Size

- Small files (< 1M rows): 50K-100K rows
- Large files: 100K-500K rows

### 3. Partition by Date for Time-Series

```
data/
├── year=2023/
├── year=2024/
```

### 4. Include Metadata

```python
# Add custom metadata
import pyarrow as pa
import pyarrow.parquet as pq

table = pa.Table.from_pandas(df)
metadata = {
    b'source': b'Bloomberg',
    b'created': b'2024-01-15',
    b'frequency': b'daily'
}
table = table.replace_schema_metadata(metadata)
pq.write_table(table, 'data.parquet')
```

## Next Steps

- [S3 Storage](s3.md) - Cloud storage integration
- [Data Quality](data-quality.md) - Validating your data
- [CSV Format](csv.md) - When CSV is appropriate
