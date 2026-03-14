# Reference

Quick reference guides and lookup tables.

## Quick Links

| Reference | Description |
|-----------|-------------|
| [CLI Reference](cli.md) | Command-line interface |
| [Operators Table](operators-table.md) | All 120+ operators |
| [Error Messages](error-messages.md) | Error codes and solutions |
| [Configuration](configuration.md) | Config file reference |
| [Environment Variables](environment-variables.md) | Environment settings |
| [Exit Codes](exit-codes.md) | Process exit codes |

## CLI Quick Reference

```bash
# Run a strategy
sigc run strategy.sig

# Validate syntax
sigc check strategy.sig

# Start daemon
sigc daemon --config sigc.yaml

# Get help
sigc --help
sigc run --help
```

[Full CLI Reference →](cli.md)

## Common Operators

### Time-Series

| Operator | Description |
|----------|-------------|
| `lag(x, n)` | Shift values back n periods |
| `ret(x, n)` | N-period return |
| `rolling_mean(x, n)` | Rolling average |
| `rolling_std(x, n)` | Rolling standard deviation |
| `ema(x, n)` | Exponential moving average |

### Cross-Sectional

| Operator | Description |
|----------|-------------|
| `zscore(x)` | Cross-sectional z-score |
| `rank(x)` | Cross-sectional rank (0-1) |
| `winsor(x, p)` | Winsorize at percentile p |
| `neutralize(x, by)` | Group neutralization |

### Portfolio

| Operator | Description |
|----------|-------------|
| `long_short(top, bottom)` | Long-short weights |
| `clip(x, lo, hi)` | Bound values |

[All Operators →](operators-table.md)

## File Extensions

| Extension | Type |
|-----------|------|
| `.sig` | sigc strategy file |
| `.yaml` | Configuration file |
| `.parquet` | Data file (recommended) |
| `.csv` | Data file (simple) |

## Data Types

| Type | Description | Example |
|------|-------------|---------|
| `Date` | Trading date | `2024-01-15` |
| `Symbol` | Asset identifier | `AAPL` |
| `Numeric` | Numbers | `185.64` |
| `String` | Text | `"Technology"` |
| `Boolean` | True/false | `true` |

## Configuration Quick Reference

```yaml
# sigc.yaml minimal
mode: production

data:
  source: s3://bucket/prices/
  format: parquet

output:
  type: alpaca
  alpaca:
    paper: true

schedule:
  jobs:
    - name: rebalance
      cron: "0 9 * * 1-5"
      strategy: momentum
```

[Full Configuration →](configuration.md)

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ALPACA_API_KEY` | For trading | Alpaca API key |
| `ALPACA_API_SECRET` | For trading | Alpaca API secret |
| `SIGC_CONFIG` | No | Config file path |
| `SIGC_LOG_LEVEL` | No | Log level (debug, info, warn, error) |

[All Variables →](environment-variables.md)

## Common Errors

| Error | Solution |
|-------|----------|
| `undefined variable` | Check variable name spelling |
| `type mismatch` | Verify operator input types |
| `missing emit` | Add `emit` statement to signal |
| `parse error` | Check syntax near error line |

[All Errors →](error-messages.md)

## Keyboard Shortcuts (VSCode)

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+R` | Run strategy |
| `Ctrl+Space` | Trigger completion |
| `F12` | Go to definition |
| `Shift+F12` | Find references |

## Next Steps

- [CLI Reference](cli.md) - Full command reference
- [Operators Table](operators-table.md) - Complete operator list
- [Getting Started](../getting-started/index.md) - If you're new
