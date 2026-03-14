# Audit Logging

Comprehensive audit trail for compliance and debugging.

## Overview

Audit logs capture:

- All trading decisions
- Order submissions and fills
- Position changes
- Configuration changes
- User actions

## Configuration

```yaml
audit:
  enabled: true
  file: /var/log/sigc/audit.log
  format: json

  # What to log
  events:
    - signals
    - orders
    - fills
    - positions
    - config_changes
    - user_actions

  # Retention
  retention:
    days: 365
    compress: true
    archive_to: s3://bucket/audit/
```

## Log Format

### Standard Fields

Every audit event includes:

```json
{
  "timestamp": "2024-01-15T14:30:00.123456Z",
  "event_type": "order_submitted",
  "event_id": "evt_abc123",
  "strategy": "momentum",
  "session_id": "sess_xyz789",
  "environment": "production",
  "version": "0.10.0"
}
```

### Signal Events

```json
{
  "event_type": "signal_computed",
  "timestamp": "2024-01-15T09:00:00.123Z",
  "strategy": "momentum",
  "computation_id": "comp_abc123",
  "duration_ms": 2340,
  "input": {
    "data_timestamp": "2024-01-15T08:45:00Z",
    "symbols_count": 500
  },
  "output": {
    "positions_long": 42,
    "positions_short": 38,
    "gross_exposure": 1.85,
    "net_exposure": 0.05,
    "turnover_pct": 0.18
  },
  "weights_hash": "sha256:abc123..."
}
```

### Order Events

```json
{
  "event_type": "order_submitted",
  "timestamp": "2024-01-15T09:30:05.123Z",
  "order_id": "ord_123",
  "client_order_id": "sigc_mom_20240115_001",
  "strategy": "momentum",
  "ticker": "AAPL",
  "side": "buy",
  "quantity": 100,
  "order_type": "limit",
  "limit_price": 185.50,
  "time_in_force": "day",
  "reason": "rebalance",
  "pre_trade_checks": {
    "position_limit": "pass",
    "liquidity": "pass",
    "circuit_breaker": "pass"
  }
}
```

### Fill Events

```json
{
  "event_type": "order_filled",
  "timestamp": "2024-01-15T09:30:08.456Z",
  "order_id": "ord_123",
  "fill_id": "fill_456",
  "ticker": "AAPL",
  "side": "buy",
  "quantity": 100,
  "fill_price": 185.48,
  "commission": 0.50,
  "slippage_bps": -1.08,
  "execution_venue": "NASDAQ"
}
```

### Position Events

```json
{
  "event_type": "position_changed",
  "timestamp": "2024-01-15T09:30:08.456Z",
  "ticker": "AAPL",
  "previous": {
    "shares": 500,
    "market_value": 92740.00,
    "weight_pct": 4.2
  },
  "current": {
    "shares": 600,
    "market_value": 111288.00,
    "weight_pct": 5.0
  },
  "change": {
    "shares": 100,
    "reason": "rebalance",
    "order_id": "ord_123"
  }
}
```

### Configuration Events

```json
{
  "event_type": "config_changed",
  "timestamp": "2024-01-15T10:00:00.000Z",
  "user": "admin",
  "ip_address": "10.0.0.50",
  "changes": [
    {
      "path": "safety.max_position",
      "old_value": 0.05,
      "new_value": 0.03,
      "reason": "Reduce risk limits"
    }
  ]
}
```

## Event Types

| Event Type | Description |
|------------|-------------|
| `signal_computed` | Signal computation completed |
| `order_submitted` | Order sent to broker |
| `order_filled` | Order fully filled |
| `order_partial_fill` | Order partially filled |
| `order_cancelled` | Order cancelled |
| `order_rejected` | Order rejected by broker |
| `position_changed` | Position updated |
| `circuit_breaker_triggered` | Safety limit hit |
| `config_changed` | Configuration modified |
| `user_login` | User authenticated |
| `user_action` | Manual user action |

## Querying Logs

### CLI Query

```bash
# Recent orders
sigc audit query --type order_submitted --last 24h

# Failed orders
sigc audit query --type order_rejected --last 7d

# Specific strategy
sigc audit query --strategy momentum --last 24h

# Specific ticker
sigc audit query --ticker AAPL --last 7d

# Export to JSON
sigc audit query --last 30d --output audit_export.json
```

### Filter Syntax

```bash
# Complex filters
sigc audit query \
  --type order_filled \
  --strategy momentum \
  --after "2024-01-01" \
  --before "2024-01-31" \
  --filter "slippage_bps > 5" \
  --format json
```

## Log Rotation

### Configuration

```yaml
audit:
  rotation:
    max_size_mb: 100
    max_files: 365
    compress: true
    compress_after_days: 7
```

### Manual Rotation

```bash
sigc audit rotate
```

## Archiving

### S3 Archive

```yaml
audit:
  archive:
    enabled: true
    destination: s3://bucket/audit/
    schedule: "0 2 * * *"  # Daily at 2 AM
    retention_days: 30     # Keep 30 days locally
```

### Archive Format

```
s3://bucket/audit/
├── 2024/
│   ├── 01/
│   │   ├── 2024-01-01.log.gz
│   │   ├── 2024-01-02.log.gz
│   │   └── ...
│   └── 02/
└── ...
```

## Compliance

### SOC 2 Requirements

```yaml
audit:
  compliance:
    soc2: true
    events:
      - all_orders
      - all_fills
      - config_changes
      - user_actions
      - access_logs
    tamper_evident: true
    retention_days: 365
```

### Immutable Logs

```yaml
audit:
  immutable:
    enabled: true
    hash_chain: true    # Merkle tree for tamper detection
    signature: true     # Sign log entries
    key_file: /etc/sigc/audit_key.pem
```

### Verification

```bash
sigc audit verify --file /var/log/sigc/audit.log

# Output:
# Verification: PASSED
# Entries: 15,234
# First entry: 2024-01-01T00:00:00Z
# Last entry: 2024-01-15T14:30:00Z
# Hash chain: valid
# Signatures: valid
```

## Integration

### Splunk

```yaml
audit:
  sinks:
    - type: splunk
      url: https://splunk.example.com:8088
      token: ${SPLUNK_TOKEN}
      index: sigc_audit
```

### Elasticsearch

```yaml
audit:
  sinks:
    - type: elasticsearch
      url: https://elasticsearch:9200
      index: sigc-audit-%Y.%m.%d
      username: ${ES_USER}
      password: ${ES_PASSWORD}
```

### AWS CloudWatch

```yaml
audit:
  sinks:
    - type: cloudwatch
      region: us-east-1
      log_group: /sigc/audit
      log_stream: production
```

## Reports

### Daily Audit Summary

```bash
sigc audit report --type daily --date 2024-01-15
```

```
Daily Audit Report: 2024-01-15
==============================

Signals Computed: 3
  momentum: 1 (success)
  value: 1 (success)
  mean_reversion: 1 (success)

Orders:
  Submitted: 85
  Filled: 83
  Rejected: 1
  Cancelled: 1

Fills:
  Total Value: $1,250,000
  Avg Slippage: 2.3 bps
  Total Commission: $125.50

Position Changes:
  Long increased: 25
  Long decreased: 18
  Short increased: 22
  Short decreased: 16

Configuration Changes: 0
User Actions: 2
  - admin: Acknowledged alert
  - admin: Reset circuit breaker

Anomalies: 0
```

### Compliance Report

```bash
sigc audit report --type compliance --month 2024-01
```

## Best Practices

### 1. Log Everything

```yaml
audit:
  events:
    - all  # Don't miss anything
```

### 2. Use Structured Format

```yaml
audit:
  format: json  # Machine-readable
```

### 3. Archive Offsite

```yaml
audit:
  archive:
    destination: s3://bucket/audit/
```

### 4. Verify Regularly

```bash
# Weekly verification
0 0 * * 0 sigc audit verify --last 7d
```

### 5. Retain Appropriately

Regulatory requirements vary:

| Regulation | Retention |
|------------|-----------|
| SEC Rule 17a-4 | 6 years |
| MiFID II | 5 years |
| FINRA | 6 years |

## Next Steps

- [Monitoring](monitoring.md) - Operational metrics
- [Safety Systems](safety-systems.md) - Pre-trade checks
- [Configuration](configuration.md) - Full config reference
