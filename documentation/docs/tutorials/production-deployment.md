# Tutorial: Production Deployment

Deploy your validated strategy to live trading.

## Overview

This tutorial covers:

1. Pre-deployment checklist
2. Configuration for production
3. Setting up daemon mode
4. Connecting to brokers
5. Monitoring and alerting
6. Handling errors

## Pre-Deployment Checklist

### Strategy Validation

```bash
# 1. Run walk-forward validation
sigc run strategy.sig --walk-forward --validate

# 2. Check parameter stability
sigc run strategy.sig --walk-forward --stability-analysis

# 3. Verify recent performance
sigc run strategy.sig --from 2023-01-01

# 4. Test with realistic costs
sigc run strategy.sig --costs "tc.bps(15)"
```

### Required Results

- [ ] Out-of-sample Sharpe > 0.5
- [ ] Degradation < 50%
- [ ] Stable parameters across periods
- [ ] Positive return after costs
- [ ] Max drawdown within risk tolerance

## Step 1: Production Configuration

### Create Config File

```yaml
# config/production.yaml

# Data source
data:
  source: "s3://mybucket/prices/"
  format: parquet
  refresh_interval: 1h

# Strategy parameters (from walk-forward optimization)
params:
  lookback: 60
  top_pct: 0.20
  position_cap: 0.03

# Risk limits
risk:
  max_position: 0.03
  max_sector: 0.20
  max_drawdown: 0.15
  gross_exposure: 2.0
  net_exposure_range: [-0.1, 0.1]

# Transaction costs
costs:
  commission_bps: 5
  spread_bps: 5
  market_impact_bps: 2

# Safety systems
safety:
  pre_trade_checks: true
  max_order_size: 0.02
  max_daily_turnover: 0.20
  circuit_breaker:
    daily_loss_limit: 0.03
    position_divergence: 0.10

# Execution
execution:
  broker: alpaca
  paper_trading: true  # Start with paper
  order_type: limit
  limit_offset_bps: 5
```

## Step 2: Strategy File for Production

### Separate Production Strategy

```sig
// strategies/momentum_prod.sig

// Import production config
config: "config/production.yaml"

data:
  source = config.data.source
  format = parquet

// Use fixed parameters (from optimization)
signal momentum:
  emit zscore(ret(prices, 60))

signal value:
  emit zscore(book_to_market)

signal combined:
  emit neutralize(0.6 * momentum + 0.4 * value, by=sectors)

portfolio production:
  weights = rank(combined).long_short(
    top = 0.20,
    bottom = 0.20,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure: [-0.1, 0.1]
    max_sector = 0.20
    max_position = 0.03
    max_turnover = 0.20

  costs = tc.bps(12)

  // No backtest - this is live
```

## Step 3: Set Up Daemon Mode

### Daemon Configuration

```yaml
# config/daemon.yaml

daemon:
  enabled: true
  pid_file: "/var/run/sigc/momentum.pid"
  log_file: "/var/log/sigc/momentum.log"

  schedule:
    # Generate weights at 3:30 PM (before market close)
    - cron: "30 15 * * 1-5"
      action: compute_weights
      timezone: "America/New_York"

    # Execute at 3:55 PM
    - cron: "55 15 * * 1-5"
      action: execute_trades
      timezone: "America/New_York"

  # Retry configuration
  retry:
    max_attempts: 3
    delay_seconds: 60
```

### Start Daemon

```bash
# Start in paper trading mode first
sigc daemon start --config config/production.yaml --paper

# Check status
sigc daemon status

# View logs
tail -f /var/log/sigc/momentum.log
```

## Step 4: Connect to Broker

### Alpaca Configuration

```yaml
# config/broker.yaml

broker:
  name: alpaca
  paper: true  # Paper trading

  credentials:
    api_key: ${ALPACA_API_KEY}
    api_secret: ${ALPACA_API_SECRET}

  settings:
    base_url: "https://paper-api.alpaca.markets"
    data_url: "https://data.alpaca.markets"
```

### Set Environment Variables

```bash
export ALPACA_API_KEY="your_api_key"
export ALPACA_API_SECRET="your_api_secret"
```

### Test Connection

```bash
sigc broker test --config config/broker.yaml
```

```
Broker Connection Test: Alpaca (Paper)
======================================
Connection: ✓ Success
Account Status: ACTIVE
Buying Power: $100,000.00
Equity: $100,000.00
```

## Step 5: Paper Trading Period

### Run Paper Trading

```bash
# Start paper trading daemon
sigc daemon start \
  --strategy strategies/momentum_prod.sig \
  --config config/production.yaml \
  --broker config/broker.yaml \
  --paper
```

### Monitor Paper Trading

```bash
# Check current positions
sigc positions

# View today's trades
sigc trades --today

# Performance summary
sigc performance --period 1w
```

### Paper Trading Checklist

Run paper trading for at least 2-4 weeks:

- [ ] Trades execute correctly
- [ ] Position sizes match targets
- [ ] Costs are as expected
- [ ] No unexpected errors
- [ ] Performance tracks backtest

## Step 6: Set Up Monitoring

### Prometheus Metrics

```yaml
# config/monitoring.yaml

monitoring:
  prometheus:
    enabled: true
    port: 9090

  metrics:
    - portfolio_value
    - daily_return
    - position_count
    - turnover
    - tracking_error
```

### Alerting Rules

```yaml
# config/alerts.yaml

alerting:
  # Slack notifications
  slack:
    webhook: ${SLACK_WEBHOOK_URL}
    channel: "#trading-alerts"

  # Alert rules
  rules:
    - name: "Large Drawdown"
      condition: drawdown > 0.05
      severity: warning

    - name: "Circuit Breaker"
      condition: daily_loss > 0.03
      severity: critical
      action: halt_trading

    - name: "Execution Failure"
      condition: trade_failed
      severity: critical

    - name: "Data Staleness"
      condition: data_age > 2h
      severity: warning
```

### Enable Alerting

```bash
sigc daemon start \
  --strategy strategies/momentum_prod.sig \
  --config config/production.yaml \
  --alerts config/alerts.yaml
```

## Step 7: Go Live

### Final Checklist

```bash
# Run pre-flight checks
sigc preflight --config config/production.yaml
```

```
Pre-Flight Checks
=================
[✓] Strategy file valid
[✓] Data source accessible
[✓] Broker connection OK
[✓] Account has sufficient capital
[✓] Risk limits configured
[✓] Alerting configured
[✓] Logging enabled

Ready for live trading.
```

### Switch to Live

```yaml
# config/production.yaml
broker:
  paper: false  # Switch to live
```

```bash
# Start live daemon
sigc daemon start \
  --strategy strategies/momentum_prod.sig \
  --config config/production.yaml \
  --live

# Confirm
sigc daemon status
```

## Step 8: Daily Operations

### Morning Checks

```bash
# Check overnight status
sigc status

# Verify positions
sigc positions --compare target

# Check for alerts
sigc alerts --last 24h
```

### End of Day

```bash
# Review trades
sigc trades --today

# Check P&L
sigc performance --today

# Verify reconciliation
sigc reconcile
```

### Weekly Review

```bash
# Performance report
sigc report --period 1w --output reports/weekly.html

# Compare to backtest
sigc compare --backtest --period 1w
```

## Error Handling

### Common Errors

**Trade Rejection:**
```
Error: Trade rejected - insufficient buying power
Action: Check account balance, reduce position sizes
```

**Data Error:**
```
Error: Data source unavailable
Action: Check S3 credentials, data pipeline
```

**Broker Connection:**
```
Error: Broker API timeout
Action: Retry automatically (configured), check broker status
```

### Recovery Procedures

```yaml
# config/recovery.yaml

recovery:
  # Automatic recovery
  auto_retry:
    enabled: true
    max_attempts: 3

  # Manual intervention required
  manual_intervention:
    - circuit_breaker_triggered
    - large_position_divergence
    - broker_account_issue

  # Notification
  notify_on_recovery: true
```

## Complete Production Setup

### Directory Structure

```
trading/
├── config/
│   ├── production.yaml
│   ├── daemon.yaml
│   ├── broker.yaml
│   ├── alerts.yaml
│   └── monitoring.yaml
├── strategies/
│   └── momentum_prod.sig
├── logs/
│   └── (auto-generated)
├── reports/
│   └── (auto-generated)
└── scripts/
    ├── start.sh
    ├── stop.sh
    └── health_check.sh
```

### Start Script

```bash
#!/bin/bash
# scripts/start.sh

# Load environment
source ~/.trading_env

# Start daemon
sigc daemon start \
  --strategy strategies/momentum_prod.sig \
  --config config/production.yaml \
  --broker config/broker.yaml \
  --alerts config/alerts.yaml \
  --monitoring config/monitoring.yaml \
  --live

echo "Trading daemon started"
```

### Health Check Script

```bash
#!/bin/bash
# scripts/health_check.sh

# Check daemon status
STATUS=$(sigc daemon status --json | jq -r '.status')

if [ "$STATUS" != "running" ]; then
  echo "CRITICAL: Daemon not running"
  exit 2
fi

# Check last trade time
LAST_TRADE=$(sigc trades --last --json | jq -r '.timestamp')
# ... additional checks

echo "OK: All systems operational"
exit 0
```

## Best Practices

### 1. Start Small

Begin with small position sizes, increase gradually:

```yaml
# Week 1-2
max_position: 0.01  # 1% max

# Week 3-4
max_position: 0.02  # 2% max

# Month 2+
max_position: 0.03  # 3% max (target)
```

### 2. Monitor Closely Initially

Check multiple times per day during first week.

### 3. Have a Kill Switch

```bash
# Emergency stop
sigc daemon stop --immediate

# Or via script
./scripts/emergency_stop.sh
```

### 4. Keep Backups

```bash
# Backup configuration
cp -r config/ backup/config_$(date +%Y%m%d)/

# Backup state
sigc state export --output backup/state_$(date +%Y%m%d).json
```

### 5. Document Everything

Keep a trading log of:
- Parameter changes
- Incidents
- Performance notes
- Lessons learned

## Next Steps

- [Python Workflow](python-workflow.md) - Advanced analysis
- [Monitoring](../production/monitoring.md) - Detailed monitoring setup
- [Alerting](../production/alerting.md) - Alert configuration
