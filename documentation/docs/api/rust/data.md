# Data Module

Data loading and manipulation.

## Loading Data

### From Parquet

```rust
use sigc::data::DataFrame;

let df = DataFrame::from_parquet("prices.parquet")?;
```

### From CSV

```rust
let df = DataFrame::from_csv("prices.csv")?;
```

### From S3

```rust
let df = DataFrame::from_s3("s3://bucket/prices.parquet")?;
```

### From PostgreSQL

```rust
let df = DataFrame::from_sql(
    "postgres://user:pass@host/db",
    "SELECT * FROM prices WHERE date > '2020-01-01'"
)?;
```

## DataFrame Operations

### Column Access

```rust
// Get column
let prices = df.column("close")?;

// Get multiple columns
let subset = df.select(&["date", "ticker", "close"])?;
```

### Filtering

```rust
// Filter rows
let filtered = df.filter(|row| row["market_cap"] > 1_000_000_000.0)?;

// Filter by date
let recent = df.filter_date_range("2020-01-01", "2024-12-31")?;
```

### Grouping

```rust
// Group by column
let grouped = df.group_by("sector")?;

// Aggregate
let sector_avg = grouped.mean("close")?;
```

### Joining

```rust
let prices = DataFrame::from_parquet("prices.parquet")?;
let fundamentals = DataFrame::from_parquet("fundamentals.parquet")?;

let joined = prices.join(&fundamentals, &["date", "ticker"])?;
```

## Time Series Operations

### Lag

```rust
let lagged = df.lag("close", 1)?;
```

### Diff

```rust
let returns = df.pct_change("close", 1)?;
```

### Rolling

```rust
let ma_20 = df.rolling_mean("close", 20)?;
let std_60 = df.rolling_std("close", 60)?;
```

## Data Validation

### Check Missing

```rust
let missing = df.missing_values()?;
for (col, count) in missing {
    println!("{}: {} missing", col, count);
}
```

### Fill Missing

```rust
// Forward fill
let filled = df.forward_fill("close")?;

// Fill with value
let filled = df.fill_value("close", 0.0)?;
```

### Remove Duplicates

```rust
let deduped = df.drop_duplicates(&["date", "ticker"])?;
```

## Data Types

### Column Types

```rust
use sigc::data::ColumnType;

let schema = df.schema();
for (name, dtype) in schema.iter() {
    match dtype {
        ColumnType::Numeric => println!("{}: Numeric", name),
        ColumnType::Date => println!("{}: Date", name),
        ColumnType::Symbol => println!("{}: Symbol", name),
        ColumnType::Category => println!("{}: Category", name),
    }
}
```

### Type Conversion

```rust
let df = df.cast_column("close", ColumnType::Numeric)?;
```

## Memory Mapping

### Enable Memory Mapping

```rust
let df = DataFrame::from_parquet_mmap("large_file.parquet")?;
```

### Configure

```rust
use sigc::data::MmapOptions;

let options = MmapOptions::default()
    .max_size_gb(50)
    .read_only(true);

let df = DataFrame::from_parquet_with_options("data.parquet", options)?;
```

## Export

```rust
// To Parquet
df.to_parquet("output.parquet")?;

// To CSV
df.to_csv("output.csv")?;

// To Arrow
df.to_arrow("output.arrow")?;
```

## See Also

- [Strategy Module](strategy.md)
- [Types Module](types.md)
