# Alerting

Configure notifications for trading events, errors, and anomalies.

## Overview

sigc supports multiple alerting channels:

- **Slack**: Team notifications
- **PagerDuty**: On-call escalation
- **Email**: Backup notifications
- **Webhooks**: Custom integrations

## Slack Integration

### Setup

1. Create Slack App at https://api.slack.com/apps
2. Add Incoming Webhook
3. Configure in sigc:

```yaml
alerting:
  slack:
    enabled: true
    webhook: ${SLACK_WEBHOOK}
    channel: "#trading-alerts"
```

### Alert Levels

```yaml
alerting:
  slack:
    channels:
      info: "#trading-info"
      warning: "#trading-alerts"
      critical: "#trading-critical"
    mention:
      warning: "@here"
      critical: "@channel"
```

### Example Alert

```
🚨 sigc Alert: Daily Loss Limit

Severity: HIGH
Strategy: momentum_strategy
Time: 2024-01-15 14:32:00 EST

Daily P&L: -2.1%
Limit: 2.0%

Action: Trading halted
Status: Circuit breaker triggered

@channel Please investigate immediately.
```

## PagerDuty Integration

### Setup

```yaml
alerting:
  pagerduty:
    enabled: true
    api_key: ${PAGERDUTY_API_KEY}
    service_id: ${PAGERDUTY_SERVICE_ID}
```

### Escalation Policies

```yaml
alerting:
  pagerduty:
    escalation:
      - level: 1
        delay_minutes: 0
        target: on_call_engineer

      - level: 2
        delay_minutes: 15
        target: trading_lead

      - level: 3
        delay_minutes: 30
        target: engineering_manager
```

### Severity Mapping

```yaml
alerting:
  pagerduty:
    severity_map:
      info: info
      warning: warning
      high: error
      critical: critical
```

## Email Alerts

### Setup

```yaml
alerting:
  email:
    enabled: true
    smtp:
      host: smtp.gmail.com
      port: 587
      username: ${SMTP_USERNAME}
      password: ${SMTP_PASSWORD}
      tls: true
    from: alerts@example.com
    to:
      - trading-team@example.com
      - oncall@example.com
```

### Email Template

```yaml
alerting:
  email:
    subject_template: "[sigc ${severity}] ${alert_name}"
    body_template: |
      Alert: ${alert_name}
      Severity: ${severity}
      Time: ${timestamp}

      Details:
      ${message}

      Strategy: ${strategy}
      Current Value: ${value}
      Threshold: ${threshold}

      ---
      This is an automated alert from sigc.
```

## Webhook Alerts

### Custom Webhook

```yaml
alerting:
  webhooks:
    - name: custom_endpoint
      url: https://api.example.com/alerts
      method: POST
      headers:
        Authorization: "Bearer ${WEBHOOK_TOKEN}"
        Content-Type: application/json
      body_template: |
        {
          "alert": "${alert_name}",
          "severity": "${severity}",
          "message": "${message}",
          "timestamp": "${timestamp}"
        }
```

### Microsoft Teams

```yaml
alerting:
  teams:
    enabled: true
    webhook: ${TEAMS_WEBHOOK}
```

## Alert Rules

### Define Alert Conditions

```yaml
alerting:
  rules:
    # Performance alerts
    - name: daily_loss
      condition: "daily_pnl_pct < -0.02"
      severity: critical
      channels: [slack, pagerduty]
      message: "Daily loss limit exceeded: ${daily_pnl_pct}%"

    - name: large_drawdown
      condition: "drawdown > 0.10"
      severity: high
      channels: [slack, pagerduty]
      message: "Drawdown at ${drawdown}%"

    # Data alerts
    - name: data_stale
      condition: "data_age_hours > 2"
      severity: high
      channels: [slack, email]
      message: "Data is ${data_age_hours} hours old"

    - name: data_missing
      condition: "missing_symbols > 0"
      severity: warning
      channels: [slack]
      message: "Missing ${missing_symbols} symbols"

    # System alerts
    - name: high_memory
      condition: "memory_pct > 90"
      severity: warning
      channels: [slack]
      message: "Memory usage at ${memory_pct}%"

    - name: connection_lost
      condition: "broker_connected == false"
      severity: critical
      channels: [slack, pagerduty]
      message: "Lost connection to broker"

    # Trading alerts
    - name: large_position
      condition: "max_position_pct > 0.04"
      severity: warning
      channels: [slack]
      message: "Position in ${max_position_ticker} at ${max_position_pct}%"

    - name: high_turnover
      condition: "daily_turnover > 0.50"
      severity: warning
      channels: [slack]
      message: "Daily turnover at ${daily_turnover}%"
```

### Complex Conditions

```yaml
alerting:
  rules:
    - name: sustained_loss
      condition: |
        daily_pnl_pct < -0.01 AND
        weekly_pnl_pct < -0.03
      severity: high
      channels: [slack, pagerduty]

    - name: correlation_breakdown
      condition: |
        abs(rolling_corr_benchmark - historical_corr) > 0.3
      severity: warning
      channels: [slack]
```

## Alert Throttling

Prevent alert fatigue:

```yaml
alerting:
  throttling:
    # Don't repeat same alert within window
    dedupe_window_minutes: 30

    # Max alerts per hour
    max_alerts_per_hour: 10

    # Aggregate similar alerts
    aggregate:
      enabled: true
      window_minutes: 5
```

## Alert Status

### Check Alert Status

```bash
sigc alerts status
```

```
Active Alerts:
  [HIGH] data_stale - Data is 3 hours old (triggered 2h ago)

Recent Alerts (24h):
  [RESOLVED] daily_loss - Resolved after 45 minutes
  [WARNING] high_turnover - Turnover at 52% (auto-resolved)

Alert Statistics:
  Total (24h): 5
  Critical: 0
  High: 1
  Warning: 4
```

### Acknowledge Alert

```bash
sigc alerts ack --id alert_123
```

### Resolve Alert

```bash
sigc alerts resolve --id alert_123 --message "Investigated, false positive"
```

## Testing Alerts

### Test All Channels

```bash
sigc alerts test
```

### Test Specific Channel

```bash
sigc alerts test --channel slack
sigc alerts test --channel pagerduty
sigc alerts test --channel email
```

### Simulate Alert

```bash
sigc alerts simulate --rule daily_loss --value -0.025
```

## Daily Reports

### Enable Daily Summary

```yaml
alerting:
  reports:
    daily_summary:
      enabled: true
      time: "17:00"
      timezone: America/New_York
      channels: [email, slack]
```

### Daily Summary Content

```
📊 sigc Daily Summary - 2024-01-15

Performance:
  Daily P&L: +1.2%
  MTD: +3.5%
  YTD: +8.2%

Positions:
  Long: 42 positions
  Short: 38 positions
  Gross: 185%
  Net: +5%

Trading Activity:
  Orders: 85
  Fills: 83 (97.6%)
  Turnover: 18%

Alerts Today:
  Critical: 0
  High: 0
  Warning: 2

System Health: All systems operational
```

## Best Practices

### 1. Start with Fewer Alerts

```yaml
alerting:
  rules:
    # Start with critical alerts only
    - name: daily_loss_critical
      condition: "daily_pnl_pct < -0.03"
      severity: critical
```

Add more alerts as you understand baseline.

### 2. Use Severity Appropriately

| Severity | When to Use |
|----------|-------------|
| `info` | FYI, no action needed |
| `warning` | Worth watching, may need action |
| `high` | Needs attention soon |
| `critical` | Immediate action required |

### 3. Include Context

```yaml
message: |
  Daily loss exceeded: ${daily_pnl_pct}%
  Largest loss: ${worst_position_ticker} (${worst_position_pnl}%)
  Time in drawdown: ${drawdown_duration}
  Recommended: Review position sizing
```

### 4. Test Regularly

```bash
# Weekly alert testing
0 9 * * 1 sigc alerts test
```

### 5. Review and Tune

Track alert frequency and adjust thresholds:

```bash
sigc alerts stats --period 30d
```

## Next Steps

- [Monitoring](monitoring.md) - Prometheus metrics
- [Safety Systems](safety-systems.md) - Circuit breakers
- [Configuration](configuration.md) - Full config reference
