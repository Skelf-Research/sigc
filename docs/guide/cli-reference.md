# CLI Reference

Complete reference for all sigc commands.

## Global Options

```bash
sigc [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable debug logging |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

## Commands

### compile

Compile a .sig file to IR without executing.

```bash
sigc compile <INPUT> [--emit <OUTPUT>]
```

**Arguments:**
- `INPUT` - Path to .sig file

**Options:**
- `--emit <OUTPUT>` - Save compiled IR to file

**Example:**
```bash
sigc compile strategy.sig
sigc compile strategy.sig --emit strategy.ir
```

**Output:**
```
INFO sigc: sigc v0.1.0
INFO sig_compiler: Parsing source
INFO sig_compiler: Parsed 1 data, 2 params, 1 signals, 1 portfolios
INFO sig_compiler: Lowered to 7 IR nodes
INFO sigc: Compilation complete: 7 nodes
```

---

### run

Compile and execute a signal, running a backtest.

```bash
sigc run <INPUT> [--output <OUTPUT>]
```

**Arguments:**
- `INPUT` - Path to .sig file

**Options:**
- `--output <OUTPUT>` - Save results to file (.json or .csv)

**Example:**
```bash
sigc run strategy.sig
sigc run strategy.sig --output results.json
sigc run strategy.sig --output metrics.csv
```

**Output:**
```
=== Backtest Results ===
Total Return:         15.23%
Annualized Return:    15.23%
Sharpe Ratio:          1.45
Max Drawdown:          8.12%
Turnover:            312.00%
```

---

### explain

Show detailed IR breakdown for debugging.

```bash
sigc explain <INPUT>
```

**Example:**
```bash
sigc explain strategy.sig
```

**Output:**
```
=== IR Explanation ===
Source: strategy.sig
Nodes:  7
Outputs: 1

Node Graph:
  #0: Ret [#2] -> Float64
  #1: Constant [] -> Float64
  #2: Input [] -> Float64
  #3: Zscore [#0] -> Float64
  #4: Winsor [#3] -> Float64
  ...

Outputs: [4]
```

---

### diff

Compare two signals or backtest results.

```bash
sigc diff <A> <B>
```

**Arguments:**
- `A` - First .sig file or .json result
- `B` - Second .sig file or .json result

**Example:**
```bash
sigc diff momentum.sig meanrev.sig
sigc diff results_v1.json results_v2.json
```

**Output:**
```
=== Backtest Comparison ===
A: momentum.sig
B: meanrev.sig

Metric               A            B        Delta
----------------------------------------------------------
Total Return       12.45%       8.32%      -4.13%
Ann. Return        12.45%       8.32%      -4.13%
Sharpe Ratio        1.23        0.89       -0.34
Max Drawdown        8.76%      12.34%      +3.58%
Turnover          245.00%     189.00%     -56.00%
```

---

### cache

Manage the compilation cache.

```bash
sigc cache <SUBCOMMAND>
```

#### cache stats

Show cache statistics.

```bash
sigc cache stats
```

**Output:**
```
Cache Statistics:
  Location: /home/user/.cache/sigc
  Entries:  42
```

#### cache verify

Verify cache integrity.

```bash
sigc cache verify
```

#### cache clear

Delete all cached artifacts.

```bash
sigc cache clear
```

---

### daemon

Start the daemon server for persistent service.

```bash
sigc daemon [--listen <ADDR>]
```

**Options:**
- `--listen <ADDR>` - Listen address (default: `tcp://127.0.0.1:7240`)

**Example:**
```bash
sigc daemon
sigc daemon --listen tcp://0.0.0.0:7240
```

---

### request

Send commands to a running daemon.

```bash
sigc request [--addr <ADDR>] <SUBCOMMAND>
```

**Options:**
- `--addr <ADDR>` - Daemon address (default: `tcp://127.0.0.1:7240`)

#### request ping

Check if daemon is responsive.

```bash
sigc request ping
```

#### request status

Get daemon status.

```bash
sigc request status
```

**Output:**
```
Daemon Status:
  Version:  0.1.0
  Uptime:   3600s
  Requests: 42
```

#### request compile

Compile via daemon.

```bash
sigc request compile <INPUT>
```

#### request run

Run backtest via daemon.

```bash
sigc request run <INPUT>
```

#### request shutdown

Gracefully shutdown daemon.

```bash
sigc request shutdown
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SIGC_CACHE_DIR` | Cache directory | `~/.cache/sigc` |
| `SIGC_LOG_LEVEL` | Log level (trace/debug/info/warn/error) | `info` |
| `AWS_ACCESS_KEY_ID` | For S3 data sources | - |
| `AWS_SECRET_ACCESS_KEY` | For S3 data sources | - |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Parse error |
| 3 | Type error |
| 4 | Runtime error |

## Examples

### Development Workflow

```bash
# Edit, compile, iterate
sigc compile strategy.sig

# Run when ready
sigc run strategy.sig

# Compare versions
sigc diff strategy_v1.sig strategy_v2.sig
```

### Production Workflow

```bash
# Start daemon
sigc daemon &

# Send requests
sigc request run strategy.sig

# Monitor
sigc request status
```

### Debugging

```bash
# Verbose logging
sigc -v run strategy.sig

# Inspect IR
sigc explain strategy.sig

# Check cache
sigc cache stats
```
