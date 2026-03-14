# Configuration

Complete configuration reference for sigc.

## Configuration Files

### Location Priority

1. Command line: `--config path/to/config.yaml`
2. Environment: `$SIGC_CONFIG`
3. Current directory: `./sigc.yaml`
4. Home directory: `~/.sigc/config.yaml`
5. System: `/etc/sigc/config.yaml`

### Format

YAML format with environment variable substitution:

```yaml
# Variables from environment
api_key: ${ALPACA_API_KEY}

# With default value
region: ${AWS_REGION:-us-east-1}
```

## Full Configuration Reference

```yaml
# sigc.yaml - Complete Configuration

#=============================================================================
# Mode: development | staging | production
#=============================================================================
mode: production

#=============================================================================
# Data Sources
#=============================================================================
data:
  # Primary data source
  source: s3://my-bucket/prices/
  format: parquet

  # Backup data source (used if primary fails)
  backup:
    source: postgresql://readonly@db.example.com/marketdb
    query: "SELECT date, ticker, close, volume FROM prices"

  # Caching
  cache:
    enabled: true
    directory: /var/cache/sigc
    ttl_hours: 24
    max_size_gb: 10

  # Data validation
  validation:
    max_missing_pct: 0.05
    max_stale_hours: 24
    require_recent: true

#=============================================================================
# Strategies
#=============================================================================
strategies:
  - name: momentum
    file: strategies/momentum.sig
    enabled: true
    schedule: "0 9 * * 1-5"  # 9 AM weekdays

  - name: value
    file: strategies/value.sig
    enabled: true
    schedule: "0 9 * * 1-5"

#=============================================================================
# Output / Execution
#=============================================================================
output:
  type: alpaca

  alpaca:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}
    base_url: https://api.alpaca.markets
    paper: false

  # Order settings
  orders:
    type: limit            # market | limit | adaptive
    limit_offset_pct: 0.1  # 0.1% better than market
    time_in_force: day
    timeout_seconds: 60

  # Execution algorithm
  execution:
    algorithm: twap        # twap | vwap | pov | arrival
    duration_minutes: 30
    max_participation: 0.05

#=============================================================================
# Safety Systems
#=============================================================================
safety:
  # Position limits
  max_position: 0.05          # 5% max per position
  max_sector: 0.25            # 25% max per sector
  max_gross_exposure: 2.0     # 200% max gross
  max_net_exposure: 0.1       # 10% max net

  # Loss limits
  max_daily_loss: 0.02        # 2% daily loss limit
  max_weekly_loss: 0.05       # 5% weekly loss limit
  max_drawdown: 0.10          # 10% max drawdown

  # Trading limits
  max_order_size: 0.03        # 3% max single order
  max_daily_orders: 100       # Max orders per day
  max_daily_turnover: 0.50    # 50% max daily turnover

  # Circuit breakers
  circuit_breakers:
    enabled: true
    triggers:
      - type: daily_loss
        threshold: 0.02
        action: halt_trading

      - type: drawdown
        threshold: 0.10
        action: halt_trading

      - type: consecutive_losses
        count: 5
        action: alert

  # Error handling
  on_data_error: use_cache    # use_cache | hold | halt
  on_compute_error: hold      # hold | halt | alert
  on_execution_error: cancel  # cancel | retry | halt

#=============================================================================
# Alerting
#=============================================================================
alerting:
  # Slack integration
  slack:
    enabled: true
    webhook: ${SLACK_WEBHOOK}
    channel: "#trading-alerts"
    mention: "@oncall"

  # PagerDuty integration
  pagerduty:
    enabled: false
    api_key: ${PAGERDUTY_API_KEY}
    service_id: ${PAGERDUTY_SERVICE_ID}

  # Email
  email:
    enabled: true
    smtp_host: smtp.gmail.com
    smtp_port: 587
    username: ${SMTP_USERNAME}
    password: ${SMTP_PASSWORD}
    from: alerts@example.com
    to:
      - team@example.com
      - oncall@example.com

  # Alert rules
  rules:
    - name: large_loss
      condition: "daily_pnl < -0.01"
      severity: high
      channels: [slack, pagerduty]

    - name: position_limit
      condition: "max_position > 0.04"
      severity: medium
      channels: [slack]

    - name: data_stale
      condition: "data_age > 2h"
      severity: high
      channels: [slack, email]

#=============================================================================
# Monitoring
#=============================================================================
monitoring:
  # Prometheus metrics
  prometheus:
    enabled: true
    port: 9090
    path: /metrics

  # Health checks
  health:
    enabled: true
    port: 8080
    checks:
      - data_freshness
      - broker_connection
      - memory_usage
      - disk_space

  # Logging
  logging:
    level: info                # debug | info | warn | error
    format: json               # json | text
    file: /var/log/sigc/sigc.log
    max_size_mb: 100
    max_files: 10
    compress: true

#=============================================================================
# Daemon Settings
#=============================================================================
daemon:
  # RPC interface
  rpc:
    address: "127.0.0.1:5555"
    auth: token
    token: ${SIGC_RPC_TOKEN}

  # Process settings
  pid_file: /var/run/sigc.pid
  work_dir: /var/lib/sigc

  # Resource limits
  limits:
    max_memory_mb: 4096
    max_cpu_pct: 80
    worker_threads: 4

  # Recovery
  recovery:
    enabled: true
    max_restarts: 3
    restart_delay_seconds: 10

  # Shutdown
  shutdown:
    timeout_seconds: 60
    save_state: true

#=============================================================================
# Scheduling
#=============================================================================
schedule:
  timezone: America/New_York

  # Trading calendar
  calendar:
    type: nyse                # nyse | nasdaq | custom
    holidays: auto            # auto | custom file

  # Schedule rules
  rules:
    - name: morning_rebalance
      cron: "0 9 * * 1-5"
      strategy: momentum
      if: market_open

    - name: eod_check
      cron: "0 15 * * 1-5"
      action: check_positions

#=============================================================================
# Audit Logging
#=============================================================================
audit:
  enabled: true
  file: /var/log/sigc/audit.log
  format: json

  # What to log
  log:
    - signals
    - orders
    - fills
    - positions
    - config_changes

  # Retention
  retention_days: 365

  # External sink
  sink:
    type: s3
    bucket: my-bucket
    prefix: audit/
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `SIGC_RPC_TOKEN` | RPC authentication token |
| `ALPACA_API_KEY` | Alpaca API key |
| `ALPACA_API_SECRET` | Alpaca API secret |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `SIGC_CONFIG` | Config file path | `./sigc.yaml` |
| `SIGC_LOG_LEVEL` | Log level | `info` |
| `SIGC_DATA_DIR` | Data directory | `/var/lib/sigc` |
| `SIGC_CACHE_DIR` | Cache directory | `/var/cache/sigc` |
| `AWS_ACCESS_KEY_ID` | AWS access key | - |
| `AWS_SECRET_ACCESS_KEY` | AWS secret | - |
| `AWS_REGION` | AWS region | `us-east-1` |

## Configuration Profiles

### Development

```yaml
# sigc.dev.yaml
mode: development

data:
  source: ./data/prices.csv
  format: csv

output:
  type: mock

safety:
  enabled: false

monitoring:
  logging:
    level: debug
    format: text
```

### Staging

```yaml
# sigc.staging.yaml
mode: staging

output:
  type: alpaca
  alpaca:
    paper: true  # Paper trading

safety:
  enabled: true
  # Same limits as production
```

### Production

```yaml
# sigc.prod.yaml
mode: production

output:
  type: alpaca
  alpaca:
    paper: false  # Live trading

safety:
  enabled: true
  max_daily_loss: 0.02

alerting:
  slack:
    enabled: true
```

## Validation

Validate configuration:

```bash
sigc config validate --config sigc.yaml
```

Output:

```
Configuration validation: PASSED

Warnings:
  - alerting.pagerduty.enabled is false (no escalation for critical alerts)
  - safety.max_position (5%) may be too high for concentrated strategies

Recommendations:
  - Consider enabling audit logging for compliance
  - Set up backup data source for resilience
```

## Secrets Management

### Environment Variables

```bash
export ALPACA_API_KEY=your_key
sigc daemon --config sigc.yaml
```

### AWS Secrets Manager

```yaml
secrets:
  provider: aws-secrets-manager
  secret_name: sigc/production
```

### HashiCorp Vault

```yaml
secrets:
  provider: vault
  address: https://vault.example.com
  path: secret/sigc
```

## Best Practices

### 1. Use Separate Configs per Environment

```
config/
├── sigc.dev.yaml
├── sigc.staging.yaml
└── sigc.prod.yaml
```

### 2. Never Commit Secrets

```yaml
# Good
api_key: ${ALPACA_API_KEY}

# Bad
api_key: pk_live_abc123  # Never do this!
```

### 3. Validate Before Deploy

```bash
sigc config validate --config sigc.prod.yaml --strict
```

### 4. Version Control Configs

Track configuration changes in git.

### 5. Document Changes

Add comments explaining non-obvious settings.

## Next Steps

- [Safety Systems](safety-systems.md) - Pre-trade checks
- [Alerting](alerting.md) - Notification setup
- [Daemon Mode](daemon-mode.md) - Running as service
