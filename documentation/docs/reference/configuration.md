# Configuration Reference

Complete reference for sigc configuration options.

## Configuration File

sigc uses YAML configuration files:

```yaml
# config.yaml
data:
  source: "prices.parquet"
  format: parquet

performance:
  parallel:
    enabled: true
    workers: 8

daemon:
  enabled: true
  schedule:
    - cron: "0 16 * * 1-5"
      action: compute_weights
```

## Data Configuration

### Source Options

```yaml
data:
  # Local file
  source: "path/to/data.parquet"

  # S3
  source: "s3://bucket/path/data.parquet"

  # PostgreSQL
  source: "postgres://user:pass@host:5432/db"

  # Yahoo Finance
  source: "yahoo://AAPL,GOOGL,MSFT"

  format: parquet  # csv, parquet, arrow, sql
```

### Column Mapping

```yaml
data:
  columns:
    date: Date
    ticker: Symbol
    close: Numeric as prices
    volume: Numeric as volume
    sector: Category as sectors
```

### Filters

```yaml
data:
  filter:
    - column: market_cap
      op: ">"
      value: 1000000000
    - column: country
      value: "US"
```

## Performance Configuration

### Parallelism

```yaml
performance:
  parallel:
    enabled: true
    workers: auto  # or specific number
    min_batch_size: 100
```

### Memory

```yaml
performance:
  memory:
    max_memory_gb: 16
    target_memory_gb: 12

  memory_map:
    enabled: true
    max_mapped_size_gb: 50
```

### Incremental

```yaml
performance:
  incremental:
    enabled: true
    checkpoint_dir: "./checkpoints"
    checkpoint_interval: 1d
```

## Daemon Configuration

### Basic Daemon

```yaml
daemon:
  enabled: true
  pid_file: "/var/run/sigc/strategy.pid"
  log_file: "/var/log/sigc/strategy.log"
```

### Schedule

```yaml
daemon:
  schedule:
    - cron: "30 15 * * 1-5"
      action: compute_weights
      timezone: "America/New_York"

    - cron: "55 15 * * 1-5"
      action: execute_trades
```

### Actions

| Action | Description |
|--------|-------------|
| `compute_weights` | Calculate target weights |
| `execute_trades` | Send orders to broker |
| `reconcile` | Check positions match |
| `report` | Generate report |

## Risk Configuration

### Limits

```yaml
risk:
  limits:
    max_position: 0.03
    max_sector: 0.20
    gross_exposure: 2.0
    net_exposure: [-0.1, 0.1]
    max_turnover: 0.25
```

### Loss Limits

```yaml
risk:
  loss_limits:
    daily:
      warning: 0.02
      hard: 0.03
    drawdown:
      warning: 0.10
      hard: 0.15
```

## Safety Configuration

### Pre-Trade Checks

```yaml
safety:
  pre_trade_checks:
    enabled: true
    checks:
      - price_reasonableness
      - size_limit
      - liquidity
```

### Circuit Breakers

```yaml
safety:
  circuit_breakers:
    - name: daily_loss
      condition: daily_pnl < -0.03
      action: halt_trading
```

## Broker Configuration

### Alpaca

```yaml
broker:
  name: alpaca
  paper: true

  credentials:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}

  settings:
    base_url: "https://paper-api.alpaca.markets"
```

### Order Settings

```yaml
execution:
  order_type: limit
  limit_offset_bps: 5
  timeout_seconds: 30
  max_retries: 3
```

## Alerting Configuration

### Channels

```yaml
alerting:
  slack:
    webhook: ${SLACK_WEBHOOK}
    channel: "#trading"

  pagerduty:
    service_key: ${PAGERDUTY_KEY}

  email:
    smtp_host: "smtp.gmail.com"
    smtp_port: 587
    from: "alerts@example.com"
    to: ["team@example.com"]
```

### Rules

```yaml
alerting:
  rules:
    - name: "Large Loss"
      condition: daily_loss > 0.02
      severity: warning
      channels: [slack]

    - name: "Circuit Breaker"
      condition: daily_loss > 0.03
      severity: critical
      channels: [slack, pagerduty]
```

## Monitoring Configuration

### Prometheus

```yaml
monitoring:
  prometheus:
    enabled: true
    port: 9090

  metrics:
    - portfolio_value
    - daily_return
    - position_count
```

### Health Checks

```yaml
monitoring:
  health_checks:
    interval: 60s
    checks:
      - daemon_alive
      - broker_connected
      - data_fresh
```

## Logging Configuration

```yaml
logging:
  level: info  # debug, info, warn, error
  format: json
  output: /var/log/sigc/
  rotation:
    max_size_mb: 100
    max_files: 10
  retention: 30d
```

## Environment Variables

Configuration can reference environment variables:

```yaml
broker:
  credentials:
    api_key: ${ALPACA_API_KEY}
```

Set in environment:
```bash
export ALPACA_API_KEY="your_key"
```

## Configuration Precedence

1. CLI arguments (highest)
2. Environment variables
3. Configuration file
4. Default values (lowest)

## Validation

Validate configuration:

```bash
sigc config validate --config config.yaml
```

## See Also

- [Environment Variables](environment-variables.md)
- [CLI Reference](cli.md)
