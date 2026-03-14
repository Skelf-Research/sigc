# Chapter 7: Going to Production

Deploy strategies from research to live trading.

## The Production Journey

```
Research → Validation → Paper Trading → Live Trading → Monitoring
```

Each stage requires different considerations.

## Pre-Production Checklist

### Strategy Validation

- [ ] Walk-forward Sharpe > 0.5
- [ ] Degradation < 50%
- [ ] Positive after realistic costs
- [ ] Stable parameters
- [ ] Reasonable turnover

### Infrastructure

- [ ] Data pipeline operational
- [ ] Broker connection tested
- [ ] Alerting configured
- [ ] Monitoring in place
- [ ] Backup procedures documented

### Risk Controls

- [ ] Position limits set
- [ ] Exposure limits configured
- [ ] Circuit breakers enabled
- [ ] Maximum loss defined

## From Research to Production

### Research Code

```sig
// research/momentum.sig
data:
  source = "historical_prices.parquet"

params:
  lookback: range(40, 100, 20)  // Optimize

signal momentum:
  emit zscore(ret(prices, lookback))

portfolio research:
  weights = rank(momentum).long_short(top=0.2, bottom=0.2)
  backtest walk_forward(...) from 2010-01-01 to 2024-12-31
```

### Production Code

```sig
// production/momentum_live.sig
config: "config/production.yaml"

data:
  source = "s3://mybucket/live_prices/"
  format = parquet

// Fixed parameters (from optimization)
signal momentum:
  emit zscore(ret(prices, 60))

portfolio live:
  weights = rank(momentum).long_short(
    top = 0.2,
    bottom = 0.2,
    cap = 0.03
  )

  constraints:
    gross_exposure = 2.0
    net_exposure = 0.0
    max_sector = 0.20
    max_position = 0.03

  costs = tc.bps(10)
```

### Key Differences

| Aspect | Research | Production |
|--------|----------|------------|
| Parameters | Optimized | Fixed |
| Data Source | Historical | Live feed |
| Execution | Simulated | Real orders |
| Monitoring | Manual | Automated |
| Error Handling | None | Comprehensive |

## Production Configuration

### Main Config File

```yaml
# config/production.yaml

# Environment
environment: production
log_level: info

# Data
data:
  source: "s3://mybucket/prices/"
  refresh_interval: 1h
  cache_dir: "/var/cache/sigc"

# Strategy parameters
params:
  lookback: 60
  top_pct: 0.20
  position_cap: 0.03

# Risk limits
risk:
  max_position: 0.03
  max_sector: 0.20
  max_daily_loss: 0.03
  max_drawdown: 0.15
  gross_exposure: 2.0
  net_exposure_range: [-0.1, 0.1]

# Transaction costs
costs:
  commission_bps: 5
  spread_bps: 5
  market_impact_bps: 2

# Execution
execution:
  broker: alpaca
  order_type: limit
  limit_offset_bps: 5
  max_order_size: 0.02
```

## Daemon Mode

### Setup

```yaml
# config/daemon.yaml
daemon:
  enabled: true
  pid_file: "/var/run/sigc/strategy.pid"
  log_file: "/var/log/sigc/strategy.log"

  schedule:
    # Compute weights before market close
    - cron: "30 15 * * 1-5"
      action: compute_weights
      timezone: "America/New_York"

    # Execute trades
    - cron: "55 15 * * 1-5"
      action: execute_trades
      timezone: "America/New_York"
```

### Start Daemon

```bash
# Start in paper mode first
sigc daemon start \
  --strategy production/momentum_live.sig \
  --config config/production.yaml \
  --paper

# Check status
sigc daemon status
```

## Paper Trading

### Why Paper Trade First?

1. Verify execution logic
2. Test broker integration
3. Validate data pipeline
4. Check alerting
5. Build confidence

### Duration

Minimum 2-4 weeks of paper trading before going live.

### What to Check

- [ ] Trades execute correctly
- [ ] Position sizes match targets
- [ ] Transaction costs reasonable
- [ ] No unexpected errors
- [ ] Performance tracks expectations

## Going Live

### Final Pre-Live Checklist

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

### Start Live Trading

```yaml
# Update config
execution:
  paper: false  # Switch to live
```

```bash
# Start live
sigc daemon start \
  --strategy production/momentum_live.sig \
  --config config/production.yaml \
  --live
```

## Monitoring

### Key Metrics

| Metric | Monitor | Alert |
|--------|---------|-------|
| Daily P&L | Continuous | >2% loss |
| Positions | Every trade | Divergence >10% |
| Exposure | Hourly | Outside limits |
| Data freshness | Hourly | >2h stale |
| System health | Continuous | Any failure |

### Dashboard Example

```
MOMENTUM STRATEGY - LIVE
========================
Status: RUNNING

Positions:
  Long:  50 stocks, $500,000
  Short: 50 stocks, $500,000
  Net:   $0 (0.0%)
  Gross: $1,000,000 (200%)

Performance (Today):
  P&L: +$1,234 (+0.12%)
  vs SPY: +0.08%

Performance (MTD):
  P&L: +$8,456 (+0.85%)
  Sharpe: 1.2

Risk Metrics:
  Daily VaR: $15,000
  Current DD: -2.3%
  Beta: 0.05
```

### Alerting

```yaml
alerting:
  slack:
    webhook: ${SLACK_WEBHOOK}
    channel: "#trading"

  rules:
    - name: "Large Loss"
      condition: daily_loss > 0.02
      severity: warning

    - name: "Circuit Breaker"
      condition: daily_loss > 0.03
      severity: critical
      action: halt_trading

    - name: "Trade Failure"
      condition: trade_failed
      severity: critical
```

## Daily Operations

### Morning Routine

1. Check overnight status
2. Verify positions match targets
3. Review any alerts
4. Check data freshness
5. Review broker account

```bash
# Morning checks
sigc status
sigc positions --compare target
sigc alerts --last 24h
```

### End of Day

1. Review today's trades
2. Check P&L
3. Verify reconciliation
4. Note any issues

```bash
# EOD checks
sigc trades --today
sigc performance --today
sigc reconcile
```

### Weekly Review

1. Performance analysis
2. Compare to backtest
3. Check parameter drift
4. Review any incidents

```bash
# Weekly report
sigc report --period 1w --output reports/weekly.html
```

## Error Handling

### Common Issues

**Data Stale:**
```
Alert: Data not updated in 2 hours
Action: Check data pipeline, S3 access
```

**Trade Rejected:**
```
Alert: Order rejected - insufficient buying power
Action: Check account balance, reduce position sizes
```

**Connection Lost:**
```
Alert: Broker connection lost
Action: Auto-retry, failover to backup
```

### Recovery Procedures

```yaml
recovery:
  auto_retry:
    max_attempts: 3
    delay_seconds: 60

  failover:
    enabled: true
    backup_broker: "backup_config.yaml"

  manual_intervention:
    - circuit_breaker_triggered
    - large_position_divergence
```

## Best Practices

### 1. Start Small

```yaml
# Week 1-2
max_position: 0.01
gross_exposure: 1.0

# Week 3-4
max_position: 0.02
gross_exposure: 1.5

# Month 2+
max_position: 0.03
gross_exposure: 2.0
```

### 2. Monitor Everything

Log all:
- Trades executed
- Signals computed
- Data received
- Errors encountered

### 3. Have Kill Switches

```bash
# Emergency stop
sigc daemon stop --immediate

# Reduce exposure
sigc reduce-exposure --target 0.5
```

### 4. Document Everything

Maintain:
- Run books
- Incident reports
- Parameter change log
- Performance records

### 5. Regular Reviews

- Daily: Quick health check
- Weekly: Performance review
- Monthly: Deep analysis
- Quarterly: Strategy review

## Exercises

1. Create a production configuration file
2. Set up paper trading for your strategy
3. Configure alerting for key metrics
4. Create morning and EOD checklists

## Next Chapter

Continue to [Chapter 8: Advanced Analytics](08-advanced-analytics.md) for factor models and attribution.
