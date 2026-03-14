# Incremental Computation

Update strategies efficiently when new data arrives.

## Overview

Instead of recomputing everything from scratch, sigc can incrementally update:

- Rolling statistics
- Signal values
- Portfolio weights

## How It Works

### Traditional vs Incremental

**Traditional (Full Recompute):**
```
Day N: Compute everything from scratch
Day N+1: Compute everything from scratch again
```

**Incremental:**
```
Day N: Compute everything
Day N+1: Update only what changed
```

### Performance Comparison

| Operation | Full Recompute | Incremental | Speedup |
|-----------|---------------|-------------|---------|
| Rolling mean (252) | O(n × 252) | O(n) | 252x |
| Rolling std (60) | O(n × 60) | O(n) | 60x |
| EMA (20) | O(n × 20) | O(n) | 20x |

## Enabling Incremental Mode

### Configuration

```yaml
performance:
  incremental:
    enabled: true
    checkpoint_interval: 1d  # Save state daily
```

### CLI Flag

```bash
sigc run strategy.sig --incremental
```

## Incremental Operations

### Rolling Mean

Maintains running sum:

```
new_mean = old_mean + (new_value - oldest_value) / window
```

### Rolling Standard Deviation

Uses Welford's online algorithm:

```
new_variance = update(old_variance, new_value, oldest_value)
new_std = sqrt(new_variance)
```

### Exponential Moving Average

Naturally incremental:

```
new_ema = alpha * new_value + (1 - alpha) * old_ema
```

## State Management

### Checkpoint Files

```yaml
performance:
  incremental:
    enabled: true
    checkpoint_dir: "./checkpoints"
    checkpoint_interval: 1d
```

Checkpoint structure:
```
checkpoints/
├── momentum.state
├── value.state
└── portfolio.state
```

### Checkpoint Contents

```
Checkpoint: momentum
  Last Date: 2024-01-15
  Rolling Buffers:
    - ret_60: [252 values per asset]
    - zscore_252: [252 values per asset]
  Computed Values:
    - AAPL: 1.234
    - GOOGL: -0.567
    ...
```

## Daemon Mode Integration

### Automatic Incremental Updates

```yaml
daemon:
  enabled: true
  mode: incremental

  schedule:
    - cron: "0 16 * * 1-5"  # 4 PM weekdays
      action: update
```

### Update Flow

```
1. Load checkpoint from previous run
2. Fetch new data since checkpoint
3. Update rolling statistics
4. Compute new signal values
5. Generate weights
6. Save new checkpoint
```

## Memory Efficiency

### Buffer Management

Only keeps required history:

```yaml
performance:
  incremental:
    enabled: true
    max_history: 252  # Maximum lookback needed
```

### Example

If your longest lookback is 252 days:
```sig
signal momentum:
  emit zscore(ret(prices, 252))  // Needs 252 days
```

sigc keeps only 252 days in memory, not full history.

## Supported Operations

### Fully Incremental

| Operation | State Size |
|-----------|-----------|
| `rolling_mean` | O(window) |
| `rolling_std` | O(window) |
| `rolling_sum` | O(window) |
| `ema` | O(1) |
| `lag` | O(lag) |
| `diff` | O(1) |

### Partially Incremental

| Operation | Notes |
|-----------|-------|
| `rolling_corr` | Requires covariance state |
| `rolling_rank` | Requires sorted buffer |
| `quantile` | Requires sorted buffer |

### Non-Incremental

Some operations require full recompute:

| Operation | Reason |
|-----------|--------|
| `zscore` (cross-sectional) | Needs all assets |
| `rank` (cross-sectional) | Needs all assets |
| `neutralize` | Needs all assets |

## Handling Dependencies

### Dependency Graph

```sig
signal a:
  emit rolling_mean(prices, 20)  // Independent

signal b:
  emit rolling_std(prices, 20)  // Independent

signal c:
  emit a / b  // Depends on a and b
```

Update order:
```
1. Update a (parallel)
2. Update b (parallel)
3. Update c (after a and b complete)
```

### Automatic Ordering

sigc automatically determines update order based on dependencies.

## Error Recovery

### Checkpoint Validation

```yaml
performance:
  incremental:
    validate_checkpoints: true
```

Validates:
- Checkpoint date matches expected
- All assets present
- Buffer sizes correct

### Recovery Options

```yaml
performance:
  incremental:
    on_error: recompute  # Full recompute on error
    # or: fail          # Stop and report
```

## Full Recompute Triggers

Force full recompute when:

1. **No checkpoint exists**
2. **Strategy changed**
3. **Data gap detected**
4. **Checkpoint corrupted**
5. **Manual trigger**

```bash
# Force full recompute
sigc run strategy.sig --no-incremental
```

## Best Practices

### 1. Set Checkpoint Interval

Balance between:
- Too frequent: Disk I/O overhead
- Too rare: Long recovery time

```yaml
checkpoint_interval: 1d  # Good default
```

### 2. Monitor Checkpoint Size

```bash
sigc status --checkpoints
```

### 3. Validate Periodically

Run full recompute periodically to verify:

```yaml
schedule:
  - cron: "0 0 * * 0"  # Weekly Sunday
    action: full_recompute
```

### 4. Handle Universe Changes

When assets are added/removed:

```yaml
performance:
  incremental:
    on_universe_change: recompute
```

## Example: Daily Update Workflow

```yaml
# config.yaml
data:
  source = "prices.parquet"

daemon:
  enabled: true

  schedule:
    - cron: "0 16 * * 1-5"
      action: update

performance:
  incremental:
    enabled: true
    checkpoint_dir: "./checkpoints"
    checkpoint_interval: 1d

# Strategy
signal momentum:
  emit zscore(ret(prices, 60))

portfolio main:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
```

Daily execution:
```
4:00 PM - Daemon triggers
4:00:01 - Load checkpoint
4:00:02 - Fetch today's prices
4:00:03 - Update rolling returns
4:00:04 - Compute new signals
4:00:05 - Generate weights
4:00:06 - Save checkpoint
4:00:07 - Output weights
```

## Troubleshooting

### Slow Incremental Updates

Check if cross-sectional operations are the bottleneck:

```bash
sigc run strategy.sig --incremental --profile
```

### Checkpoint Mismatch

```
Error: Checkpoint date mismatch
Expected: 2024-01-15
Found: 2024-01-14
```

Solution: Check for missing data or run full recompute.

### Memory Issues

Reduce checkpoint buffer sizes:

```yaml
performance:
  incremental:
    max_history: 126  # Reduce if possible
```

## Next Steps

- [Memory Mapping](memory-mapping.md) - Large dataset handling
- [Parallel Execution](parallel-execution.md) - Multi-core processing
- [Daemon Mode](../production/daemon-mode.md) - Production scheduling
