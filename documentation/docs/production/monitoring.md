# Monitoring

Prometheus metrics and observability for sigc.

## Overview

sigc exposes comprehensive metrics for:

- Strategy performance
- System health
- Trading activity
- Data pipeline status

## Prometheus Setup

### Enable Metrics

```yaml
monitoring:
  prometheus:
    enabled: true
    port: 9090
    path: /metrics
```

### Scrape Configuration

Add to your Prometheus config:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'sigc'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
```

## Available Metrics

### Performance Metrics

```prometheus
# Portfolio value
sigc_portfolio_value{strategy="momentum"} 1050000

# Daily P&L
sigc_daily_pnl_pct{strategy="momentum"} 0.012

# Drawdown
sigc_drawdown_pct{strategy="momentum"} 0.032

# Sharpe ratio (rolling 30d)
sigc_sharpe_ratio_30d{strategy="momentum"} 1.25

# Returns
sigc_return_total{strategy="momentum"} 0.085
sigc_return_daily{strategy="momentum"} 0.0012
```

### Position Metrics

```prometheus
# Position count
sigc_positions_long_count{strategy="momentum"} 42
sigc_positions_short_count{strategy="momentum"} 38

# Exposure
sigc_exposure_gross_pct{strategy="momentum"} 1.85
sigc_exposure_net_pct{strategy="momentum"} 0.05

# Concentration
sigc_position_max_pct{strategy="momentum",ticker="NVDA"} 0.042
sigc_sector_max_pct{strategy="momentum",sector="Technology"} 0.22
```

### Trading Metrics

```prometheus
# Order counts
sigc_orders_total{strategy="momentum",status="filled"} 1523
sigc_orders_total{strategy="momentum",status="rejected"} 12
sigc_orders_total{strategy="momentum",status="cancelled"} 45

# Turnover
sigc_turnover_daily_pct{strategy="momentum"} 0.18
sigc_turnover_monthly_pct{strategy="momentum"} 0.85

# Transaction costs
sigc_transaction_costs_bps{strategy="momentum"} 8.5
```

### System Metrics

```prometheus
# Daemon status
sigc_daemon_uptime_seconds 302400
sigc_daemon_restarts_total 2

# Memory
sigc_memory_used_bytes 1073741824
sigc_memory_limit_bytes 4294967296

# CPU
sigc_cpu_usage_pct 15.2

# Computation
sigc_computation_duration_seconds{strategy="momentum"} 2.34
sigc_computation_last_success_timestamp{strategy="momentum"} 1705320000
```

### Data Metrics

```prometheus
# Data freshness
sigc_data_age_seconds{source="prices"} 900
sigc_data_rows_total{source="prices"} 2500000

# Cache
sigc_cache_hits_total 15234
sigc_cache_misses_total 523
sigc_cache_size_bytes 2147483648
```

### Alert Metrics

```prometheus
# Alert counts
sigc_alerts_total{severity="critical"} 0
sigc_alerts_total{severity="high"} 2
sigc_alerts_total{severity="warning"} 15

# Circuit breakers
sigc_circuit_breaker_triggered_total{breaker="daily_loss"} 1
sigc_circuit_breaker_status{breaker="daily_loss"} 0  # 0=ok, 1=tripped
```

## Grafana Dashboards

### Import Dashboard

sigc provides pre-built Grafana dashboards:

```bash
# Export dashboard JSON
sigc monitoring export-dashboard > sigc-dashboard.json

# Import to Grafana
curl -X POST \
  -H "Content-Type: application/json" \
  -d @sigc-dashboard.json \
  http://grafana:3000/api/dashboards/db
```

### Dashboard Panels

**Overview Panel:**
- Portfolio value over time
- Daily P&L
- Current drawdown
- Key metrics

**Performance Panel:**
- Cumulative returns
- Rolling Sharpe
- Drawdown chart
- Monthly returns heatmap

**Positions Panel:**
- Long/short counts
- Gross/net exposure
- Sector breakdown
- Top positions

**Trading Panel:**
- Orders by status
- Turnover trend
- Transaction costs
- Fill rates

**System Panel:**
- Memory/CPU usage
- Computation times
- Data freshness
- Error rates

## Health Checks

### Endpoints

```yaml
monitoring:
  health:
    enabled: true
    port: 8080
```

```bash
# Liveness (is process running?)
curl http://localhost:8080/live
# Returns 200 OK or 503

# Readiness (is service ready?)
curl http://localhost:8080/ready
# Returns 200 OK or 503

# Health (detailed status)
curl http://localhost:8080/health
```

### Health Response

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:30:00Z",
  "checks": {
    "data_freshness": {
      "status": "healthy",
      "message": "Data is 15 minutes old",
      "last_check": "2024-01-15T14:29:55Z"
    },
    "broker_connection": {
      "status": "healthy",
      "message": "Connected to Alpaca",
      "latency_ms": 45
    },
    "memory": {
      "status": "healthy",
      "used_pct": 62,
      "limit_mb": 4096
    },
    "disk": {
      "status": "healthy",
      "used_pct": 45,
      "free_gb": 120
    }
  },
  "uptime_seconds": 302400
}
```

### Custom Health Checks

```yaml
monitoring:
  health:
    checks:
      - name: market_hours
        type: time_window
        start: "09:30"
        end: "16:00"
        timezone: America/New_York

      - name: data_complete
        type: custom
        script: /opt/sigc/scripts/check_data.sh
        timeout_seconds: 30
```

## Logging

### Structured Logging

```yaml
monitoring:
  logging:
    format: json
    level: info
    file: /var/log/sigc/sigc.log
```

### Log Format

```json
{
  "timestamp": "2024-01-15T14:30:00.123Z",
  "level": "info",
  "component": "scheduler",
  "strategy": "momentum",
  "message": "Signal computation completed",
  "duration_ms": 2340,
  "positions": 80,
  "turnover_pct": 0.18
}
```

### Log Levels

| Level | Description |
|-------|-------------|
| `debug` | Detailed debugging info |
| `info` | Normal operations |
| `warn` | Warning conditions |
| `error` | Error conditions |

### Log Aggregation

Send logs to external systems:

```yaml
monitoring:
  logging:
    sinks:
      - type: elasticsearch
        url: http://elasticsearch:9200
        index: sigc-logs

      - type: cloudwatch
        region: us-east-1
        log_group: /sigc/production
```

## Alertmanager Integration

### Configure Alertmanager

```yaml
# alertmanager.yml
route:
  group_by: ['alertname', 'strategy']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'default'

  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#trading-alerts'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: ${PAGERDUTY_KEY}
```

### Alert Rules

```yaml
# sigc.rules.yml
groups:
  - name: sigc
    rules:
      - alert: SigcHighDrawdown
        expr: sigc_drawdown_pct > 0.10
        for: 5m
        labels:
          severity: high
        annotations:
          summary: "High drawdown: {{ $value | humanizePercentage }}"

      - alert: SigcDaemonDown
        expr: up{job="sigc"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "sigc daemon is down"

      - alert: SigcDataStale
        expr: sigc_data_age_seconds > 7200
        for: 5m
        labels:
          severity: high
        annotations:
          summary: "Data is {{ $value | humanizeDuration }} old"
```

## Tracing

### OpenTelemetry Integration

```yaml
monitoring:
  tracing:
    enabled: true
    exporter: jaeger
    endpoint: http://jaeger:14268/api/traces
    sample_rate: 0.1
```

### Trace Spans

```
[signal_computation] 2.34s
├─[load_data] 0.85s
│  ├─[fetch_prices] 0.72s
│  └─[fetch_fundamentals] 0.12s
├─[compute_signals] 1.12s
│  ├─[momentum] 0.45s
│  └─[value] 0.67s
└─[generate_weights] 0.37s
```

## Best Practices

### 1. Set Up Dashboards Early

Create dashboards before going live.

### 2. Alert on Symptoms, Not Causes

```yaml
# Good: Alert on outcome
- alert: SigcHighDrawdown
  expr: sigc_drawdown_pct > 0.10

# Less useful: Alert on potential cause
- alert: SigcHighMemory
  expr: sigc_memory_used_pct > 80
```

### 3. Use Recording Rules

Pre-compute expensive queries:

```yaml
groups:
  - name: sigc_recording
    rules:
      - record: sigc:sharpe_ratio_30d
        expr: |
          (avg_over_time(sigc_return_daily[30d]) * 252) /
          (stddev_over_time(sigc_return_daily[30d]) * sqrt(252))
```

### 4. Retain Data Appropriately

```yaml
# prometheus.yml
storage:
  tsdb:
    retention.time: 90d  # Keep 90 days
```

### 5. Test Monitoring

```bash
sigc monitoring test
```

Verify all metrics are being collected.

## Next Steps

- [Alerting](alerting.md) - Notification setup
- [Safety Systems](safety-systems.md) - Circuit breakers
- [Daemon Mode](daemon-mode.md) - Running as service
