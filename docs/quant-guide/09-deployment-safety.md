# Chapter 9: Deployment & Safety

This chapter covers production deployment infrastructure and critical trading safety systems.

## Trading Safety Systems

Safety systems protect against catastrophic losses from bugs, market events, or system failures.

### Kill Switch

The global kill switch immediately halts all trading.

```rust
use sig_runtime::{activate_kill_switch, deactivate_kill_switch, is_kill_switch_active};

// Check before any trade
if is_kill_switch_active() {
    return Err("Trading halted - kill switch active");
}

// In emergency
activate_kill_switch();  // Stops ALL trading immediately

// After resolution
deactivate_kill_switch();
```

**When to use**:
- System malfunction detected
- Unexpected market conditions
- Data feed failure
- Manual intervention needed

### Circuit Breakers

Circuit breakers prevent cascading failures by stopping requests after repeated errors.

```rust
use sig_runtime::{CircuitBreaker, CircuitBreakerConfig, CircuitState};

let config = CircuitBreakerConfig {
    failure_threshold: 5,           // Open after 5 failures
    reset_timeout: Duration::from_secs(60), // Try again after 60s
    success_threshold: 3,           // Close after 3 successes
};

let breaker = CircuitBreaker::new("api", config);

// Before making request
if !breaker.allow_request() {
    return Err("Circuit breaker open");
}

// After request
match result {
    Ok(_) => breaker.record_success(),
    Err(_) => breaker.record_failure(),
}
```

**States**:
- **Closed**: Normal operation
- **Open**: Blocking all requests
- **Half-Open**: Testing if service recovered

### Position Limits

Enforce maximum exposure to prevent concentration risk.

```rust
use sig_runtime::{PositionLimits, PositionLimitEnforcer};

let limits = PositionLimits {
    max_position_value: 1_000_000.0,  // $1M per position
    max_position_pct: 0.10,           // 10% of portfolio
    max_gross_exposure: 2.0,          // 200% gross
    max_net_exposure: 1.0,            // 100% net
    max_sector_exposure: 0.30,        // 30% per sector
    max_positions: 100,               // Max 100 positions
};

let enforcer = PositionLimitEnforcer::new(limits);

// Set sector mappings
enforcer.set_sector("AAPL", "technology");
enforcer.set_sector("MSFT", "technology");

// Check before trading
enforcer.check_trade("AAPL", 50_000.0, portfolio_value)?;

// Update after trade executes
enforcer.update_position("AAPL", 50_000.0);
```

### Order Validation

Validate orders before submission.

```rust
use sig_runtime::{OrderValidator, OrderValidationRules, Order, OrderSide, OrderType};

let rules = OrderValidationRules {
    max_order_value: 500_000.0,    // Max $500K per order
    max_adv_pct: 0.10,             // Max 10% of ADV
    min_order_value: 100.0,        // Min $100
    max_price_deviation: 0.05,     // Max 5% from reference
    blocked_symbols: vec!["BANNED".to_string()],
};

let validator = OrderValidator::new(rules);

// Set market data
validator.set_adv("AAPL", 5_000_000.0);  // $5M ADV
validator.set_reference_price("AAPL", 150.0);

// Validate order
let order = Order {
    symbol: "AAPL".to_string(),
    side: OrderSide::Buy,
    quantity: 100.0,
    price: Some(151.0),
    order_type: OrderType::Limit,
};

validator.validate(&order)?;
```

### Rate Limiting

Prevent exceeding API rate limits.

```rust
use sig_runtime::RateLimiter;

// 100 requests per second
let limiter = RateLimiter::new("api", 100, 100);

// Before each request
if !limiter.try_acquire() {
    return Err("Rate limit exceeded");
}

// Or block until token available
limiter.acquire();  // Blocks if needed
```

### Safety Manager

Combine all safety systems:

```rust
use sig_runtime::{SafetyManager, PositionLimits, OrderValidationRules};

let mut manager = SafetyManager::new(
    PositionLimits::default(),
    OrderValidationRules::default(),
);

// Add circuit breakers
manager.add_circuit_breaker("exchange", CircuitBreakerConfig::default());
manager.add_circuit_breaker("data_feed", CircuitBreakerConfig::default());

// Add rate limiters
manager.add_rate_limiter("orders", 100, 100);  // 100/sec
manager.add_rate_limiter("quotes", 1000, 1000); // 1000/sec

// Pre-trade check (validates everything)
manager.pre_trade_check(&order, portfolio_value)?;

// Get system status
let status = manager.system_status();
println!("Kill switch: {}", status.kill_switch_active);
println!("Gross exposure: {}", status.gross_exposure);
```

## Docker Deployment

### Building the Image

```bash
# Build Docker image
docker build -t sigc:latest .

# Run container
docker run -d \
  --name sigc \
  -p 8080:8080 \
  -p 9090:9090 \
  -v ./data:/app/data \
  -v ./config:/app/config \
  sigc:latest daemon
```

### Docker Compose

Full stack with monitoring:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f sigc

# Stop services
docker-compose down
```

Services included:
- **sigc**: Main application
- **postgres**: Database
- **prometheus**: Metrics collection
- **grafana**: Dashboards

### Environment Variables

```bash
# Database
SIGC_DB_HOST=postgres
SIGC_DB_PORT=5432
SIGC_DB_NAME=sigc
SIGC_DB_USER=sigc
SIGC_DB_PASSWORD=secret

# Logging
RUST_LOG=info

# Directories
SIGC_DATA_DIR=/app/data
SIGC_CONFIG_DIR=/app/config
SIGC_LOG_DIR=/app/logs
SIGC_CACHE_DIR=/app/cache
```

## Monitoring

### Metrics

sigc exports Prometheus metrics:

- `sigc_runs_total` - Total strategy runs
- `sigc_run_duration_seconds` - Execution time histogram
- `sigc_portfolio_value` - Current portfolio value
- `sigc_portfolio_return` - Current return
- `sigc_portfolio_drawdown` - Current drawdown
- `sigc_orders_total` - Orders submitted
- `sigc_errors_total` - Error count

### Health Checks

```bash
# Check if service is healthy
curl http://localhost:8080/health

# Response
{
  "status": "healthy",
  "version": "0.6.0",
  "uptime_seconds": 3600,
  "checks": {
    "database": "ok",
    "data_feed": "ok"
  }
}
```

### Grafana Dashboard

Key panels:
- Portfolio equity curve
- Daily P&L
- Rolling Sharpe ratio
- Position heatmap
- Risk metrics gauges
- Alert history

## CI/CD Pipeline

### GitHub Actions Workflow

The CI pipeline:
1. **Lint**: Check formatting and clippy warnings
2. **Test**: Run all tests
3. **Build**: Create release binary
4. **Docker**: Build and push image
5. **Release**: Create GitHub release

### Deployment Steps

```bash
# 1. Push to main (triggers CI)
git push origin main

# 2. Create release tag
git tag v0.6.0
git push origin v0.6.0

# 3. Deploy to production
docker-compose pull
docker-compose up -d
```

## Production Configuration

### Example Config

```toml
# config/production.toml

[general]
environment = "production"
strategy_name = "momentum"

[database]
host = "${SIGC_DB_HOST}"
port = 5432
pool_size = 20

[safety]
max_position_value = 1000000
max_position_pct = 0.05
max_gross_exposure = 1.5
kill_switch_enabled = true

[execution]
parallel_workers = 8
simd_enabled = true

[alerts]
slack_webhook = "${SLACK_WEBHOOK_URL}"
drawdown_warning = 0.05
drawdown_critical = 0.10

[monitoring]
metrics_port = 9090
health_port = 8080
```

## Best Practices

### Safety Checklist

Before going live:

- [ ] Kill switch tested and accessible
- [ ] Position limits configured appropriately
- [ ] Circuit breakers on all external services
- [ ] Rate limits respect API quotas
- [ ] Order validation rules set
- [ ] Monitoring and alerting configured
- [ ] Runbook for common incidents

### Incident Response

1. **Detect**: Monitoring alerts
2. **Triage**: Assess severity
3. **Mitigate**: Kill switch if needed
4. **Investigate**: Review logs
5. **Resolve**: Fix root cause
6. **Post-mortem**: Document learnings

### Common Failures

| Failure | Detection | Response |
|---------|-----------|----------|
| Data feed down | Circuit breaker opens | Use backup feed |
| Exchange error | Order validation fails | Retry with backoff |
| Position breach | Limit check fails | Reject trade |
| High drawdown | Monitoring alert | Review strategy |

## Security

### Secrets Management

- Use environment variables for secrets
- Never commit credentials to git
- Rotate passwords regularly
- Use vault for production

### Network Security

- Use TLS for all connections
- Restrict database access
- Firewall non-essential ports
- VPN for sensitive operations

### Audit Logging

All actions are logged:

```rust
// Automatic audit log
[2024-01-15 17:00:00] ORDER symbol=AAPL side=BUY qty=100 price=150.00
[2024-01-15 17:00:01] FILL symbol=AAPL side=BUY qty=100 price=150.05
[2024-01-15 17:00:02] POSITION symbol=AAPL qty=100 value=15005.00
```

## Disaster Recovery

### Backup Strategy

```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d)

# Backup database
pg_dump sigc > backup/db-$DATE.sql

# Backup configuration
tar -czf backup/config-$DATE.tar.gz config/

# Backup logs
tar -czf backup/logs-$DATE.tar.gz logs/

# Upload to S3
aws s3 sync backup/ s3://sigc-backups/
```

### Recovery Procedure

1. Stop trading (kill switch)
2. Restore from backup
3. Verify data integrity
4. Test in paper trading
5. Resume live trading

## Key Takeaways

1. **Safety first**: Multiple layers of protection
2. **Kill switch**: Always accessible emergency stop
3. **Monitor everything**: Can't fix what you can't see
4. **Automate deployment**: Reduces human error
5. **Test recovery**: Practice restoring from backups
6. **Document incidents**: Learn from failures

## Exercises

1. **Kill switch drill**: Practice activating and deactivating the kill switch.

2. **Circuit breaker test**: Simulate failures and verify the breaker opens.

3. **Position limit test**: Try to exceed limits and verify rejection.

4. **Monitoring setup**: Configure Grafana dashboard with key metrics.

5. **Incident simulation**: Practice responding to a simulated outage.

## Resources

- [Docker Documentation](https://docs.docker.com/)
- [Prometheus Guide](https://prometheus.io/docs/)
- [Grafana Dashboards](https://grafana.com/docs/)
- [Production Features](../advanced/production-features.md)
