# Production Deployment

Deploy sigc strategies for live trading.

## Overview

sigc provides production-grade features:

- **Daemon Mode**: Long-running signal computation
- **Safety Systems**: Pre-trade checks and circuit breakers
- **Alerting**: Notifications for anomalies and errors
- **Monitoring**: Prometheus metrics and health checks
- **Scheduling**: Cron-based signal generation
- **Audit Logging**: Complete audit trail

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Production sigc                          │
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  Scheduler  │───▶│   Daemon    │───▶│   Output    │    │
│  │  (cron)     │    │  (compute)  │    │  (broker)   │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│         │                  │                  │            │
│         ▼                  ▼                  ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   Alerts    │    │  Monitoring │    │   Logging   │    │
│  │  (Slack)    │    │(Prometheus) │    │   (JSON)    │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Create Production Config

```yaml
# sigc.prod.yaml
mode: production

data:
  source: s3://bucket/prices/
  cache_ttl: 3600

signals:
  - momentum_strategy.sig

schedule:
  cron: "0 9 * * 1-5"  # 9 AM weekdays

output:
  type: alpaca
  paper: false

safety:
  max_position: 0.05
  max_daily_loss: 0.02
  max_drawdown: 0.10

alerting:
  slack:
    webhook: ${SLACK_WEBHOOK}
    channel: "#trading"
```

### 2. Start Daemon

```bash
sigc daemon --config sigc.prod.yaml
```

### 3. Monitor

```bash
# Check status
sigc status

# View logs
sigc logs --follow

# Check health
curl http://localhost:9090/health
```

## Production Checklist

### Before Going Live

- [ ] Backtest with realistic costs
- [ ] Walk-forward validation complete
- [ ] Paper trading for 30+ days
- [ ] Safety limits configured
- [ ] Alerting tested
- [ ] Monitoring dashboards ready
- [ ] Runbook documented
- [ ] Rollback plan ready

### Data

- [ ] Data source reliable (SLA)
- [ ] Backup data source configured
- [ ] Corporate actions handled
- [ ] Missing data alerts

### Infrastructure

- [ ] Server provisioned
- [ ] Network connectivity tested
- [ ] Secrets managed securely
- [ ] Backups configured
- [ ] Disaster recovery tested

## Production vs Development

| Aspect | Development | Production |
|--------|-------------|------------|
| Data | Local files | S3/Database |
| Caching | Optional | Required |
| Safety | Minimal | Strict |
| Logging | Console | Structured JSON |
| Alerts | None | Slack/PagerDuty |
| Monitoring | Optional | Required |
| Error handling | Fail fast | Graceful degradation |

## Key Concepts

### Idempotency

Same input produces same output:

```bash
# Running twice produces identical results
sigc run strategy.sig --date 2024-01-15
sigc run strategy.sig --date 2024-01-15  # Same output
```

### Determinism

Results are reproducible:

```bash
# Same version + same data = same results
sigc run strategy.sig --seed 42
```

### Graceful Degradation

Handle failures gracefully:

```yaml
safety:
  on_data_error: use_cache      # Use cached data
  on_compute_error: hold        # Keep current positions
  on_execution_error: cancel    # Cancel pending orders
```

## Best Practices

### 1. Paper Trade First

```yaml
output:
  type: alpaca
  paper: true  # Paper trading mode
```

Run paper trading for at least 30 days.

### 2. Start Small

```yaml
portfolio:
  initial_capital: 10000  # Start small
  max_position_pct: 0.02  # Very conservative
```

### 3. Set Conservative Limits

```yaml
safety:
  max_daily_loss: 0.01      # 1% daily loss limit
  max_position: 0.03        # 3% max per position
  max_drawdown: 0.05        # 5% drawdown stops trading
```

### 4. Monitor Everything

```yaml
monitoring:
  prometheus:
    enabled: true
    port: 9090
  health_check:
    enabled: true
    port: 8080
```

### 5. Test Alerts

```bash
sigc alert test --channel slack
```

## Documentation Index

- [Daemon Mode](daemon-mode.md) - Running sigc as a service
- [Configuration](configuration.md) - Production configuration
- [Safety Systems](safety-systems.md) - Pre-trade checks and circuit breakers
- [Alerting](alerting.md) - Notifications and escalations
- [Monitoring](monitoring.md) - Metrics and dashboards
- [Scheduling](scheduling.md) - Automated signal generation
- [Audit Logging](audit-logging.md) - Compliance logging
- [Docker](docker.md) - Container deployment

## Next Steps

- [Daemon Mode](daemon-mode.md) - Set up long-running service
- [Safety Systems](safety-systems.md) - Configure safety limits
- [Docker](docker.md) - Container deployment
