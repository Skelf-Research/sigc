# sigc Production Roadmap

This roadmap outlines the path from current state to full production deployment.

---

## Architectural Principles

**Always build abstractions first.** Each feature should follow these patterns:

### 1. Trait-Based Design
```rust
// Define the interface
pub trait OrderExecutor: Send + Sync {
    fn submit(&self, order: &Order) -> Result<OrderId>;
    fn cancel(&self, id: OrderId) -> Result<()>;
    fn status(&self, id: OrderId) -> Result<OrderStatus>;
}

// Then implement for specific brokers
pub struct IbkrExecutor { /* ... */ }
pub struct PaperExecutor { /* ... */ }
pub struct SimulatedExecutor { /* ... */ }

impl OrderExecutor for IbkrExecutor { /* ... */ }
impl OrderExecutor for PaperExecutor { /* ... */ }
```

### 2. Registry Pattern
```rust
// Central registry for swappable components
pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn OrderExecutor>>,
}

// Register implementations at runtime
registry.register("paper", Box::new(PaperExecutor::new()));
registry.register("ibkr", Box::new(IbkrExecutor::new(config)));
```

### 3. Configuration-Driven
```rust
// Implementations selected by config, not code
let executor = match config.executor_type {
    "paper" => registry.get("paper"),
    "ibkr" => registry.get("ibkr"),
    _ => registry.get("simulated"),
};
```

### 4. Testable by Default
- Every trait should have a mock/test implementation
- No hardcoded dependencies
- Dependency injection throughout

### Key Abstractions Needed

| Domain | Trait | Implementations |
|--------|-------|-----------------|
| Data | `DataSource` | Postgres, Snowflake, CSV, Parquet, S3 |
| Execution | `OrderExecutor` | Paper, IBKR, FIX, Simulated |
| Alerting | `AlertSink` | Slack, Email, PagerDuty, Console |
| Storage | `ResultStore` | Postgres, SQLite, File, Memory |
| Scheduling | `JobScheduler` | Cron, SystemD, Cloud Scheduler |
| Risk | `RiskCalculator` | Historical, Monte Carlo, Parametric |
| Auth | `Authenticator` | Local, LDAP, OAuth, API Key |

---

## Current State (v0.1)

**Completed:**
- Core DSL and compiler
- Backtesting engine with cost models
- 60+ operators (time-series, cross-sectional, technical)
- Position tracking and export
- Benchmark-relative metrics (alpha, beta, IR)
- Returns attribution (factor, Brinson)
- Parallel execution for large universes
- Position constraints and risk limits
- PostgreSQL connector
- Audit logging and observability
- Comprehensive documentation

**Test Coverage:** 105 tests passing

---

## Phase 1: Data Infrastructure

### 1.1 Async Database Layer
- [ ] Replace sync postgres with sqlx (async)
- [ ] Connection pooling
- [ ] Query timeout handling
- [ ] Prepared statements for performance

### 1.2 Result Persistence
- [ ] Store backtest results in database
- [ ] Historical performance tracking
- [ ] Strategy versioning
- [ ] Result comparison queries

### 1.3 Data Quality
- [ ] Automated data validation on load
- [ ] Missing data detection and alerts
- [ ] Outlier detection
- [ ] Data freshness monitoring

### 1.4 Corporate Actions
- [ ] Stock splits adjustment
- [ ] Dividend handling
- [ ] Spinoffs and mergers
- [ ] Symbol changes

---

## Phase 2: Operations & Monitoring

### 2.1 Alerting
- [ ] Slack integration
- [ ] Email notifications
- [ ] PagerDuty for critical alerts
- [ ] Configurable alert rules

### 2.2 Scheduling
- [ ] Cron-like job scheduling
- [ ] Dependency management between jobs
- [ ] Retry logic with backoff
- [ ] Job history and logging

### 2.3 Secrets Management
- [ ] Environment variable support (done)
- [ ] HashiCorp Vault integration
- [ ] AWS Secrets Manager
- [ ] Encrypted config files

### 2.4 Dashboard
- [ ] Web UI for monitoring
- [ ] Strategy performance views
- [ ] System health metrics
- [ ] Log viewer

---

## Phase 3: Risk Management

### 3.1 Real-time Risk
- [ ] Live position monitoring
- [ ] P&L tracking
- [ ] Exposure dashboards
- [ ] Concentration alerts

### 3.2 Risk Metrics
- [ ] VaR (Value at Risk)
- [ ] CVaR / Expected Shortfall
- [ ] Stress testing scenarios
- [ ] Factor exposure reports

### 3.3 Margin Management
- [ ] Margin utilization tracking
- [ ] Buying power calculations
- [ ] Margin call alerts
- [ ] Cash forecasting

---

## Phase 4: Compliance

### 4.1 Pre-trade Compliance
- [ ] Restricted list checking
- [ ] Position limit enforcement
- [ ] Sector concentration rules
- [ ] Custom compliance rules engine

### 4.2 Regulatory Reporting
- [ ] Form 13F generation
- [ ] MiFID II reporting
- [ ] Transaction reporting
- [ ] Audit trail exports

### 4.3 Access Control
- [ ] User authentication
- [ ] Role-based permissions
- [ ] Strategy access control
- [ ] Audit of user actions

---

## Phase 5: Execution Layer

### 5.1 Paper Trading
- [ ] Simulated order execution
- [ ] Realistic fill simulation
- [ ] Slippage modeling
- [ ] Paper P&L tracking

### 5.2 Broker Integration
- [ ] Interactive Brokers API
- [ ] FIX protocol support
- [ ] Order types (market, limit, etc.)
- [ ] Order status tracking

### 5.3 Order Management
- [ ] Order submission
- [ ] Cancel/modify orders
- [ ] Partial fills handling
- [ ] Order routing logic

### 5.4 Position Reconciliation
- [ ] Broker position sync
- [ ] Discrepancy detection
- [ ] Auto-reconciliation
- [ ] Break alerts

---

## Phase 6: Advanced Features

### 6.1 Real-time Data
- [ ] WebSocket streaming
- [ ] Kafka consumer
- [ ] Incremental signal updates
- [ ] Tick data support

### 6.2 Multi-strategy
- [ ] Strategy orchestration
- [ ] Capital allocation
- [ ] Cross-strategy risk
- [ ] Correlation monitoring

### 6.3 Distributed Computing
- [ ] Multi-machine execution
- [ ] Work distribution
- [ ] Result aggregation
- [ ] Fault tolerance

### 6.4 ML Integration
- [ ] Model inference in signals
- [ ] Feature engineering operators
- [ ] Model versioning
- [ ] Online learning support

---

## Implementation Priority

### Immediate (Next Sprint)
1. Result persistence to database
2. Slack alerting
3. Basic scheduling

### Short-term (1-2 months)
4. Async database with pooling
5. Data quality checks
6. Web dashboard MVP

### Medium-term (3-6 months)
7. Paper trading
8. Interactive Brokers integration
9. Real-time risk monitoring
10. Pre-trade compliance

### Long-term (6+ months)
11. Full OMS
12. Regulatory reporting
13. Distributed computing
14. ML integration

---

## Success Criteria

### Phase 1 Complete When:
- Can persist and query historical backtests
- Data validation catches 95% of issues
- Corporate actions handled automatically

### Phase 2 Complete When:
- Strategies run on schedule without manual intervention
- Alerts notify team within 1 minute of issues
- All secrets managed securely

### Phase 3 Complete When:
- Risk dashboard shows real-time exposures
- VaR calculated daily
- Margin utilization tracked

### Phase 4 Complete When:
- Pre-trade checks block violating orders
- Regulatory reports generated automatically
- Full audit trail available

### Phase 5 Complete When:
- Can execute trades through broker API
- Positions reconcile with broker
- Paper trading validates strategies

---

## Dependencies

```
Phase 1 (Data) ─────┬──> Phase 2 (Operations)
                    │
                    └──> Phase 3 (Risk)
                              │
                              v
                         Phase 4 (Compliance)
                              │
                              v
                         Phase 5 (Execution)
                              │
                              v
                         Phase 6 (Advanced)
```

---

## Contributing

Each feature should:
1. **Define the trait first** - Interface before implementation
2. **Include a mock/test implementation** - For testing without external deps
3. **Add to registry** - Make it swappable at runtime
4. **Have comprehensive tests** - Unit and integration
5. **Update documentation** - Usage examples
6. **Maintain backwards compatibility** - Don't break existing code

### Example: Adding a New Alert Sink

```rust
// 1. Define trait (if not exists)
pub trait AlertSink: Send + Sync {
    fn send(&self, alert: &Alert) -> Result<()>;
    fn name(&self) -> &str;
}

// 2. Implement for your service
pub struct PagerDutySink { api_key: String }

impl AlertSink for PagerDutySink {
    fn send(&self, alert: &Alert) -> Result<()> { /* ... */ }
    fn name(&self) -> &str { "pagerduty" }
}

// 3. Add mock for testing
pub struct MockAlertSink { sent: Arc<Mutex<Vec<Alert>>> }

// 4. Register in AlertRegistry
registry.register("pagerduty", Box::new(PagerDutySink::new(key)));

// 5. Test both real and mock
#[test]
fn test_pagerduty_sink() { /* ... */ }
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for full guidelines.
