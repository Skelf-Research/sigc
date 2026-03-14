# Safety Systems

Pre-trade checks and circuit breakers to protect your portfolio.

## Overview

Safety systems prevent:

- Excessive losses
- Position concentration
- Runaway algorithms
- Data errors causing bad trades

## Pre-Trade Checks

Every order is validated before submission:

```yaml
safety:
  pre_trade_checks:
    - position_limit
    - sector_limit
    - loss_limit
    - liquidity_check
    - price_sanity
```

### Position Limits

```yaml
safety:
  max_position: 0.05        # No position > 5%
  min_position: 0.001       # No position < 0.1%
  max_sector: 0.25          # No sector > 25%
```

If an order would violate limits, it's rejected:

```
[REJECTED] Order would exceed position limit
  Ticker: AAPL
  Current Position: 4.2%
  Order Would Create: 6.1%
  Limit: 5.0%
```

### Loss Limits

```yaml
safety:
  max_daily_loss: 0.02      # 2% daily loss limit
  max_weekly_loss: 0.05     # 5% weekly loss limit
  max_monthly_loss: 0.10    # 10% monthly loss limit
  max_drawdown: 0.15        # 15% max drawdown
```

### Liquidity Checks

```yaml
safety:
  liquidity:
    min_adv: 1000000        # Min $1M average daily volume
    max_participation: 0.10  # Max 10% of daily volume
    max_order_pct_volume: 0.01  # Max 1% of daily volume per order
```

### Price Sanity

```yaml
safety:
  price_checks:
    max_spread_bps: 50      # Reject if spread > 50 bps
    max_price_move: 0.10    # Reject if price moved > 10% from reference
    stale_quote_seconds: 60 # Reject if quote older than 60s
```

## Circuit Breakers

Automatic trading halts when thresholds are breached.

### Configuration

```yaml
safety:
  circuit_breakers:
    enabled: true

    triggers:
      - name: daily_loss
        type: loss
        threshold: 0.02     # 2% daily loss
        window: 1d
        action: halt_trading

      - name: rapid_loss
        type: loss
        threshold: 0.01     # 1% loss
        window: 1h          # In 1 hour
        action: halt_trading

      - name: drawdown
        type: drawdown
        threshold: 0.10     # 10% drawdown
        action: halt_trading

      - name: consecutive_losses
        type: streak
        losses: 5           # 5 consecutive losing trades
        action: pause_30min

      - name: volatility_spike
        type: volatility
        threshold: 3.0      # 3x normal volatility
        action: reduce_exposure

    # Recovery
    recovery:
      auto_resume: false    # Require manual intervention
      cooldown_minutes: 30
```

### Circuit Breaker Actions

| Action | Description |
|--------|-------------|
| `halt_trading` | Stop all trading immediately |
| `pause_30min` | Pause for 30 minutes |
| `reduce_exposure` | Reduce positions by 50% |
| `flatten` | Close all positions |
| `alert_only` | Send alert, continue trading |

### Manual Override

```bash
# Check circuit breaker status
sigc safety status

# Reset circuit breaker (after investigation)
sigc safety reset --breaker daily_loss

# Force halt
sigc safety halt

# Resume trading
sigc safety resume
```

## Order Validation

### Size Validation

```yaml
safety:
  orders:
    max_order_value: 100000     # Max $100K per order
    max_order_pct: 0.03         # Max 3% of portfolio
    min_order_value: 100        # Min $100 per order
```

### Price Validation

```yaml
safety:
  orders:
    max_slippage_bps: 50        # Reject if execution > 50 bps from limit
    require_limit_price: true    # No market orders
    limit_price_offset_max: 0.01 # Limit within 1% of market
```

### Duplicate Detection

```yaml
safety:
  orders:
    duplicate_window_seconds: 5  # Block duplicate orders within 5s
    max_orders_per_minute: 20    # Rate limit
```

## Data Validation

### Freshness Checks

```yaml
safety:
  data:
    max_stale_hours: 2          # Data must be < 2 hours old
    require_complete: true       # All expected symbols present
    min_data_points: 252         # At least 1 year of history
```

### Anomaly Detection

```yaml
safety:
  data:
    anomaly_detection:
      enabled: true
      max_return_1d: 0.50       # Flag returns > 50%
      max_volume_ratio: 10      # Flag volume > 10x average
      check_gaps: true          # Flag missing data
```

### Action on Data Issues

```yaml
safety:
  on_data_error: use_cache      # use_cache | hold | halt
  on_stale_data: alert          # alert | use_cache | halt
  on_missing_symbol: exclude    # exclude | halt
```

## Signal Validation

### Sanity Checks

```yaml
safety:
  signals:
    max_turnover: 1.0           # Reject if turnover > 100%
    max_position_change: 0.10   # No single position change > 10%
    require_neutral: true       # Must be dollar neutral
```

### Anomaly Detection

```yaml
safety:
  signals:
    anomaly_detection:
      enabled: true
      historical_comparison: true
      max_deviation_sigma: 3    # Flag if signal > 3σ from history
```

## Kill Switch

Emergency stop for all trading:

### Manual Kill Switch

```bash
sigc kill
```

This immediately:

1. Cancels all open orders
2. Halts new order submission
3. Sends alert to all channels
4. Requires manual restart

### Automatic Kill Switch

```yaml
safety:
  kill_switch:
    # External kill file
    file: /var/run/sigc/kill

    # Network-based kill
    endpoint: https://kill-switch.example.com/sigc
    check_interval_seconds: 10
```

Create kill file to stop:

```bash
touch /var/run/sigc/kill
```

Remove to resume:

```bash
rm /var/run/sigc/kill
sigc safety resume
```

## Position Reconciliation

### Automatic Reconciliation

```yaml
safety:
  reconciliation:
    enabled: true
    frequency: "*/5 * * * *"    # Every 5 minutes
    tolerance_pct: 0.01         # 1% tolerance
    on_mismatch: alert          # alert | halt | sync
```

### Manual Check

```bash
sigc reconcile
```

Output:

```
Position Reconciliation
=======================
Status: MATCH

Internal Positions:
  AAPL: 1000 shares ($185,640)
  MSFT: 500 shares ($187,255)
  ...

Broker Positions:
  AAPL: 1000 shares ($185,640)
  MSFT: 500 shares ($187,255)
  ...

Discrepancies: None
```

## Risk Monitoring

### Real-time Risk

```yaml
safety:
  risk:
    monitor_interval_seconds: 60
    metrics:
      - daily_var_95
      - portfolio_beta
      - sector_exposure
      - concentration
```

### Risk Alerts

```yaml
safety:
  risk:
    alerts:
      - metric: daily_var_95
        threshold: 0.03
        action: alert

      - metric: portfolio_beta
        min: 0.5
        max: 1.5
        action: alert

      - metric: max_sector_exposure
        threshold: 0.30
        action: reduce
```

## Safety Dashboard

```bash
sigc safety dashboard
```

```
Safety Systems Dashboard
========================

Circuit Breakers:
  daily_loss:      OK (current: -0.5%, limit: 2%)
  drawdown:        OK (current: 3.2%, limit: 10%)
  consecutive:     OK (losses: 1, limit: 5)

Position Limits:
  max_position:    OK (largest: 4.2%, limit: 5%)
  max_sector:      OK (largest: 22%, limit: 25%)

Data Status:
  freshness:       OK (age: 15 min)
  completeness:    OK (100% symbols)

Reconciliation:
  status:          MATCH
  last_check:      2 min ago

Open Orders: 3
Pending Risk: 1.2% of NAV
```

## Best Practices

### 1. Start Conservative

```yaml
safety:
  max_position: 0.02        # Very conservative
  max_daily_loss: 0.01      # Tight loss limit
```

Loosen gradually with experience.

### 2. Test Circuit Breakers

```bash
sigc safety test --breaker daily_loss
```

### 3. Require Manual Recovery

```yaml
safety:
  circuit_breakers:
    recovery:
      auto_resume: false    # Always investigate
```

### 4. Multiple Alert Channels

```yaml
alerting:
  channels:
    - slack       # Primary
    - pagerduty   # Escalation
    - email       # Backup
```

### 5. Regular Reconciliation

```yaml
safety:
  reconciliation:
    frequency: "*/5 * * * *"  # Every 5 minutes
```

### 6. Document Override Procedures

Create runbook for:

- When to override safety
- Who can authorize
- Required documentation

## Next Steps

- [Alerting](alerting.md) - Notification configuration
- [Monitoring](monitoring.md) - Metrics and dashboards
- [Configuration](configuration.md) - Full config reference
