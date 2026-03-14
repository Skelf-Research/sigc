# Chapter 7: Going Live

Taking a strategy from backtest to production requires careful infrastructure, monitoring, and processes. This chapter covers the operational aspects of running live trading systems.

## Pre-Production Checklist

Before going live, verify:

### Research Quality
- [ ] Walk-forward validation passed
- [ ] Statistical significance (t > 2)
- [ ] Clear economic rationale
- [ ] Robust to parameter changes
- [ ] Works across market regimes

### Operational Readiness
- [ ] Data pipeline tested
- [ ] Execution system ready
- [ ] Monitoring configured
- [ ] Alerts set up
- [ ] Disaster recovery plan
- [ ] Kill switch implemented

### Risk Controls
- [ ] Position limits configured
- [ ] Drawdown limits set
- [ ] Exposure limits defined
- [ ] Manual override capability

## Infrastructure Setup

### System Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ Data Feeds  │ ──▶ │ sigc Engine  │ ──▶ │  Execution  │
└─────────────┘     └──────────────┘     └─────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Monitoring  │
                    └──────────────┘
```

### Configuration

Create a production configuration file:

```toml
# config/production.toml

[database]
host = "db.example.com"
port = 5432
database = "sigc_prod"
pool_size = 10
timeout_seconds = 30

[execution]
parallel_workers = 8
simd_enabled = true
chunk_size = 10000

[data]
cache_dir = "/var/cache/sigc"
default_format = "parquet"

[alerts]
slack_webhook = "${SLACK_WEBHOOK_URL}"
email_recipients = ["team@example.com"]
pagerduty_key = "${PAGERDUTY_KEY}"

[logging]
level = "info"
file = "/var/log/sigc/sigc.log"
max_size_mb = 100
rotation_count = 10
```

Load in code:
```bash
sigc run strategy.sig --config config/production.toml
```

### Environment Variables

Sensitive data should use environment variables:

```bash
export SIGC_DB_HOST=db.example.com
export SIGC_DB_PASSWORD=secret
export SLACK_WEBHOOK_URL=https://hooks.slack.com/...
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Data Pipeline

### Data Sources

Set up reliable data feeds:

```sig
// production_strategy.sig

// Primary data source
data prices = load("postgres://prod/prices", source="primary")

// Backup data source (fallback)
data prices_backup = load("s3://backup/prices.parquet", source="backup")

// Use primary, fallback to backup
data prices = coalesce(prices_primary, prices_backup)
```

### Data Quality Checks

Run validation before using data:

```bash
# Validate data before running strategy
sigc validate data/prices.parquet \
  --check missing \
  --check outliers \
  --check freshness \
  --max-missing 0.01 \
  --freshness-hours 24
```

Configure automatic validation:

```toml
[data_quality]
enabled = true
max_missing_ratio = 0.01
max_stale_hours = 24
outlier_std = 5.0
```

### Corporate Actions

Handle corporate actions automatically:

```bash
sigc run strategy.sig --adjust-corporate-actions
```

Requires corporate actions data:
```sig
data corp_actions = load("corporate_actions.csv")
```

## Scheduling

### Cron-Based Scheduling

Schedule regular runs:

```bash
# Run daily at 5 PM ET (after market close)
0 17 * * 1-5 sigc run strategy.sig --config prod.toml >> /var/log/sigc/daily.log 2>&1

# Run hourly data update
0 * * * * sigc update-data >> /var/log/sigc/data.log 2>&1
```

### Built-in Scheduler

Use sigc's scheduler for more control:

```bash
# Start scheduler daemon
sigc scheduler start

# Add jobs
sigc scheduler add daily-run \
  --command "sigc run strategy.sig" \
  --schedule "0 17 * * 1-5"

sigc scheduler add data-quality \
  --command "sigc validate data/" \
  --schedule "0 6 * * *"

# List jobs
sigc scheduler list

# Check status
sigc scheduler status
```

### Signal Confirmation

Wait for data before running:

```bash
# Only run if data is fresh
sigc run strategy.sig --require-fresh-data --max-stale 1h
```

## Monitoring

### Metrics Collection

sigc exposes metrics for monitoring:

```bash
# Enable metrics endpoint
sigc daemon --metrics-port 9090
```

Available metrics:
- `sigc_runs_total` - Number of strategy runs
- `sigc_run_duration_seconds` - Execution time
- `sigc_portfolio_return` - Current return
- `sigc_portfolio_drawdown` - Current drawdown
- `sigc_data_staleness_seconds` - Data age

### Grafana Dashboard

Import the sigc dashboard:

```bash
sigc export-dashboard grafana > sigc-dashboard.json
```

Key panels:
- Portfolio equity curve
- Daily P&L
- Rolling Sharpe
- Current positions
- Risk metrics
- System health

### Logging

Configure structured logging:

```bash
sigc run strategy.sig --log-level info --log-format json
```

Example log entry:
```json
{
  "timestamp": "2024-01-15T17:00:00Z",
  "level": "info",
  "message": "Strategy run complete",
  "strategy": "momentum",
  "runtime_ms": 1234,
  "positions": 50,
  "turnover": 0.15
}
```

### Audit Trail

Maintain audit log for compliance:

```bash
sigc run strategy.sig --audit-log /var/log/sigc/audit.log
```

Logged events:
- All trades
- Parameter changes
- Manual overrides
- System errors

## Alerting

### Alert Configuration

Set up alerts for critical events:

```bash
# Configure alerts
sigc alerts config \
  --slack-webhook "$SLACK_WEBHOOK_URL" \
  --pagerduty-key "$PAGERDUTY_KEY"

# Set thresholds
sigc alerts set drawdown-warning 0.05
sigc alerts set drawdown-critical 0.10
sigc alerts set data-stale 3600
sigc alerts set run-failed true
```

### Alert Types

| Alert | Level | Trigger |
|-------|-------|---------|
| Data stale | Warning | Data older than threshold |
| Run failed | Critical | Strategy execution error |
| Drawdown warning | Warning | Drawdown > 5% |
| Drawdown critical | Critical | Drawdown > 10% |
| Position limit | Warning | Position exceeds limit |
| Unusual turnover | Warning | Turnover > 2x normal |

### Alert Routing

Route alerts by severity:

```toml
[alerts.routing]
info = ["log"]
warning = ["log", "slack"]
error = ["log", "slack", "email"]
critical = ["log", "slack", "email", "pagerduty"]
```

## Execution

### Order Generation

sigc generates target positions. You need to translate to orders:

```bash
# Generate target positions
sigc run strategy.sig --output-positions positions.csv

# Review positions
cat positions.csv
# symbol,target_weight,current_weight,action,shares
# AAPL,0.05,0.03,BUY,100
# MSFT,0.04,0.06,SELL,50
```

### Execution System Integration

Connect to your execution system:

```python
# Example: Custom execution integration
import pandas as pd

# Load target positions from sigc
positions = pd.read_csv("positions.csv")

# Connect to broker
from broker_api import BrokerClient
client = BrokerClient(api_key=os.environ["BROKER_KEY"])

# Execute trades
for _, pos in positions.iterrows():
    if pos['action'] == 'BUY':
        client.market_order(pos['symbol'], pos['shares'], 'buy')
    elif pos['action'] == 'SELL':
        client.market_order(pos['symbol'], pos['shares'], 'sell')
```

### Transaction Cost Tracking

Track actual vs expected costs:

```bash
sigc compare-costs \
  --expected-file backtest_trades.csv \
  --actual-file executed_trades.csv
```

## Disaster Recovery

### Backup Strategy

Regular backups of:
- Configuration files
- Strategy code
- Historical data
- Position history
- Audit logs

```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d)
tar -czf backup-$DATE.tar.gz \
  config/ \
  strategies/ \
  /var/lib/sigc/data/ \
  /var/log/sigc/audit.log

aws s3 cp backup-$DATE.tar.gz s3://backups/sigc/
```

### Kill Switch

Emergency stop capability:

```bash
# Immediate halt
sigc emergency-stop

# This:
# 1. Stops all scheduled jobs
# 2. Cancels pending orders
# 3. Alerts team
# 4. Logs event
```

### Failover

Maintain standby system:

```bash
# Primary system
sigc daemon --primary --failover-to standby.example.com

# Standby system
sigc daemon --standby --primary-check primary.example.com
```

### Recovery Procedure

1. **Identify issue**: Check logs and alerts
2. **Stop trading**: Kill switch if needed
3. **Assess damage**: Check positions and P&L
4. **Fix issue**: Update code/config
5. **Test fix**: Run in simulation
6. **Resume**: Restart with monitoring
7. **Post-mortem**: Document and improve

## Performance Optimization

### Execution Speed

Optimize for faster runs:

```bash
sigc run strategy.sig \
  --parallel \
  --simd \
  --cache-data
```

### Memory Management

For large datasets:

```bash
sigc run strategy.sig \
  --chunk-size 10000 \
  --streaming
```

### Data Caching

Cache frequently used data:

```toml
[cache]
enabled = true
directory = "/var/cache/sigc"
ttl_hours = 24
max_size_gb = 10
```

## Testing in Production

### Paper Trading

Run with paper money first:

```bash
sigc run strategy.sig --paper-trade --duration 30d
```

Compare paper results to backtest predictions.

### Gradual Rollout

Start small, scale up:

1. **Week 1**: 10% of target capital
2. **Week 2**: 25% if performing as expected
3. **Week 3**: 50%
4. **Week 4**: 100%

Monitor closely at each stage.

### A/B Testing

Test strategy variations:

```bash
# Run both versions
sigc run strategy_v1.sig --allocation 0.5
sigc run strategy_v2.sig --allocation 0.5

# Compare results
sigc compare results_v1/ results_v2/
```

## Compliance

### Record Keeping

Maintain records for:
- All trades
- Order rationale
- Risk metrics
- System changes

```bash
# Export compliance report
sigc report compliance \
  --start 2024-01-01 \
  --end 2024-03-31 \
  --output compliance_q1.pdf
```

### Audit Support

Enable detailed audit logging:

```toml
[audit]
enabled = true
log_file = "/var/log/sigc/audit.log"
log_decisions = true
log_data_access = true
retention_days = 2555  # 7 years
```

## Operational Runbook

### Daily Procedures

**Morning (before market open)**:
1. Check overnight alerts
2. Verify data freshness
3. Review market conditions
4. Check system health

**After market close**:
1. Verify strategy ran
2. Review trades executed
3. Check P&L and risk metrics
4. Investigate any anomalies

### Weekly Procedures

1. Review weekly performance
2. Compare to backtest expectations
3. Check for strategy decay
4. Update documentation

### Monthly Procedures

1. Full performance review
2. Factor exposure analysis
3. Capacity analysis
4. Strategy review meeting

## Example Production Setup

### Complete Configuration

```toml
# production.toml

[general]
environment = "production"
strategy_name = "momentum_quality"
version = "1.2.0"

[database]
host = "${PGHOST}"
port = 5432
database = "sigc_prod"
pool_size = 20

[execution]
parallel_workers = 16
simd_enabled = true
cache_enabled = true

[data]
sources = ["postgres", "s3"]
cache_dir = "/var/cache/sigc"
validate_on_load = true

[risk]
max_position = 0.03
max_sector = 0.25
max_drawdown = 0.15
max_turnover = 4.0

[alerts]
slack_webhook = "${SLACK_WEBHOOK_URL}"
pagerduty_key = "${PAGERDUTY_KEY}"
drawdown_warning = 0.05
drawdown_critical = 0.10

[monitoring]
metrics_port = 9090
health_check_port = 8080

[logging]
level = "info"
format = "json"
file = "/var/log/sigc/sigc.log"

[audit]
enabled = true
log_file = "/var/log/sigc/audit.log"
```

### Startup Script

```bash
#!/bin/bash
# start_production.sh

set -e

# Load environment
source /etc/sigc/env

# Check prerequisites
sigc validate data/ --strict
sigc health-check

# Start daemon
sigc daemon \
  --config /etc/sigc/production.toml \
  --pid-file /var/run/sigc.pid \
  --log-file /var/log/sigc/daemon.log

echo "sigc production daemon started"
```

## Key Takeaways

1. **Automate everything**: Manual processes cause errors
2. **Monitor constantly**: Issues compound quickly
3. **Test thoroughly**: Paper trade before live
4. **Start small**: Scale up gradually
5. **Plan for failure**: Have backups and kill switches
6. **Document everything**: Future you will thank present you

## Final Thoughts

Taking a strategy live is just the beginning. Markets change, edges decay, and systems fail. Success requires:

- **Continuous monitoring**: Watch for degradation
- **Regular updates**: Improve and adapt
- **Discipline**: Stick to the process
- **Humility**: Markets are hard

Good luck with your trading journey!

## Resources

- [Production Features Reference](../advanced/production-features.md)
- [Configuration Reference](../reference/configuration.md)
- [API Reference](../reference/)
- [Example Strategies](../examples/)
