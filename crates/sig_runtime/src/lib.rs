//! Runtime execution engine for sigc
//!
//! Executes IR and manages data ingestion.

pub mod backtest;
pub mod connectors;
pub mod costs;
pub mod data;
pub mod engine;
pub mod kernels;
pub mod optimize;
pub mod panel;
pub mod reporting;
pub mod universe;
pub mod viz;
pub mod walk_forward;

use polars::prelude::*;
use sig_types::{BacktestPlan, BacktestReport, Ir, Result};
use std::collections::HashMap;

pub use backtest::Backtester;
pub use connectors::{Connector, SqlConnector, CloudConnector, ConnectorRegistry, ConnectorEnv};
pub use costs::{CostModel, ImpactModel, TradeCost, PortfolioCostCalculator, PortfolioCost};
pub use data::{DataFormat, DataLoader, DataManager, DataSource, DateRange};
pub use engine::Engine;
pub use optimize::{GridSearch, OptimizationResult};
pub use panel::Panel;
pub use reporting::{Attribution, ReportExporter};
pub use universe::{Universe, UniverseManager, DynamicUniverse, MarketCapCategory};
pub use viz::{ChartGenerator, ReportVisualizer};
pub use walk_forward::{WalkForward, WalkForwardConfig, WalkForwardResult, FoldResult};

/// Runtime execution context
pub struct Runtime {
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

            match loader.load(&source.path) {
                Ok(df) => {
                    tracing::info!("Loaded {} rows x {} columns from {}", df.height(), df.width(), source.path);
                    df
                }
                Err(e) => {
                    tracing::warn!("Failed to load {}: {}, using sample data", source.path, e);
                    DataLoader::sample_prices(252, 10)?
                }
            }
        } else {
            DataLoader::sample_prices(252, 10)?
        };
        tracing::info!("Loaded {} rows x {} columns of price data", prices.height(), prices.width());

        let n_outputs = plan.ir.outputs.len();
        tracing::info!("IR has {} outputs, {} nodes", n_outputs, plan.ir.nodes.len());

        // Execute IR graph to produce weights
        let n_assets = prices.width() - 1; // Exclude date column
        let n_rows = prices.height();

        // Convert price columns to input series for engine
        let mut inputs: HashMap<String, polars::prelude::Series> = HashMap::new();

        // Get all price columns (exclude date)
        let col_names: Vec<String> = prices.get_column_names()
            .iter()
            .filter(|&name| *name != "date")
            .map(|s| s.to_string())
            .collect();

        // Create a combined price series (average across assets for simplicity)
        // In a full implementation, this would handle multi-asset data properly
        let mut combined_prices = vec![0.0f64; n_rows];
        for col_name in &col_names {
            if let Ok(col) = prices.column(col_name) {
                if let Ok(values) = col.f64() {
                    for (i, val) in values.into_iter().enumerate() {
                        combined_prices[i] += val.unwrap_or(0.0) / col_names.len() as f64;
                    }
                }
            }
        }

        // Register input data - use first data source name or "prices"
        let input_name = plan.ir.metadata.data_sources
            .first()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "prices".to_string());

        inputs.insert(input_name, polars::prelude::Series::new("prices".into(), combined_prices));

        // Execute IR graph
        let outputs = self.engine.execute(&plan.ir, &inputs)?;

        // Convert output series to weight DataFrame
        let weights = if !outputs.is_empty() {
            let signal = &outputs[0];
            let signal_values: Vec<f64> = signal.f64()
                .map(|ca| ca.into_iter().map(|v| v.unwrap_or(0.0)).collect())
                .unwrap_or_else(|_| vec![0.0; n_rows]);

            // Distribute signal across assets (equal allocation per asset)
            let mut weight_cols: Vec<Column> = vec![];
            for i in 0..n_assets {
                let weights_vec: Vec<f64> = signal_values.iter()
                    .map(|&s| s / n_assets as f64)
                    .collect();
                weight_cols.push(Column::new(format!("w_{}", i).into(), weights_vec));
            }

            DataFrame::new(weight_cols).unwrap_or_else(|_| DataFrame::empty())
        } else {
            // Fallback to equal weights
            tracing::warn!("No IR outputs, using equal weights");
            let weight = 1.0 / n_assets as f64;
            let mut weight_cols: Vec<Column> = vec![];
            for i in 0..n_assets {
                let weights_vec = vec![weight; n_rows];
                weight_cols.push(Column::new(format!("w_{}", i).into(), weights_vec));
            }
            DataFrame::new(weight_cols).unwrap_or_else(|_| DataFrame::empty())
        };

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
