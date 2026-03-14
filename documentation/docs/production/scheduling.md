# Scheduling

Automate signal generation and trading with cron-based scheduling.

## Overview

sigc scheduling:

- Cron expressions for flexibility
- Trading calendar awareness
- Market hours handling
- Timezone support

## Basic Scheduling

### Configuration

```yaml
schedule:
  timezone: America/New_York

  jobs:
    - name: morning_rebalance
      cron: "0 9 * * 1-5"          # 9 AM weekdays
      strategy: momentum_strategy
```

### Cron Syntax

```
┌─────────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌─────────── day of month (1 - 31)
│ │ │ ┌───────── month (1 - 12)
│ │ │ │ ┌─────── day of week (0 - 6) (Sunday = 0)
│ │ │ │ │
│ │ │ │ │
* * * * *
```

### Common Schedules

| Schedule | Cron | Description |
|----------|------|-------------|
| Daily 9 AM | `0 9 * * *` | Every day at 9 AM |
| Weekdays 9 AM | `0 9 * * 1-5` | Mon-Fri at 9 AM |
| Hourly | `0 * * * *` | Every hour |
| Every 15 min | `*/15 * * * *` | Every 15 minutes |
| Monday 9 AM | `0 9 * * 1` | Weekly on Monday |
| First of month | `0 9 1 * *` | Monthly on 1st |

## Trading Calendar

### NYSE Calendar

```yaml
schedule:
  calendar: nyse
  skip_holidays: true
  skip_early_close: false

  jobs:
    - name: rebalance
      cron: "0 9 * * 1-5"
      strategy: momentum
      # Automatically skips NYSE holidays
```

### Custom Calendar

```yaml
schedule:
  calendar: custom
  holidays_file: /etc/sigc/holidays.yaml

  # Or inline
  holidays:
    - "2024-01-01"  # New Year's Day
    - "2024-01-15"  # MLK Day
    - "2024-02-19"  # Presidents' Day
    # ...
```

### Market Hours

```yaml
schedule:
  market_hours:
    open: "09:30"
    close: "16:00"
    timezone: America/New_York

  jobs:
    - name: intraday_signal
      cron: "*/30 9-15 * * 1-5"  # Every 30 min during market
      condition: market_open
```

## Job Types

### Signal Computation

```yaml
jobs:
  - name: compute_signals
    type: compute
    cron: "0 8 * * 1-5"
    strategy: momentum_strategy
```

### Execute Trades

```yaml
jobs:
  - name: execute_trades
    type: execute
    cron: "30 9 * * 1-5"
    strategy: momentum_strategy
```

### Compute + Execute

```yaml
jobs:
  - name: full_rebalance
    type: rebalance
    cron: "0 9 * * 1-5"
    strategy: momentum_strategy
    # Computes signals, then executes trades
```

### Data Refresh

```yaml
jobs:
  - name: refresh_data
    type: data_refresh
    cron: "0 6 * * 1-5"
    source: prices
```

### Reconciliation

```yaml
jobs:
  - name: position_check
    type: reconcile
    cron: "0 16 * * 1-5"
```

### Custom Script

```yaml
jobs:
  - name: custom_job
    type: script
    cron: "0 17 * * 1-5"
    script: /opt/sigc/scripts/eod_report.sh
```

## Job Conditions

### Market Open

```yaml
jobs:
  - name: intraday_signal
    cron: "*/30 * * * 1-5"
    condition: market_open
```

### Data Fresh

```yaml
jobs:
  - name: compute
    cron: "0 9 * * 1-5"
    condition: data_fresh
    data_max_age_minutes: 30
```

### Previous Job Success

```yaml
jobs:
  - name: compute
    cron: "0 9 * * 1-5"

  - name: execute
    cron: "30 9 * * 1-5"
    depends_on: compute
```

### Custom Condition

```yaml
jobs:
  - name: low_vol_trade
    cron: "0 10 * * 1-5"
    condition:
      type: metric
      expr: "vix < 20"
```

## Execution Windows

### Time Window

```yaml
jobs:
  - name: rebalance
    cron: "0 9 * * 1-5"
    execution:
      window_minutes: 30
      algorithm: twap
```

### Retry Policy

```yaml
jobs:
  - name: rebalance
    cron: "0 9 * * 1-5"
    retry:
      max_attempts: 3
      delay_minutes: 5
      backoff: exponential
```

## Schedule Management

### View Schedule

```bash
sigc schedule list
```

```
Scheduled Jobs:
NAME               | CRON           | NEXT RUN            | STATUS
-------------------+----------------+---------------------+--------
morning_rebalance  | 0 9 * * 1-5   | 2024-01-16 09:00:00| enabled
eod_reconcile      | 0 16 * * 1-5  | 2024-01-15 16:00:00| enabled
data_refresh       | 0 6 * * 1-5   | 2024-01-16 06:00:00| enabled
```

### Enable/Disable Jobs

```bash
# Disable a job
sigc schedule disable morning_rebalance

# Enable a job
sigc schedule enable morning_rebalance

# Disable all
sigc schedule pause

# Enable all
sigc schedule resume
```

### Run Immediately

```bash
# Run job now (outside schedule)
sigc schedule run morning_rebalance

# Dry run
sigc schedule run morning_rebalance --dry-run
```

### View History

```bash
sigc schedule history morning_rebalance
```

```
Job History: morning_rebalance

TIMESTAMP            | STATUS  | DURATION | DETAILS
---------------------+---------+----------+---------
2024-01-15 09:00:00 | success | 2.34s    | 80 positions
2024-01-12 09:00:00 | success | 2.51s    | 82 positions
2024-01-11 09:00:00 | success | 2.28s    | 79 positions
2024-01-10 09:00:00 | failed  | 0.85s    | Data error
2024-01-09 09:00:00 | success | 2.45s    | 81 positions
```

## Multiple Strategies

### Separate Schedules

```yaml
schedule:
  jobs:
    - name: momentum_rebal
      cron: "0 9 * * 1-5"
      strategy: momentum

    - name: value_rebal
      cron: "0 9 1 * *"        # Monthly
      strategy: value

    - name: intraday_signal
      cron: "*/30 9-15 * * 1-5"
      strategy: mean_reversion
```

### Coordinated Execution

```yaml
schedule:
  jobs:
    # Compute all signals first
    - name: compute_all
      cron: "0 9 * * 1-5"
      type: compute
      strategies: [momentum, value, mean_reversion]

    # Then execute in order
    - name: execute_momentum
      cron: "15 9 * * 1-5"
      type: execute
      strategy: momentum
      depends_on: compute_all

    - name: execute_value
      cron: "20 9 * * 1-5"
      type: execute
      strategy: value
      depends_on: compute_all
```

## Notifications

### Job Notifications

```yaml
schedule:
  notifications:
    on_start: false
    on_success: true
    on_failure: true
    channels: [slack]

  jobs:
    - name: rebalance
      cron: "0 9 * * 1-5"
      notifications:
        on_success: slack
        on_failure: [slack, pagerduty]
```

## Monitoring

### Schedule Metrics

```prometheus
sigc_schedule_runs_total{job="morning_rebalance",status="success"} 152
sigc_schedule_runs_total{job="morning_rebalance",status="failed"} 3
sigc_schedule_last_run_timestamp{job="morning_rebalance"} 1705320000
sigc_schedule_next_run_timestamp{job="morning_rebalance"} 1705406400
sigc_schedule_duration_seconds{job="morning_rebalance"} 2.34
```

## Best Practices

### 1. Schedule Before Market Open

```yaml
# Good: Compute before market opens
- name: compute
  cron: "0 8 * * 1-5"     # 8 AM

# Execute at open
- name: execute
  cron: "30 9 * * 1-5"    # 9:30 AM
  depends_on: compute
```

### 2. Use Trading Calendar

```yaml
schedule:
  calendar: nyse
  skip_holidays: true     # Don't run on holidays
```

### 3. Add Retry Logic

```yaml
retry:
  max_attempts: 3
  delay_minutes: 5
```

### 4. Monitor Job Health

Set up alerts for failed jobs:

```yaml
alerting:
  rules:
    - name: schedule_failure
      condition: "job_status == 'failed'"
      severity: high
```

### 5. Test Schedule Changes

```bash
# Preview next N runs
sigc schedule preview --runs 10
```

## Next Steps

- [Daemon Mode](daemon-mode.md) - Running sigc as service
- [Monitoring](monitoring.md) - Track schedule execution
- [Alerting](alerting.md) - Notifications on failure
