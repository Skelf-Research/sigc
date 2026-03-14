# Parallel Execution

Leverage multi-core processing for faster computation.

## Overview

sigc automatically parallelizes:

- Cross-sectional computations (across assets)
- Independent signal calculations
- Parameter optimization
- Walk-forward windows

## Configuration

### Enable Parallelism

```yaml
performance:
  parallel:
    enabled: true
    workers: 8  # Number of threads
```

### Auto-Detection

```yaml
performance:
  parallel:
    enabled: true
    workers: auto  # Use all available cores
```

## What Gets Parallelized

### Cross-Sectional Operations

Operations across assets run in parallel:

```sig
signal momentum:
  // This runs in parallel across all assets
  emit zscore(ret(prices, 60))
```

### Independent Signals

Multiple signals compute simultaneously:

```sig
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal quality:
  emit zscore(roe)

// All three signals compute in parallel
signal combined:
  emit 0.33 * momentum + 0.33 * value + 0.34 * quality
```

### Parameter Grid

```sig
params:
  lookback: range(20, 120, 20)
  top_pct: range(0.1, 0.4, 0.1)

// Each parameter combination runs in parallel
portfolio optimized:
  weights = rank(momentum).long_short(top=top_pct, bottom=top_pct)
  backtest from 2020-01-01 to 2024-12-31
```

### Walk-Forward Windows

```sig
portfolio validated:
  backtest walk_forward(
    train_years = 5,
    test_years = 2,
    step_years = 2,
    parallel = true  // Windows compute in parallel
  ) from 2010-01-01 to 2024-12-31
```

## Thread Pool Configuration

### Basic Configuration

```yaml
performance:
  parallel:
    enabled: true
    workers: 8
    stack_size_mb: 2
```

### Advanced Configuration

```yaml
performance:
  parallel:
    enabled: true
    workers: 8

    # Thread pool settings
    thread_pool:
      name: "sigc-compute"
      stack_size_mb: 4

    # Work stealing
    work_stealing: true

    # Granularity
    min_batch_size: 100  # Min items per thread
```

## Parallelism Strategies

### Data Parallelism

Same operation on different data:

```
┌─────────────────────────────────────────────────────┐
│ zscore(ret(prices, 60))                             │
│                                                     │
│  Thread 1    Thread 2    Thread 3    Thread 4      │
│  ┌───────┐   ┌───────┐   ┌───────┐   ┌───────┐    │
│  │AAPL   │   │GOOGL  │   │AMZN   │   │NVDA   │    │
│  │MSFT   │   │META   │   │TSLA   │   │JPM    │    │
│  │...    │   │...    │   │...    │   │...    │    │
│  └───────┘   └───────┘   └───────┘   └───────┘    │
└─────────────────────────────────────────────────────┘
```

### Task Parallelism

Different operations simultaneously:

```
┌─────────────────────────────────────────────────────┐
│ Multiple Signals                                     │
│                                                     │
│  Thread 1         Thread 2         Thread 3        │
│  ┌───────────┐   ┌───────────┐   ┌───────────┐    │
│  │ momentum  │   │   value   │   │  quality  │    │
│  │ signal    │   │  signal   │   │  signal   │    │
│  └───────────┘   └───────────┘   └───────────┘    │
│         │              │              │            │
│         └──────────────┼──────────────┘            │
│                        ▼                           │
│                 ┌───────────┐                      │
│                 │  combined │                      │
│                 └───────────┘                      │
└─────────────────────────────────────────────────────┘
```

## Performance Tuning

### Optimal Worker Count

```yaml
# CPU-bound: Use all cores
workers: auto

# Memory-bound: Use fewer
workers: 4

# I/O-bound: Can exceed core count
workers: 16
```

### Batch Size

```yaml
performance:
  parallel:
    min_batch_size: 100  # Minimum items per thread
```

Small batch = more overhead
Large batch = less parallelism

### Memory Considerations

```yaml
performance:
  parallel:
    workers: 4  # Reduce if memory-constrained
  memory:
    max_memory_gb: 8
```

## Measuring Performance

### Benchmark Mode

```bash
sigc run strategy.sig --benchmark
```

Output:

```
Performance Benchmark:
======================
Total Time: 2.34s
  Data Loading: 0.45s (19%)
  Signal Computation: 1.52s (65%)
  Portfolio Construction: 0.25s (11%)
  Backtest Simulation: 0.12s (5%)

Parallelism:
  Workers Used: 8
  Parallel Efficiency: 85%
  Speedup vs Serial: 6.8x

Memory:
  Peak Usage: 1.2 GB
  Data Size: 850 MB
```

### Profiling

```bash
sigc run strategy.sig --profile
```

Generates detailed profile report.

## CLI Options

```bash
# Specify worker count
sigc run strategy.sig --workers 8

# Disable parallelism
sigc run strategy.sig --workers 1

# Auto-detect
sigc run strategy.sig --workers auto
```

## Best Practices

### 1. Match Workers to Cores

```yaml
workers: auto  # Or number of physical cores
```

### 2. Profile Before Optimizing

```bash
sigc run strategy.sig --benchmark
```

### 3. Consider Memory

More workers = more memory usage.

### 4. Batch Small Operations

Very small operations have parallelism overhead.

### 5. Use SIMD Where Available

sigc uses SIMD for rolling statistics automatically.

## Limitations

### Sequential Dependencies

Some operations must be sequential:

```sig
// Cumulative operations are sequential
cum_ret = cumsum(daily_return)
```

### Cross-Time Dependencies

```sig
// Requires previous values
ema = ema(prices, 20)  // Sequential per asset
```

### Memory Bandwidth

Parallelism limited by memory bandwidth for large datasets.

## Troubleshooting

### High CPU, Low Speedup

- Too many small tasks
- Memory bandwidth limited
- Lock contention

### Memory Issues

```yaml
# Reduce parallelism
workers: 4

# Or increase memory limit
memory:
  max_memory_gb: 16
```

### Inconsistent Results

Ensure operations are deterministic:

```yaml
performance:
  deterministic: true
  seed: 42
```

## Next Steps

- [Incremental Computation](incremental-computation.md) - Efficient updates
- [Memory Mapping](memory-mapping.md) - Large datasets
- [Configuration](../production/configuration.md) - Full config reference
