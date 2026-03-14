//! Runtime execution engine for sigc
//!
//! Executes IR and manages data ingestion.

pub mod alerts;
pub mod attribution;
pub mod audit;
pub mod backtest;
pub mod benchmark;
pub mod config;
pub mod connectors;
pub mod connectors_async;
pub mod constraints;
pub mod corporate_actions;
pub mod costs;
pub mod data;
pub mod data_quality;
pub mod engine;
pub mod factors;
pub mod incremental;
pub mod integrations;
pub mod kernels;
pub mod metrics;
pub mod mmap_data;
pub mod optimize;
pub mod panel;
pub mod parallel;
pub mod portfolio_opt;
pub mod regime;
pub mod reporting;
pub mod result_store;
pub mod risk_models;
pub mod safety;
pub mod scheduler;
pub mod simd_kernels;
pub mod universe;
pub mod viz;
pub mod walk_forward;

use polars::prelude::*;
use sig_types::{BacktestPlan, BacktestReport, Ir, Result};
use std::collections::HashMap;

pub use attribution::{AttributionAnalyzer, AttributionResult, SectorMapping, BrinsonResult};
pub use backtest::Backtester;
pub use connectors::{Connector, SqlConnector, CloudConnector, ConnectorRegistry, ConnectorEnv};
pub use constraints::{ConstraintSet, ConstraintEnforcer, PositionConstraint, PortfolioConstraint, SectorConstraint, TurnoverConstraint};
pub use costs::{CostModel, ImpactModel, TradeCost, PortfolioCostCalculator, PortfolioCost};
pub use data::{DataFormat, DataLoader, DataManager, DataSource, DateRange};
pub use engine::Engine;
pub use optimize::{GridSearch, OptimizationResult};
pub use panel::Panel;
pub use parallel::{ParallelExecutor, ParallelConfig, AssetResult, execute_parallel};
pub use reporting::{Attribution, ReportExporter};
pub use universe::{Universe, UniverseManager, DynamicUniverse, MarketCapCategory};
pub use viz::{ChartGenerator, ReportVisualizer};
pub use walk_forward::{WalkForward, WalkForwardConfig, WalkForwardResult, FoldResult};
pub use audit::{AuditLogger, AuditEvent, AuditEntry, audit_log, init_audit_logger};
pub use benchmark::BenchmarkAnalyzer;
pub use metrics::{MetricsRegistry, Timer, metrics};
pub use result_store::{ResultStore, ResultMetadata, ResultQuery, ResultId, ResultStoreRegistry, MemoryResultStore, PostgresResultStore, generate_result_id, ResultComparison, ComparisonWinner, PerformanceHistory, PerformanceTrend, load_performance_history};
pub use alerts::{Alert, AlertSeverity, AlertSink, AlertManager, AlertRule, ConsoleAlertSink, SlackAlertSink, MockAlertSink};
pub use scheduler::{Job, JobStatus, Schedule, JobScheduler, JobResult, MemoryScheduler, CronScheduler, SchedulerRegistry};
pub use data_quality::{DataValidator, DataQualityValidator, DataIssue, IssueSeverity, ValidationResult, MissingDataCheck, OutlierCheck, OutlierMethod, FreshnessCheck, DuplicateCheck};
pub use connectors_async::{AsyncPgConnector, AsyncConnectorConfig, AsyncConnectorRegistry, PoolStats};
pub use corporate_actions::{CorporateAction, ActionType, CorporateActionAdjuster, StandardAdjuster, CorporateActionStore, SymbolMapper};
pub use simd_kernels::{KernelDispatcher, KernelConfig, rolling_mean_simd, rolling_std_simd, cumsum_simd, ema_simd, batch_rolling_mean, batch_rolling_std};
pub use mmap_data::{MmapLoader, DataStream, MmapCache};
pub use incremental::{IncrementalCompute, IncrementalProcessor, RollingMeanState, RollingStdState, EmaState, CumsumState, RsiState};
pub use config::{RuntimeConfig, DatabaseConfig, ExecutionConfig, DataConfig, AlertConfig, LoggingConfig, CacheConfig, BacktestConfig, StrategyParams};
pub use factors::{FamaFrench, FactorExposures, ReturnDecomposition, BarraModel, FactorBuilder, FactorAnalysis, analyze_factors};
pub use risk_models::{VaRCalculator, CVaRCalculator, StressTest, StressScenario, RiskReport, generate_risk_report, MarginalVaR, component_var};
pub use regime::{MarketRegime, HiddenMarkovModel, VolatilityRegime, TrendRegime, RegimeDetector, KMeansRegime};
pub use portfolio_opt::{OptimalPortfolio, MeanVarianceOptimizer, PortfolioConstraints, RiskParityOptimizer, BlackLitterman, View, HierarchicalRiskParity};
pub use safety::{
    activate_kill_switch, deactivate_kill_switch, is_kill_switch_active,
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
    PositionLimits, PositionLimitEnforcer,
    Order, OrderSide, OrderType, OrderValidator, OrderValidationRules,
    RateLimiter, SafetyManager, SafetyStatus,
};
pub use integrations::{
    MarketDataProvider, Quote, YahooFinance,
    AlpacaBroker, AlpacaAccount, AlpacaPosition, AlpacaOrder, AlpacaOrderResponse,
    StreamingClient, IntegrationRegistry,
};

/// Runtime execution context
pub struct Runtime {
    #[allow(dead_code)]
    cache: Option<sig_cache::Cache>,
    engine: Engine,
    backtester: Backtester,
}

impl Runtime {
    /// Create a new runtime instance
    pub fn new() -> Self {
        Runtime {
            cache: None,
            engine: Engine::new(),
            backtester: Backtester::new(),
        }
    }

    /// Create a runtime with caching enabled
    pub fn with_cache(cache: sig_cache::Cache) -> Self {
        Runtime {
            cache: Some(cache),
            engine: Engine::new(),
            backtester: Backtester::new(),
        }
    }

    /// Execute IR and produce a backtest report
    pub fn execute(&mut self, plan: &BacktestPlan) -> Result<BacktestReport> {
        tracing::info!("Executing backtest plan");

        // Load data from sources specified in IR, or generate sample data
        let prices = if !plan.ir.metadata.data_sources.is_empty() {
            let loader = DataLoader::new();
            let source = &plan.ir.metadata.data_sources[0];
            tracing::info!("Loading data from: {}", source.path);

            // Create date range from plan
            let date_range = DateRange {
                start: if plan.start_date.is_empty() { None } else { Some(plan.start_date.clone()) },
                end: if plan.end_date.is_empty() { None } else { Some(plan.end_date.clone()) },
            };

            match loader.load_with_dates(&source.path, "date", &date_range) {
                Ok(df) => {
                    tracing::info!("Loaded {} rows x {} columns from {} (filtered by date)", df.height(), df.width(), source.path);
                    df
                }
                Err(e) => {
                    // Fall back to loading without date filter
                    tracing::warn!("Failed to load with dates: {}, trying without filter", e);
                    match loader.load(&source.path) {
                        Ok(df) => {
                            tracing::info!("Loaded {} rows x {} columns from {}", df.height(), df.width(), source.path);
                            df
                        }
                        Err(e2) => {
                            tracing::warn!("Failed to load {}: {}, using sample data", source.path, e2);
                            DataLoader::sample_prices(252, 10)?
                        }
                    }
                }
            }
        } else {
            DataLoader::sample_prices(252, 10)?
        };
        tracing::info!("Loaded {} rows x {} columns of price data", prices.height(), prices.width());

        let n_outputs = plan.ir.outputs.len();
        tracing::info!("IR has {} outputs, {} nodes", n_outputs, plan.ir.nodes.len());

        // Get all price columns (exclude date)
        let col_names: Vec<String> = prices.get_column_names()
            .iter()
            .filter(|&name| *name != "date")
            .map(|s| s.to_string())
            .collect();

        let n_assets = col_names.len();
        let n_rows = prices.height();

        // Get input name from data source
        let input_name = plan.ir.metadata.data_sources
            .first()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "prices".to_string());

        // Execute IR for each asset to get per-asset signals
        let mut asset_signals: Vec<Vec<f64>> = Vec::with_capacity(n_assets);

        for col_name in &col_names {
            let col = prices.column(col_name)
                .map_err(|e| sig_types::SigcError::Runtime(format!("Column not found: {}", e)))?;

            let values: Vec<f64> = col.f64()
                .map_err(|e| sig_types::SigcError::Runtime(format!("Cast failed: {}", e)))?
                .into_iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect();

            let mut inputs: HashMap<String, Series> = HashMap::new();
            inputs.insert(input_name.clone(), Series::new(col_name.clone().into(), values));

            // Execute IR graph for this asset
            let outputs = self.engine.execute(&plan.ir, &inputs)?;

            let signal = if !outputs.is_empty() {
                outputs[0].f64()
                    .map(|ca| ca.into_iter().map(|v| v.unwrap_or(0.0)).collect())
                    .unwrap_or_else(|_| vec![0.0; n_rows])
            } else {
                vec![0.0; n_rows]
            };

            asset_signals.push(signal);
        }

        // Convert signals to weights using cross-sectional ranking and long/short
        let mut weight_cols: Vec<Column> = Vec::with_capacity(n_assets);

        for t in 0..n_rows {
            // Get cross-section of signals at time t
            let mut xs: Vec<(usize, f64)> = asset_signals.iter()
                .enumerate()
                .map(|(i, signals)| (i, signals[t]))
                .collect();

            // Rank signals cross-sectionally
            xs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            // Create long/short weights based on rank
            let mut weights_t = vec![0.0; n_assets];
            let top_n = (n_assets as f64 * 0.2).ceil() as usize;
            let bottom_n = top_n;

            // Short bottom ranked
            for i in 0..bottom_n.min(n_assets) {
                let asset_idx = xs[i].0;
                weights_t[asset_idx] = -1.0 / bottom_n as f64;
            }

            // Long top ranked
            for i in (n_assets.saturating_sub(top_n))..n_assets {
                let asset_idx = xs[i].0;
                weights_t[asset_idx] = 1.0 / top_n as f64;
            }

            // Store weights for this time period
            if t == 0 {
                for i in 0..n_assets {
                    weight_cols.push(Column::new(col_names[i].clone().into(), vec![weights_t[i]]));
                }
            } else {
                for i in 0..n_assets {
                    let col = weight_cols[i].as_series()
                        .ok_or_else(|| sig_types::SigcError::Runtime("Series conversion failed".to_string()))?;
                    let mut values: Vec<f64> = col.f64()
                        .map_err(|e| sig_types::SigcError::Runtime(format!("Cast to f64 failed: {}", e)))?
                        .into_iter()
                        .map(|v| v.unwrap_or(0.0))
                        .collect();
                    values.push(weights_t[i]);
                    weight_cols[i] = Column::new(col_names[i].clone().into(), values);
                }
            }
        }

        let weights = DataFrame::new(weight_cols)
            .map_err(|e| sig_types::SigcError::Runtime(format!("Failed to create weights: {}", e)))?;

        // Run backtest
        let report = self.backtester.run(&weights, &prices, plan)?;

        tracing::info!(
            "Backtest complete: return={:.2}%, sharpe={:.2}, drawdown={:.2}%",
            report.metrics.total_return * 100.0,
            report.metrics.sharpe_ratio,
            report.metrics.max_drawdown * 100.0
        );

        Ok(report)
    }

    /// Execute IR without creating a full backtest plan
    pub fn run_ir(&mut self, ir: &Ir) -> Result<BacktestReport> {
        tracing::info!("Running IR with {} nodes", ir.nodes.len());

        // Create a default plan
        let plan = BacktestPlan {
            ir: ir.clone(),
            start_date: "2020-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
            universe: "default".to_string(),
            parameters: HashMap::new(),
        };

        self.execute(&plan)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
