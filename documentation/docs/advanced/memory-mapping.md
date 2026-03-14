# Memory Mapping

Handle datasets larger than available RAM efficiently.

## Overview

Memory mapping allows sigc to:

- Work with datasets larger than RAM
- Share data between processes
- Reduce startup time for large files

## How Memory Mapping Works

```
Traditional Loading:
┌─────────────┐    ┌─────────────┐
│  Disk File  │───▶│    RAM      │
│   10 GB     │    │   10 GB     │
└─────────────┘    └─────────────┘

Memory Mapped:
┌─────────────┐    ┌─────────────┐
│  Disk File  │◀──▶│ Virtual Mem │
│   10 GB     │    │ (on demand) │
└─────────────┘    └─────────────┘
```

## Configuration

### Enable Memory Mapping

```yaml
data:
  source = "large_dataset.parquet"
  format = parquet

performance:
  memory_map:
    enabled: true
```

### With Limits

```yaml
performance:
  memory_map:
    enabled: true
    max_mapped_size_gb: 50

  memory:
    max_memory_gb: 8  # Physical RAM limit
```

## Supported Formats

### Parquet (Recommended)

```yaml
data:
  source = "prices.parquet"
  format = parquet

performance:
  memory_map:
    enabled: true
```

Parquet advantages:
- Columnar format
- Built-in compression
- Efficient for analytics

### Arrow IPC

```yaml
data:
  source = "prices.arrow"
  format = arrow

performance:
  memory_map:
    enabled: true
    zero_copy: true  # No data copying
```

### Memory-Mapped CSV

```yaml
data:
  source = "prices.csv"
  format = csv

performance:
  memory_map:
    enabled: true
    preprocess: true  # Convert to arrow format first
```

## Usage Patterns

### Read-Only Access

Default mode, optimal for most cases:

```yaml
performance:
  memory_map:
    enabled: true
    mode: read_only
```

### Shared Access

Multiple processes can read the same mapped file:

```yaml
performance:
  memory_map:
    enabled: true
    mode: shared
```

## Memory Management

### Physical RAM Usage

Memory mapping doesn't load everything into RAM:

```
10 GB file, 4 GB RAM:
- Pages loaded on demand
- Unused pages evicted by OS
- Only active data in RAM
```

### Controlling Memory

```yaml
performance:
  memory:
    max_memory_gb: 8       # Hard limit
    target_memory_gb: 6    # Target usage

  memory_map:
    enabled: true
    prefetch: false        # Don't prefetch pages
```

## Large Dataset Workflows

### Historical Backtesting

```sig
data:
  source = "10_years_data.parquet"  // 50 GB file
  format = parquet

performance:
  memory_map:
    enabled: true

signal momentum:
  emit zscore(ret(prices, 252))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest from 2015-01-01 to 2024-12-31
```

### Multi-Asset Universe

```yaml
data:
  source = "global_equities.parquet"  // 5000+ assets
  format = parquet

performance:
  memory_map:
    enabled: true

  parallel:
    enabled: true
    workers: 8
```

## Performance Optimization

### Column Selection

Only access needed columns:

```yaml
data:
  source = "all_data.parquet"
  format = parquet
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric as volume
  # Other columns not loaded
```

### Row Filtering

Filter early to reduce data:

```yaml
data:
  source = "global_data.parquet"
  filter:
    - column: country
      value: "US"
    - column: market_cap
      op: ">"
      value: 1000000000  # $1B+
```

### Chunked Processing

For very large files:

```yaml
performance:
  memory_map:
    enabled: true
    chunk_size_mb: 256  # Process in chunks
```

## Benchmarks

### Load Time Comparison

| Dataset Size | Traditional | Memory Mapped |
|-------------|-------------|---------------|
| 1 GB | 5s | 0.1s |
| 10 GB | 50s | 0.5s |
| 50 GB | Out of memory | 2s |

### Query Performance

| Query | Traditional | Memory Mapped |
|-------|-------------|---------------|
| Single asset | 10ms | 15ms |
| Full scan | 500ms | 600ms |
| Filtered scan | 200ms | 180ms |

Memory mapping adds slight overhead for cached data but enables working with larger datasets.

## Preprocessing for Performance

### Convert CSV to Parquet

```bash
sigc convert prices.csv prices.parquet
```

### Optimize Parquet

```bash
sigc optimize prices.parquet --row-group-size 100000 --compression zstd
```

### Create Arrow Format

```bash
sigc convert prices.parquet prices.arrow --format arrow
```

## Multi-Process Sharing

### Multiple Backtests

Run parameter sweeps sharing data:

```yaml
# config.yaml
data:
  source = "prices.parquet"

performance:
  memory_map:
    enabled: true
    mode: shared  # Share across processes
```

```bash
# Run multiple backtests sharing same mapped file
sigc run strategy.sig --param lookback=20 &
sigc run strategy.sig --param lookback=40 &
sigc run strategy.sig --param lookback=60 &
wait
```

All processes share the same memory-mapped file, reducing total RAM usage.

## Troubleshooting

### Slow Performance

**Symptom:** Memory-mapped file slower than expected

**Cause:** Too much paging (disk thrashing)

**Solution:**
```yaml
performance:
  memory:
    max_memory_gb: 16  # Increase if available
```

### Out of Virtual Memory

**Symptom:** "Cannot map file" error

**Cause:** 32-bit process or system limits

**Solution:** Use 64-bit system, check ulimits

### Page Faults

**Symptom:** High page fault count

**Cause:** Random access pattern

**Solution:**
```yaml
performance:
  memory_map:
    prefetch: true      # Prefetch data
    sequential_hint: true  # Optimize for sequential access
```

## Best Practices

### 1. Use Parquet for Large Files

Parquet + memory mapping is optimal:
- Columnar layout
- Compression
- Column pruning

### 2. Pre-filter Data

```yaml
data:
  filter:
    - column: date
      op: ">="
      value: "2020-01-01"
```

### 3. Select Only Needed Columns

```yaml
columns:
  date: Date
  ticker: Symbol
  close: Numeric as prices
  # Don't include columns you don't use
```

### 4. Monitor Memory Usage

```bash
sigc run strategy.sig --benchmark
```

Shows memory statistics including mapped memory.

### 5. Use SSD Storage

Memory mapping benefits from fast storage:
- SSD: Good performance
- NVMe: Best performance
- HDD: Acceptable for sequential access

## System Configuration

### Linux

```bash
# Increase max mapped memory
sudo sysctl -w vm.max_map_count=262144

# Make permanent
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
```

### macOS

Generally no configuration needed for most use cases.

### Windows

Use large pages for better performance:

```yaml
performance:
  memory_map:
    enabled: true
    large_pages: true  # Requires admin rights
```

## Next Steps

- [Parallel Execution](parallel-execution.md) - Multi-core processing
- [Incremental Computation](incremental-computation.md) - Efficient updates
- [Configuration](../production/configuration.md) - Full config reference
