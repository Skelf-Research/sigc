# Production Features

This guide covers advanced features for production deployment of sigc strategies.

## Position Tracking & Export

### Position History

Track daily positions and export for compliance or analysis:

```rust
use sig_runtime::{Runtime, PositionHistory};

let report = runtime.execute(&plan)?;

// Access position history
if let Some(positions) = &report.positions {
    // Export to CSV
    let csv = positions.to_csv();
    std::fs::write("positions.csv", csv)?;

    // Get weights for specific date
    let weights = positions.weights_on("2024-01-15");
}

// Access returns series
let daily_returns = &report.returns_series;
```

### Enhanced Metrics

The `BacktestMetrics` now includes:

- `sortino_ratio` - Downside risk-adjusted return
- `calmar_ratio` - Return / max drawdown
- `win_rate` - Percentage of positive days
- `profit_factor` - Average win / average loss

## Benchmark-Relative Analysis

Compare strategy performance against a benchmark:

```rust
use sig_runtime::BenchmarkAnalyzer;

let metrics = BenchmarkAnalyzer::analyze(
    &portfolio_returns,
    &benchmark_returns
)?;

println!("Alpha: {:.2}%", metrics.alpha * 100.0);
println!("Beta: {:.2}", metrics.beta);
println!("Information Ratio: {:.2}", metrics.information_ratio);
println!("Tracking Error: {:.2}%", metrics.tracking_error * 100.0);
println!("Correlation: {:.2}", metrics.correlation);
println!("Up Capture: {:.0}%", metrics.up_capture * 100.0);
println!("Down Capture: {:.0}%", metrics.down_capture * 100.0);
```

### Rolling Metrics

Calculate rolling beta and correlation:

```rust
let rolling_beta = BenchmarkAnalyzer::rolling_beta(
    &portfolio_returns,
    &benchmark_returns,
    63  // 63-day window
);

let rolling_corr = BenchmarkAnalyzer::rolling_correlation(
    &portfolio_returns,
    &benchmark_returns,
    63
);
```

## Returns Attribution

Decompose returns by factor and sector.

### Factor Attribution

```rust
use sig_runtime::{AttributionAnalyzer, SectorMapping};

let mut analyzer = AttributionAnalyzer::new();

// Add factors
analyzer.add_factor("momentum", momentum_returns);
analyzer.add_factor("value", value_returns);
analyzer.add_factor("size", size_returns);

let result = analyzer.analyze(&portfolio_returns, None, None)?;

println!("Factor Contributions:");
for (factor, contrib) in &result.factor_contributions {
    println!("  {}: {:.2}%", factor, contrib * 100.0);
}
println!("Alpha: {:.2}%", result.alpha * 100.0);
println!("R-squared: {:.2}", result.r_squared);
```

### Brinson Attribution

Decompose active return into allocation and selection:

```rust
let mut sectors = SectorMapping::new();
sectors.add("AAPL", "Tech").add("MSFT", "Tech").add("JPM", "Finance");

let brinson = analyzer.brinson_attribution(
    &portfolio_weights,
    &benchmark_weights,
    &portfolio_returns,
    &benchmark_returns,
    &sectors
);

println!("Allocation Effect: {:.2}%", brinson.total_allocation * 100.0);
println!("Selection Effect: {:.2}%", brinson.total_selection * 100.0);
println!("Interaction Effect: {:.2}%", brinson.total_interaction * 100.0);
println!("Total Active Return: {:.2}%", brinson.total_active_return() * 100.0);
```

## Parallel Execution

Scale to large universes with parallel processing.

### Basic Parallel Execution

```rust
use sig_runtime::{ParallelExecutor, ParallelConfig, execute_parallel};

// Quick parallel execution
let results = execute_parallel(&ir, &prices, "prices")?;

// Custom configuration
let config = ParallelConfig::default()
    .with_threads(8)        // Use 8 threads
    .with_chunk_size(100);  // Process 100 assets per batch

let executor = ParallelExecutor::new(config);
let results = executor.execute_parallel(&ir, &prices, "prices")?;
```

### Batched Execution for Large Universes

For very large universes (1000+ assets), use batched execution to manage memory:

```rust
let config = ParallelConfig::default().with_chunk_size(200);
let executor = ParallelExecutor::new(config);

// Processes in batches of 200 assets
let results = executor.execute_batched(&ir, &prices, "prices")?;

for result in results {
    println!("{}: {} signals computed", result.asset, result.signal.len());
}
```

## Position Constraints

Enforce risk limits and position constraints.

### Built-in Constraint Sets

```rust
use sig_runtime::{ConstraintSet, ConstraintEnforcer};

// Long-short equity preset
let constraints = ConstraintSet::long_short_equity();
// Includes: 5% max weight, 200% max gross, 0% target net, 30% max turnover

// Long-only preset
let constraints = ConstraintSet::long_only();
// Includes: 10% max weight, 100% gross, no shorts, 50% max turnover
```

### Custom Constraints

```rust
use sig_runtime::{ConstraintSet, PositionConstraint, PortfolioConstraint};

let constraints = ConstraintSet::new()
    // Position-level
    .add_position(PositionConstraint::MaxWeight(0.05))
    .add_position(PositionConstraint::MinWeight(0.001))
    .add_position(PositionConstraint::MaxShortWeight(0.03))

    // Portfolio-level
    .add_portfolio(PortfolioConstraint::MaxGrossExposure(1.5))
    .add_portfolio(PortfolioConstraint::TargetNetExposure(0.0))
    .add_portfolio(PortfolioConstraint::MaxPositions(100))
    .add_portfolio(PortfolioConstraint::LongShortRatio { min: 0.8, max: 1.2 })

    // Turnover
    .with_turnover(0.25);  // 25% max one-way turnover
```

### Applying Constraints

```rust
let enforcer = ConstraintEnforcer::new(constraints);

// Apply to weights
let adjusted = enforcer.apply(&raw_weights, Some(&prev_weights))?;

// Validate (returns list of violations)
let violations = enforcer.validate(&weights);
if !violations.is_empty() {
    for v in violations {
        eprintln!("Constraint violation: {}", v);
    }
}
```

### Sector Constraints

```rust
use sig_runtime::SectorConstraint;

let constraints = ConstraintSet::new()
    .add_sector(SectorConstraint {
        sector: "Technology".to_string(),
        max_weight: Some(0.30),  // Max 30% in tech
        min_weight: None,
        max_active_weight: Some(0.10),  // Max 10% over/under benchmark
    })
    .add_sector(SectorConstraint {
        sector: "Finance".to_string(),
        max_weight: Some(0.25),
        min_weight: Some(0.05),  // Min 5% in finance
        max_active_weight: None,
    });

let mut sector_map = HashMap::new();
sector_map.insert("AAPL".to_string(), "Technology".to_string());
sector_map.insert("JPM".to_string(), "Finance".to_string());

let enforcer = ConstraintEnforcer::new(constraints)
    .with_sectors(sector_map);
```

## SQL Database Connectors

Load data directly from PostgreSQL.

### PostgreSQL Connection

```rust
use sig_runtime::{SqlConnector, Connector};

let connector = SqlConnector::postgres(
    "localhost",  // host
    5432,         // port
    "market_data", // database
    "user",       // username
    "password"    // password
);

// Load data with SQL query
let prices = connector.load(
    "SELECT date, symbol, close FROM daily_prices
     WHERE date >= '2020-01-01'
     ORDER BY date"
)?;

// Check if connection is available
if connector.is_available() {
    println!("Connected to database");
}

// List tables
let tables = connector.list_tables()?;

// Query count
let count: i64 = connector.query_count("SELECT COUNT(*) FROM daily_prices")?;
```

### Environment-Based Configuration

```rust
use sig_runtime::ConnectorEnv;

// Uses PGHOST, PGPORT, PGDATABASE, PGUSER, PGPASSWORD
if let Some(connector) = ConnectorEnv::postgres_from_env() {
    let data = connector.load("SELECT * FROM prices")?;
}
```

### Connector Registry

Manage multiple data sources:

```rust
use sig_runtime::ConnectorRegistry;

let mut registry = ConnectorRegistry::new();

registry.register("prod_db", Box::new(
    SqlConnector::postgres("prod.db.com", 5432, "market", "user", "pass")
));

registry.register("s3_data", Box::new(
    CloudConnector::s3("my-bucket", "us-east-1")
));

// Load from registered connector
let prices = registry.load("prod_db", "SELECT * FROM prices")?;
let factors = registry.load("s3_data", "factors/momentum.parquet")?;
```

## Audit Logging

Track all operations for compliance:

```rust
use sig_runtime::{init_audit_logger, audit_log, AuditEvent};

// Initialize logger
init_audit_logger("/var/log/sigc/audit.log")?;

// Log events
audit_log(AuditEvent::BacktestStart {
    plan_hash: "abc123".to_string(),
});

audit_log(AuditEvent::BacktestComplete {
    plan_hash: "abc123".to_string(),
    duration_ms: 1500,
    total_return: 0.15,
});
```

## Observability

Export Prometheus metrics:

```rust
use sig_runtime::metrics;

// Increment counters
metrics().increment("backtests_run");
metrics().increment_by("signals_computed", 1000);

// Set gauges
metrics().set_gauge("active_strategies", 5);

// Record timings
let _timer = metrics().start_timer("backtest_duration");
// ... run backtest
// timer automatically records duration when dropped

// Export for Prometheus
let prometheus_output = metrics().export_prometheus();
```

## Result Persistence

Store and query backtest results for historical analysis.

### Result Store Trait

```rust
use sig_runtime::{ResultStore, MemoryResultStore, PostgresResultStore, ResultMetadata, ResultQuery, generate_result_id};

// In-memory store for testing
let store = MemoryResultStore::new();

// PostgreSQL store for production
let store = PostgresResultStore::new("localhost", 5432, "sigc", "user", "pass");
store.init_schema()?;  // Create tables

// Store a result
let metadata = ResultMetadata {
    id: generate_result_id(),
    strategy_name: "momentum".to_string(),
    strategy_version: Some("1.0".to_string()),
    created_at: "2024-01-01 12:00:00".to_string(),
    start_date: "2023-01-01".to_string(),
    end_date: "2023-12-31".to_string(),
    total_return: report.metrics.total_return,
    sharpe_ratio: report.metrics.sharpe_ratio,
    max_drawdown: report.metrics.max_drawdown,
    tags: HashMap::new(),
};

let id = store.store(&report, metadata)?;

// Query results
let query = ResultQuery::new()
    .strategy("momentum")
    .min_sharpe(1.0)
    .order_by("sharpe_ratio")
    .limit(10);

let results = store.query(&query)?;
```

## Alerting System

Send alerts to multiple channels using the AlertSink trait.

### Alert Types

```rust
use sig_runtime::{Alert, AlertSeverity, AlertManager, ConsoleAlertSink, SlackAlertSink, AlertRule};

// Create alerts
let alert = Alert::info("Backtest Complete", "Strategy XYZ finished with 15% return")
    .with_source("backtest_runner")
    .with_tag("strategy", "momentum");

let alert = Alert::error("Data Missing", "No price data for AAPL on 2024-01-15");
let alert = Alert::critical("Risk Limit Breached", "Position size exceeds 5% limit");
```

### Slack Integration

```rust
// Create Slack sink from webhook URL
let slack = SlackAlertSink::new("https://hooks.slack.com/services/...")
    .with_channel("#alerts")
    .with_username("sigc-bot");

// Or from environment
let slack = SlackAlertSink::from_env();  // Uses SLACK_WEBHOOK_URL

slack.send(&alert)?;
```

### Alert Manager

Route alerts to different sinks based on severity:

```rust
let mut manager = AlertManager::new();

// Register sinks
manager.register("console", Box::new(ConsoleAlertSink::new()));
manager.register("slack", Box::new(SlackAlertSink::from_env().unwrap()));

// Route critical alerts to Slack
manager.add_rule(AlertRule {
    min_severity: AlertSeverity::Critical,
    sinks: vec!["slack".to_string()],
    tag_filter: None,
});

// Route all alerts to console
manager.add_rule(AlertRule {
    min_severity: AlertSeverity::Info,
    sinks: vec!["console".to_string()],
    tag_filter: None,
});

// Send alert - automatically routed
manager.send(&alert)?;
```

## Job Scheduling

Schedule strategy runs using the JobScheduler trait.

### Creating Jobs

```rust
use sig_runtime::{Job, Schedule, JobScheduler, MemoryScheduler};

// Create a job
let job = Job::new("daily-momentum", "Daily Momentum Run", Schedule::daily_at(9), "sigc")
    .with_args(vec!["run".to_string(), "momentum.sig".to_string()])
    .with_max_retries(3)
    .with_timeout(3600)
    .with_tag("env", "prod");
```

### Schedule Types

```rust
// Run once at specific timestamp
Schedule::Once(1704110400);

// Run every N seconds
Schedule::Interval(3600);  // hourly

// Cron expression
Schedule::Cron("0 9 * * 1-5".to_string());  // 9 AM weekdays

// Helpers
Schedule::every_minutes(30);
Schedule::every_hours(2);
Schedule::daily_at(9);

// Market-aware (requires calendar)
Schedule::MarketOpen;
Schedule::MarketClose;
```

### Using the Scheduler

```rust
let scheduler = MemoryScheduler::new();

// Submit job
let id = scheduler.submit(job)?;

// Check status
let status = scheduler.status(&id)?;

// List all jobs
let jobs = scheduler.list()?;

// Cancel job
scheduler.cancel(&id)?;
```

## Async Database Layer

High-performance async database access with connection pooling using sqlx.

### Async PostgreSQL Connector

```rust
use sig_runtime::{AsyncPgConnector, AsyncConnectorConfig};

// Create config
let config = AsyncConnectorConfig::new("localhost", 5432, "market_data", "user", "pass")
    .with_max_connections(20)
    .with_query_timeout(std::time::Duration::from_secs(60));

// Create connector (async)
let connector = AsyncPgConnector::new(config).await?;

// Load data
let df = connector.load("SELECT date, symbol, close FROM prices").await?;

// Query with timeout
let df = connector.load_with_timeout(
    "SELECT * FROM large_table",
    std::time::Duration::from_secs(30)
).await?;

// Check health
if connector.is_healthy().await {
    println!("Connection pool healthy");
}

// Get pool stats
let stats = connector.pool_stats();
println!("Pool size: {}, Idle: {}", stats.size, stats.idle);
```

### From Environment Variables

```rust
// Uses PGHOST, PGPORT, PGDATABASE, PGUSER, PGPASSWORD
let connector = AsyncPgConnector::from_env().await?;
```

### Async Connector Registry

```rust
use sig_runtime::AsyncConnectorRegistry;

let mut registry = AsyncConnectorRegistry::new();
registry.register("prod", connector);

let df = registry.load("prod", "SELECT * FROM prices").await?;
```

## Corporate Actions

Handle stock splits, dividends, and symbol changes for accurate backtesting.

### Corporate Action Types

```rust
use sig_runtime::{CorporateAction, ActionType, StandardAdjuster, CorporateActionStore};

// Create actions
let split = CorporateAction::split("AAPL", "2020-08-31", 4.0);
let dividend = CorporateAction::dividend("MSFT", "2024-01-15", 0.75);
let symbol_change = CorporateAction::symbol_change("FB", "META", "2022-10-28");

// Full action types
let merger = CorporateAction {
    symbol: "TARGET".to_string(),
    date: "2024-06-01".to_string(),
    action_type: ActionType::Merger {
        acquirer: "BUYER".to_string(),
        ratio: 0.5,  // 0.5 shares of acquirer per share
        cash: 10.0,  // $10 cash per share
    },
    ex_date: Some("2024-06-01".to_string()),
    record_date: None,
    payment_date: None,
};
```

### Adjusting Prices

```rust
// Standard adjuster (adjusts OHLCV)
let adjuster = StandardAdjuster::new()
    .with_date_column("date")
    .with_price_columns(vec!["open".into(), "high".into(), "low".into(), "close".into()])
    .with_volume_column("volume")
    .without_dividends();  // Optional: skip dividend adjustment

let actions = vec![
    CorporateAction::split("AAPL", "2020-08-31", 4.0),
    CorporateAction::split("AAPL", "2014-06-09", 7.0),
];

let adjusted_df = adjuster.adjust_prices(&df, &actions)?;
```

### Corporate Action Store

```rust
let mut store = CorporateActionStore::new();

// Add actions
store.add(CorporateAction::split("AAPL", "2020-08-31", 4.0));
store.add(CorporateAction::dividend("AAPL", "2024-01-15", 0.24));

// Get actions for a symbol
let actions = store.get("AAPL");

// Get actions in date range
let recent = store.get_in_range("AAPL", "2020-01-01", "2024-12-31");
```

### Symbol Mapper

```rust
use sig_runtime::SymbolMapper;

let mut mapper = SymbolMapper::new();
mapper.add_change("FB", "META", "2022-10-28");

// Get current symbol
assert_eq!(mapper.get_current("FB"), "META");

// Get historical symbol at date
assert_eq!(mapper.get_at_date("META", "2022-01-01"), "FB");
```

## Data Quality Validation

Validate data before backtesting to catch issues early.

### Built-in Validators

```rust
use sig_runtime::{
    DataQualityValidator, MissingDataCheck, OutlierCheck,
    OutlierMethod, FreshnessCheck, DuplicateCheck
};

// Missing data check
let missing_check = MissingDataCheck {
    max_missing_pct: 5.0,  // Max 5% missing values
    columns: vec!["close".into(), "volume".into()],
};

// Outlier detection (IQR method)
let outlier_check = OutlierCheck {
    method: OutlierMethod::IQR { multiplier: 3.0 },
    columns: vec!["close".into()],
    max_outlier_pct: 1.0,
};

// Z-score method
let outlier_check = OutlierCheck {
    method: OutlierMethod::ZScore { threshold: 3.0 },
    columns: vec!["returns".into()],
    max_outlier_pct: 1.0,
};

// Freshness check
let freshness_check = FreshnessCheck {
    date_column: "date".to_string(),
    max_staleness_days: 1,
};

// Duplicate check
let dup_check = DuplicateCheck {
    key_columns: vec!["date".into(), "symbol".into()],
};
```

### Composite Validator

```rust
let mut validator = DataQualityValidator::new();
validator.add_validator(Box::new(missing_check));
validator.add_validator(Box::new(outlier_check));
validator.add_validator(Box::new(freshness_check));

let result = validator.validate(&df)?;

if result.passed {
    println!("Data validation passed");
} else {
    for issue in &result.issues {
        println!("{:?}: {} - {}", issue.severity, issue.check_name, issue.message);
    }
}
```

## SIMD-Optimized Kernels

High-performance kernel implementations for compute-intensive operations.

### Using Optimized Kernels

```rust
use sig_runtime::{
    rolling_mean_simd, rolling_std_simd, cumsum_simd, ema_simd,
    KernelDispatcher, KernelConfig
};

// Direct usage
let mean = rolling_mean_simd(&series, 20)?;
let std = rolling_std_simd(&series, 20)?;
let cumsum = cumsum_simd(&series)?;
let ema = ema_simd(&series, 10)?;

// Using dispatcher (auto-selects based on data size)
let config = KernelConfig {
    min_simd_size: 64,      // Use SIMD for arrays >= 64 elements
    use_parallel: true,
    num_workers: 8,
};

let dispatcher = KernelDispatcher::new(config);
let mean = dispatcher.rolling_mean(&series, 20)?;
let std = dispatcher.rolling_std(&series, 20)?;
```

### Batch Operations

```rust
use sig_runtime::batch_rolling_mean;

// Process multiple series in parallel
let series_vec = vec![s1, s2, s3, s4];
let results = batch_rolling_mean(&series_vec, 20)?;
```

## Memory-Mapped Data Loading

Efficient loading of large datasets using memory mapping and lazy evaluation.

### MmapLoader

```rust
use sig_runtime::{MmapLoader, DataStream};

let loader = MmapLoader::new()
    .with_chunk_size(100_000)
    .with_lazy(true);

// Load Parquet
let df = loader.load_parquet("data/prices.parquet")?;

// Load specific columns only
let df = loader.load_parquet_columns(
    "data/prices.parquet",
    &["date", "close", "volume"]
)?;

// Lazy loading with filtering
let lf = loader.load_parquet_lazy("data/prices.parquet")?;
let df = lf
    .filter(col("date").gt(lit("2020-01-01")))
    .select([col("date"), col("close")])
    .collect()?;
```

### Data Streaming

```rust
let lf = loader.load_csv_lazy("data/large.csv")?;
let stream = DataStream::new(lf)
    .filter(col("volume").gt(lit(1000000)))
    .select(vec![col("date"), col("symbol"), col("close")])
    .sort("date", false)
    .limit(1000);

let df = stream.collect()?;
```

### MmapCache

```rust
use sig_runtime::MmapCache;

let cache = MmapCache::new("/tmp/sigc_cache")?;

// Cache DataFrame
cache.cache("prices_2024", &df)?;

// Load from cache
if let Some(df) = cache.load("prices_2024")? {
    // Use cached data
}

// Check if cached
if cache.contains("prices_2024") {
    // ...
}

// Clear cache
cache.clear()?;
```

## Incremental Computation

Efficient updates when new data arrives without full recomputation.

### Stateful Computations

```rust
use sig_runtime::{
    RollingMeanState, RollingStdState, EmaState,
    CumsumState, RsiState, IncrementalCompute
};

// Rolling mean
let mut state = RollingMeanState::new(20);
for value in new_values {
    let mean = state.update(value);
    println!("Current mean: {}", mean);
}

// Batch update
let results = state.update_batch(&values);

// EMA
let mut ema_state = EmaState::new(10);
let ema = ema_state.update(100.0);

// RSI
let mut rsi_state = RsiState::new(14);
let rsi = rsi_state.update(105.0);
```

### Incremental Compute Manager

```rust
let mut compute = IncrementalCompute::new();

// Register computations
compute.register_rolling_mean("sma_20", 20);
compute.register_rolling_std("std_20", 20);
compute.register_ema("ema_10", 10);
compute.register_rsi("rsi_14", 14);

// Update with new values
let mean = compute.update_rolling_mean("sma_20", 105.0)?;
let std = compute.update_rolling_std("std_20", 105.0)?;
let ema = compute.update_ema("ema_10", 105.0)?;
let rsi = compute.update_rsi("rsi_14", 105.0)?;

// Batch update
let means = compute.update_batch("sma_20", &new_values, "rolling_mean")?;

// Reset state
compute.reset("sma_20", "rolling_mean");
```

### Incremental DataFrame Processor

```rust
use sig_runtime::IncrementalProcessor;

let mut processor = IncrementalProcessor::new();

// Initialize with historical data
processor.initialize(historical_df);

// Append new data as it arrives
processor.append(new_data)?;

// Register computations
processor.register_computation("close", "sma_20", "rolling_mean", &[20]);
processor.register_computation("close", "rsi", "rsi", &[14]);

// Get current data
let df = processor.data();
```

## Best Practices

1. **Position Limits**: Always use position constraints in production
2. **Turnover Control**: Set realistic turnover limits to manage costs
3. **Benchmark Tracking**: Monitor tracking error for benchmark-relative strategies
4. **Attribution**: Regular factor attribution helps identify return sources
5. **Parallel Execution**: Use for universes > 100 assets
6. **Audit Trail**: Enable audit logging for compliance
7. **Monitoring**: Export metrics to your observability stack
8. **Result Persistence**: Store all backtest results for analysis
9. **Alerting**: Set up alerts for critical events and failures
10. **Scheduling**: Automate regular strategy runs
11. **Data Quality**: Validate data before backtesting to catch issues early
12. **Corporate Actions**: Always adjust for splits and dividends
13. **Async DB**: Use connection pooling for high-throughput database access
14. **Memory Mapping**: Use MmapLoader for datasets > 1GB
15. **Incremental Updates**: Use stateful computations for streaming data
