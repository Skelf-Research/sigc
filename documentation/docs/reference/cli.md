# CLI Reference

Complete command-line interface reference for sigc.

## Global Options

```bash
sigc [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>     Configuration file
  -v, --verbose           Increase verbosity
  -q, --quiet             Suppress output
      --version           Print version
  -h, --help              Print help
```

## Commands

### run

Run a strategy file.

```bash
sigc run [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>` - Strategy file path (.sig)

**Options:**
```bash
  -p, --params <KEY=VALUE>    Override parameters
  -o, --output <FILE>         Output file (csv, json, parquet)
  -d, --date <DATE>           Run for specific date
      --start <DATE>          Override start date
      --end <DATE>            Override end date
      --dry-run               Compute signals without execution
      --report <TYPE>         Generate report (summary, detailed, monthly)
```

**Examples:**
```bash
# Basic run
sigc run strategy.sig

# With parameters
sigc run strategy.sig -p lookback=60 -p top_pct=0.2

# Output to CSV
sigc run strategy.sig -o results.csv

# Detailed report
sigc run strategy.sig --report detailed

# Dry run (no execution)
sigc run strategy.sig --dry-run
```

### check

Validate a strategy file.

```bash
sigc check [OPTIONS] <FILE>
```

**Options:**
```bash
      --strict              Fail on warnings
      --format <FORMAT>     Output format (text, json)
```

**Examples:**
```bash
sigc check strategy.sig
sigc check strategy.sig --strict
sigc check *.sig  # Check all files
```

### daemon

Start the sigc daemon.

```bash
sigc daemon [OPTIONS]
```

**Options:**
```bash
  -c, --config <FILE>       Configuration file
  -p, --port <PORT>         RPC port (default: 5555)
      --pid-file <FILE>     PID file location
      --daemonize           Run in background
      --foreground          Run in foreground (default)
```

**Examples:**
```bash
sigc daemon
sigc daemon --config sigc.prod.yaml
sigc daemon --daemonize --pid-file /var/run/sigc.pid
```

### status

Check daemon status.

```bash
sigc status [OPTIONS]
```

**Options:**
```bash
  -a, --address <ADDR>      Daemon address (default: 127.0.0.1:5555)
      --detailed            Show detailed status
```

### stop

Stop the daemon.

```bash
sigc stop [OPTIONS]
```

**Options:**
```bash
      --force               Force stop
      --timeout <SECONDS>   Wait timeout
```

### pause / resume

Pause or resume trading.

```bash
sigc pause [STRATEGY]
sigc resume [STRATEGY]
```

### reload

Reload configuration.

```bash
sigc reload
```

### convert

Convert between data formats.

```bash
sigc convert [OPTIONS] <INPUT> <OUTPUT>
```

**Examples:**
```bash
sigc convert prices.csv prices.parquet
sigc convert data.json data.parquet
```

### validate

Validate data file.

```bash
sigc validate [OPTIONS] <FILE>
```

**Options:**
```bash
      --check <TYPE>        Check type (missing, outliers, duplicates)
      --strict              Fail on warnings
```

### cache

Manage cache.

```bash
sigc cache <SUBCOMMAND>
```

**Subcommands:**
```bash
sigc cache status           # Show cache status
sigc cache clear            # Clear all cache
sigc cache clear --pattern  # Clear matching entries
```

### alpaca

Alpaca broker commands.

```bash
sigc alpaca <SUBCOMMAND>
```

**Subcommands:**
```bash
sigc alpaca account         # Show account info
sigc alpaca positions       # Show positions
sigc alpaca orders          # Show orders
sigc alpaca cancel <ID>     # Cancel order
sigc alpaca cancel --all    # Cancel all orders
```

### schedule

Manage schedule.

```bash
sigc schedule <SUBCOMMAND>
```

**Subcommands:**
```bash
sigc schedule list          # List scheduled jobs
sigc schedule enable <JOB>  # Enable job
sigc schedule disable <JOB> # Disable job
sigc schedule run <JOB>     # Run job now
sigc schedule history <JOB> # Show job history
```

### alerts

Manage alerts.

```bash
sigc alerts <SUBCOMMAND>
```

**Subcommands:**
```bash
sigc alerts status          # Show active alerts
sigc alerts test            # Test alert channels
sigc alerts ack <ID>        # Acknowledge alert
sigc alerts resolve <ID>    # Resolve alert
```

### audit

Query audit logs.

```bash
sigc audit query [OPTIONS]
```

**Options:**
```bash
      --type <TYPE>         Event type
      --strategy <NAME>     Strategy filter
      --ticker <SYMBOL>     Ticker filter
      --last <DURATION>     Time window (24h, 7d, 30d)
      --output <FILE>       Export to file
```

### safety

Safety system commands.

```bash
sigc safety <SUBCOMMAND>
```

**Subcommands:**
```bash
sigc safety status          # Show safety status
sigc safety halt            # Emergency halt
sigc safety resume          # Resume trading
sigc safety reset <BREAKER> # Reset circuit breaker
sigc safety test <BREAKER>  # Test circuit breaker
```

### reconcile

Reconcile positions.

```bash
sigc reconcile [OPTIONS]
```

**Options:**
```bash
      --fix                 Auto-fix discrepancies
      --report              Generate report
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Command line error |
| 3 | Configuration error |
| 4 | Data error |
| 5 | Execution error |
| 64 | Usage error |
| 65 | Data format error |
| 66 | Cannot open file |
| 78 | Configuration error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SIGC_CONFIG` | Default config file |
| `SIGC_LOG_LEVEL` | Log level (debug, info, warn, error) |
| `SIGC_LOG_FORMAT` | Log format (text, json) |
| `SIGC_DATA_DIR` | Data directory |
| `SIGC_CACHE_DIR` | Cache directory |
| `SIGC_RPC_TOKEN` | RPC authentication token |
| `ALPACA_API_KEY` | Alpaca API key |
| `ALPACA_API_SECRET` | Alpaca API secret |

## Configuration File

Default locations (in order):

1. `--config` flag
2. `$SIGC_CONFIG`
3. `./sigc.yaml`
4. `~/.sigc/config.yaml`
5. `/etc/sigc/config.yaml`

## Shell Completion

### Bash

```bash
sigc completions bash > /etc/bash_completion.d/sigc
```

### Zsh

```bash
sigc completions zsh > ~/.zsh/completions/_sigc
```

### Fish

```bash
sigc completions fish > ~/.config/fish/completions/sigc.fish
```

## Examples

### Daily Workflow

```bash
# Morning: Check status
sigc status

# Run strategy
sigc run momentum.sig

# Check positions
sigc alpaca positions

# Evening: Review
sigc audit query --last 24h
```

### Production Deployment

```bash
# Validate config
sigc config validate --config sigc.prod.yaml

# Start daemon
sigc daemon --config sigc.prod.yaml --daemonize

# Monitor
sigc status --detailed
```

### Debugging

```bash
# Verbose output
sigc run strategy.sig -v

# Check for errors
sigc check strategy.sig --strict

# Validate data
sigc validate prices.parquet
```

## See Also

- [Configuration Reference](configuration.md)
- [Environment Variables](environment-variables.md)
- [Exit Codes](exit-codes.md)
