# Daemon Mode

Run sigc as a long-running service for production trading.

## Overview

Daemon mode provides:

- Persistent process for continuous operation
- RPC interface for external commands
- Automatic scheduling of signal computation
- Health monitoring and self-healing
- Graceful shutdown handling

## Starting the Daemon

### Basic Start

```bash
sigc daemon
```

### With Configuration

```bash
sigc daemon --config sigc.yaml
```

### Background Process

```bash
sigc daemon --daemonize
```

### With PID File

```bash
sigc daemon --pid-file /var/run/sigc.pid
```

## Configuration

```yaml
# sigc.yaml
daemon:
  # RPC settings
  rpc:
    address: "127.0.0.1:5555"
    auth: token
    token: ${SIGC_RPC_TOKEN}

  # Process settings
  pid_file: /var/run/sigc.pid
  work_dir: /var/lib/sigc

  # Resource limits
  max_memory_mb: 4096
  max_cpu_pct: 80

  # Logging
  log_file: /var/log/sigc/daemon.log
  log_level: info
```

## RPC Commands

The daemon exposes an RPC interface for control:

### Status

```bash
sigc status
```

Output:

```
sigc daemon status
==================
Status: running
PID: 12345
Uptime: 3d 14h 22m
Memory: 1.2 GB / 4 GB
CPU: 15%

Strategies:
  momentum_strategy: active
    Last run: 2024-01-15 09:00:00
    Next run: 2024-01-16 09:00:00
    Status: success

  value_strategy: active
    Last run: 2024-01-15 09:00:00
    Next run: 2024-01-16 09:00:00
    Status: success
```

### Run Strategy

```bash
# Run immediately
sigc run momentum_strategy.sig

# Dry run (no execution)
sigc run momentum_strategy.sig --dry-run

# Override date
sigc run momentum_strategy.sig --date 2024-01-15
```

### Pause/Resume

```bash
# Pause all strategies
sigc pause

# Pause specific strategy
sigc pause momentum_strategy

# Resume
sigc resume
sigc resume momentum_strategy
```

### Reload Configuration

```bash
sigc reload
```

Reloads configuration without restarting daemon.

### Stop

```bash
# Graceful stop
sigc stop

# Force stop
sigc stop --force

# Stop with timeout
sigc stop --timeout 30
```

## Signal Computation Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                    Computation Lifecycle                    │
│                                                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐ │
│  │  Load   │───▶│ Compute │───▶│ Validate│───▶│ Execute │ │
│  │  Data   │    │ Signal  │    │ Weights │    │ Trades  │ │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘ │
│       │              │              │              │       │
│       ▼              ▼              ▼              ▼       │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                    Audit Log                        │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Data Caching

The daemon caches data for efficiency:

```yaml
daemon:
  cache:
    enabled: true
    directory: /var/cache/sigc
    max_size_gb: 10
    ttl_hours: 24
```

### Cache Management

```bash
# View cache status
sigc cache status

# Clear cache
sigc cache clear

# Clear specific data
sigc cache clear --pattern "prices_*"
```

## Health Monitoring

### Health Endpoint

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "healthy",
  "checks": {
    "data_connection": "ok",
    "broker_connection": "ok",
    "memory": "ok",
    "disk": "ok"
  },
  "uptime_seconds": 302400
}
```

### Liveness Probe

```bash
curl http://localhost:8080/live
```

### Readiness Probe

```bash
curl http://localhost:8080/ready
```

## Graceful Shutdown

On SIGTERM or `sigc stop`:

1. Stop accepting new computations
2. Complete in-progress computations
3. Flush pending writes
4. Close connections
5. Exit cleanly

```yaml
daemon:
  shutdown:
    timeout_seconds: 60
    save_state: true
```

## Automatic Recovery

### Crash Recovery

Daemon automatically recovers from crashes:

```yaml
daemon:
  recovery:
    enabled: true
    max_restarts: 3
    restart_delay_seconds: 10
```

### State Persistence

Save state for recovery:

```yaml
daemon:
  state:
    persist: true
    file: /var/lib/sigc/state.json
```

## Systemd Integration

### Service File

```ini
# /etc/systemd/system/sigc.service
[Unit]
Description=sigc Trading Daemon
After=network.target

[Service]
Type=simple
User=sigc
Group=sigc
ExecStart=/usr/local/bin/sigc daemon --config /etc/sigc/sigc.yaml
ExecReload=/usr/local/bin/sigc reload
ExecStop=/usr/local/bin/sigc stop
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Resource limits
MemoryLimit=4G
CPUQuota=80%

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/sigc /var/log/sigc /var/cache/sigc

[Install]
WantedBy=multi-user.target
```

### Management

```bash
# Start
sudo systemctl start sigc

# Stop
sudo systemctl stop sigc

# Status
sudo systemctl status sigc

# Logs
journalctl -u sigc -f

# Enable on boot
sudo systemctl enable sigc
```

## Multiple Daemons

Run separate daemons for different purposes:

```bash
# Production daemon
sigc daemon --config sigc.prod.yaml --port 5555

# Paper trading daemon
sigc daemon --config sigc.paper.yaml --port 5556
```

## Resource Management

### Memory Limits

```yaml
daemon:
  limits:
    max_memory_mb: 4096
    gc_threshold_mb: 3072  # Trigger GC at 3GB
```

### CPU Limits

```yaml
daemon:
  limits:
    max_cpu_pct: 80
    worker_threads: 4
```

### Connection Pools

```yaml
daemon:
  connections:
    data_pool_size: 10
    broker_pool_size: 5
    timeout_seconds: 30
```

## Monitoring

### Prometheus Metrics

```yaml
daemon:
  metrics:
    enabled: true
    port: 9090
    path: /metrics
```

Exposed metrics:

```
sigc_daemon_uptime_seconds
sigc_daemon_memory_bytes
sigc_computation_duration_seconds
sigc_computation_total
sigc_computation_errors_total
sigc_positions_count
sigc_trades_total
```

## Troubleshooting

### Daemon Won't Start

```bash
# Check port availability
lsof -i :5555

# Check permissions
ls -la /var/run/sigc.pid

# Run in foreground for debugging
sigc daemon --foreground --log-level debug
```

### Connection Refused

```bash
# Check if daemon is running
sigc status

# Check RPC address
sigc status --address 127.0.0.1:5555
```

### High Memory Usage

```bash
# Check memory
sigc status --detailed

# Clear cache
sigc cache clear

# Restart daemon
sigc restart
```

## Best Practices

### 1. Use Systemd in Production

Provides automatic restart, logging, and resource management.

### 2. Set Resource Limits

```yaml
daemon:
  limits:
    max_memory_mb: 4096
    max_cpu_pct: 80
```

### 3. Enable Health Checks

```yaml
daemon:
  health:
    enabled: true
    interval_seconds: 30
```

### 4. Monitor Metrics

Set up Prometheus/Grafana for visibility.

### 5. Test Shutdown/Recovery

```bash
# Test graceful shutdown
sigc stop
sigc daemon

# Test crash recovery
kill -9 $(cat /var/run/sigc.pid)
# Daemon should restart via systemd
```

## Next Steps

- [Configuration](configuration.md) - Full configuration reference
- [Scheduling](scheduling.md) - Automated scheduling
- [Monitoring](monitoring.md) - Prometheus metrics
