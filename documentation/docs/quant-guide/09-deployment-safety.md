# Chapter 9: Deployment and Safety

Build robust, safe trading systems.

## Safety Philosophy

"Plan for failure" - every component can fail:

- Data sources become unavailable
- Broker connections drop
- Strategies behave unexpectedly
- Markets move violently

Build systems that fail safely.

## Defense in Depth

### Multiple Layers of Protection

```
Layer 1: Pre-Trade Checks
    ↓
Layer 2: Position Limits
    ↓
Layer 3: Exposure Controls
    ↓
Layer 4: Loss Limits
    ↓
Layer 5: Circuit Breakers
    ↓
Layer 6: Kill Switch
```

## Pre-Trade Checks

### Order Validation

```yaml
safety:
  pre_trade_checks:
    enabled: true

    checks:
      # Price reasonableness
      - name: "Price Check"
        condition: abs(order_price - last_price) / last_price < 0.05
        message: "Order price >5% from last price"

      # Size limits
      - name: "Size Check"
        condition: order_value < max_order_value
        message: "Order exceeds maximum size"

      # No penny stocks
      - name: "Price Floor"
        condition: price > 1.0
        message: "Stock price below $1"

      # Liquidity check
      - name: "Liquidity"
        condition: order_size < avg_daily_volume * 0.01
        message: "Order exceeds 1% of ADV"
```

### What Gets Checked

| Check | Purpose | Example |
|-------|---------|---------|
| Price | No fat fingers | Order within 5% of market |
| Size | Limit impact | <1% of ADV |
| Exposure | Risk limit | Position <3% |
| Liquidity | Executability | Stock has volume |
| Restriction | Compliance | Not on restricted list |

## Position Limits

### Static Limits

```yaml
risk:
  limits:
    # Per-position
    max_position: 0.03          # 3% max per stock
    max_position_value: 50000   # $50K max

    # Sector
    max_sector: 0.20            # 20% per sector

    # Total
    max_positions: 200          # Max 200 positions
```

### Dynamic Limits

```yaml
risk:
  dynamic_limits:
    # Scale down after losses
    - condition: mtd_return < -0.05
      action:
        max_position: 0.02      # Reduce to 2%

    # Scale down in high vol
    - condition: vix > 30
      action:
        gross_exposure: 1.5     # Reduce exposure
```

## Loss Limits

### Daily Loss Limit

```yaml
safety:
  loss_limits:
    daily:
      warning: 0.02    # 2% - send alert
      hard: 0.03       # 3% - halt trading
```

### Drawdown Limits

```yaml
safety:
  loss_limits:
    drawdown:
      warning: 0.10    # 10% - send alert
      hard: 0.15       # 15% - halt trading
```

### Actions on Breach

```yaml
safety:
  loss_limits:
    daily:
      hard: 0.03
      action:
        - halt_new_trades
        - flatten_positions     # Optional
        - notify_admin
```

## Circuit Breakers

### Automatic Halts

```yaml
safety:
  circuit_breakers:
    # Daily loss
    - name: "Daily Loss"
      condition: daily_pnl < -0.03
      action: halt_trading
      reset: next_trading_day

    # Position divergence
    - name: "Position Divergence"
      condition: max_position_divergence > 0.10
      action: halt_trading
      reset: manual

    # System error
    - name: "Error Count"
      condition: error_count_1h > 10
      action: halt_trading
      reset: manual
```

### Market-Wide Halts

Respect exchange circuit breakers:

```yaml
safety:
  market_circuit_breakers:
    enabled: true
    # Pause when market halts
    resume_delay: 300  # Wait 5 min after resume
```

## Kill Switch

### Emergency Stop

```bash
# Immediate halt - no new trades
sigc kill

# Flatten all positions
sigc kill --flatten
```

### Remote Kill Switch

```yaml
safety:
  kill_switch:
    enabled: true

    # Multiple triggers
    triggers:
      - type: slack_command
        command: "/kill trading"

      - type: api
        endpoint: "/api/kill"
        auth: api_key

      - type: watchdog
        timeout: 300  # No heartbeat for 5 min
```

## Data Validation

### Input Validation

```yaml
data:
  validation:
    # Freshness
    max_age: 2h
    stale_action: alert

    # Completeness
    min_assets: 100
    missing_action: warn

    # Sanity
    price_change_limit: 0.50  # 50% daily change suspicious
    price_check_action: exclude
```

### Anomaly Detection

```yaml
data:
  anomaly_detection:
    enabled: true

    checks:
      # Unusual price moves
      - type: price_change
        threshold: 3  # Standard deviations
        action: flag

      # Volume spikes
      - type: volume_spike
        threshold: 10x
        action: flag

      # Missing data
      - type: gaps
        max_gap: 5  # Trading days
        action: alert
```

## Execution Safety

### Order Types

```yaml
execution:
  order_settings:
    # Use limit orders
    default_type: limit
    limit_offset_bps: 5

    # Timeout for unfilled orders
    timeout_seconds: 30
    timeout_action: cancel

    # Max retries
    max_retries: 3
```

### Slippage Protection

```yaml
execution:
  slippage_protection:
    max_slippage_bps: 20
    action_on_breach: cancel
```

### Trade Reconciliation

```yaml
execution:
  reconciliation:
    frequency: hourly
    tolerance: 0.01  # 1% position difference

    on_mismatch:
      - alert
      - log_discrepancy
      - auto_correct  # Optional
```

## Monitoring & Alerting

### Health Checks

```yaml
monitoring:
  health_checks:
    - name: "Daemon Running"
      check: process_alive
      interval: 60s

    - name: "Broker Connected"
      check: broker_ping
      interval: 30s

    - name: "Data Fresh"
      check: data_age < 2h
      interval: 300s

    - name: "Positions Match"
      check: position_reconciliation
      interval: 3600s
```

### Alert Escalation

```yaml
alerting:
  escalation:
    warning:
      - channel: slack
        target: "#trading-alerts"

    critical:
      - channel: slack
        target: "#trading-alerts"
      - channel: pagerduty
        target: "trading-oncall"
      - channel: sms
        target: "+1234567890"
```

## Logging & Audit

### What to Log

```yaml
logging:
  events:
    # All trades
    - type: trade
      level: info
      retain: 7y

    # Signals
    - type: signal_computation
      level: debug
      retain: 30d

    # Errors
    - type: error
      level: error
      retain: 7y

    # Configuration changes
    - type: config_change
      level: info
      retain: 7y
```

### Audit Trail

```yaml
audit:
  enabled: true

  capture:
    - all_orders
    - position_changes
    - config_changes
    - user_actions

  storage:
    type: immutable
    retention: 7y
```

## Disaster Recovery

### Backup Procedures

```yaml
backup:
  # State backup
  state:
    frequency: daily
    destination: s3://backups/state/
    retain: 90d

  # Configuration
  config:
    frequency: on_change
    destination: git
    retain: forever

  # Logs
  logs:
    frequency: daily
    destination: s3://backups/logs/
    retain: 7y
```

### Recovery Procedures

1. **System Failure**
   ```bash
   # Restart daemon
   sigc daemon restart

   # Verify positions
   sigc reconcile

   # Resume trading
   sigc daemon resume
   ```

2. **Data Corruption**
   ```bash
   # Restore from backup
   sigc restore --state latest

   # Verify state
   sigc status --verify
   ```

3. **Broker Issues**
   ```bash
   # Switch to backup broker
   sigc broker switch --to backup
   ```

## Complete Safety Configuration

```yaml
# config/safety.yaml

safety:
  # Pre-trade
  pre_trade_checks:
    enabled: true
    checks:
      - price_reasonableness
      - size_limit
      - liquidity
      - restricted_list

  # Limits
  limits:
    max_position: 0.03
    max_sector: 0.20
    gross_exposure: 2.0
    net_exposure: [-0.1, 0.1]

  # Loss limits
  loss_limits:
    daily:
      warning: 0.02
      hard: 0.03
    drawdown:
      warning: 0.10
      hard: 0.15

  # Circuit breakers
  circuit_breakers:
    - name: daily_loss
      trigger: daily_pnl < -0.03
      action: halt

    - name: position_divergence
      trigger: max_divergence > 0.10
      action: halt

  # Kill switch
  kill_switch:
    enabled: true
    methods: [api, slack, watchdog]

# Monitoring
monitoring:
  health_checks:
    interval: 60s
    checks: [daemon, broker, data, positions]

  prometheus:
    enabled: true
    port: 9090

# Alerting
alerting:
  channels:
    - type: slack
      webhook: ${SLACK_WEBHOOK}
    - type: pagerduty
      key: ${PAGERDUTY_KEY}

  rules:
    - condition: daily_loss > 0.02
      severity: warning
    - condition: daily_loss > 0.03
      severity: critical
    - condition: system_error
      severity: critical

# Logging
logging:
  level: info
  output: /var/log/sigc/
  retention: 7y

# Audit
audit:
  enabled: true
  events: [trades, positions, config]
  storage: immutable
```

## Best Practices Checklist

### Before Going Live

- [ ] All safety systems tested
- [ ] Kill switch verified working
- [ ] Alert escalation tested
- [ ] Backup/recovery tested
- [ ] Documentation complete

### Ongoing

- [ ] Daily system health review
- [ ] Weekly safety drills
- [ ] Monthly backup verification
- [ ] Quarterly disaster recovery test

## Summary

You've completed the Quantitative Trading Guide. You now know how to:

1. **Understand** quantitative trading fundamentals
2. **Build** trading signals from data
3. **Backtest** strategies properly
4. **Manage** portfolio risk
5. **Deploy** to production
6. **Monitor** live trading
7. **Keep systems safe**

Good luck on your trading journey.

## Additional Resources

- [Tutorials](../tutorials/index.md) - Hands-on guides
- [Strategy Library](../strategies/index.md) - Example strategies
- [API Reference](../api/index.md) - Programmatic access
- [Production Guide](../production/index.md) - Detailed production docs
