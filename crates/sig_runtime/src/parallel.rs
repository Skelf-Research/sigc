//! Parallel execution for large universes
//!
//! Uses rayon for parallel IR evaluation across multiple assets.

use polars::prelude::*;
use rayon::prelude::*;
use sig_types::{Ir, Result, SigcError};
use std::collections::HashMap;
use std::sync::Arc;

use crate::Engine;

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of threads (0 = auto-detect)
    pub num_threads: usize,
    /// Chunk size for batching assets
    pub chunk_size: usize,
    /// Whether to use parallel execution
    pub enabled: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            num_threads: 0, // Auto-detect
            chunk_size: 100,
            enabled: true,
        }
    }
}

impl ParallelConfig {
    /// Create config with specific thread count
    pub fn with_threads(mut self, n: usize) -> Self {
        self.num_threads = n;
        self
    }

    /// Set chunk size for batching
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Disable parallel execution
    pub fn disabled() -> Self {
        ParallelConfig {
            enabled: false,
            ..Default::default()
        }
    }
}

/// Result from parallel execution for a single asset
#[derive(Debug, Clone)]
pub struct AssetResult {
    /// Asset name
    pub asset: String,
    /// Computed signal values
    pub signal: Vec<f64>,
}

/// Parallel executor for IR evaluation
pub struct ParallelExecutor {
    config: ParallelConfig,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: ParallelConfig) -> Self {
        ParallelExecutor { config }
    }

    /// Execute IR across multiple assets in parallel
    pub fn execute_parallel(
        &self,
        ir: &Ir,
        prices: &DataFrame,
        input_name: &str,
    ) -> Result<Vec<AssetResult>> {
        // Get all price columns (exclude date)
        let col_names: Vec<String> = prices
            .get_column_names()
            .iter()
            .filter(|&name| *name != "date")
            .map(|s| s.to_string())
            .collect();

        if col_names.is_empty() {
            return Ok(Vec::new());
        }

        let n_assets = col_names.len();
        let n_rows = prices.height();

        // Prepare data for parallel processing
        let asset_data: Vec<(String, Vec<f64>)> = col_names
            .iter()
            .map(|col_name| {
                let col = prices.column(col_name).ok();
                let values: Vec<f64> = col
                    .and_then(|c| c.f64().ok())
                    .map(|ca| ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect())
                    .unwrap_or_else(|| vec![f64::NAN; n_rows]);
                (col_name.clone(), values)
            })
            .collect();

        // Share IR across threads
        let ir = Arc::new(ir.clone());
        let input_name = Arc::new(input_name.to_string());

        // Execute in parallel or sequentially based on config
        let results: Vec<AssetResult> = if self.config.enabled && n_assets > 1 {
            // Configure thread pool if specified
            if self.config.num_threads > 0 {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(self.config.num_threads)
                    .build_global()
                    .ok(); // Ignore if already configured
            }

            asset_data
                .par_iter()
                .map(|(asset_name, values)| {
                    self.execute_single(&ir, &input_name, asset_name, values, n_rows)
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            // Sequential execution
            asset_data
                .iter()
                .map(|(asset_name, values)| {
                    self.execute_single(&ir, &input_name, asset_name, values, n_rows)
                })
                .collect::<Result<Vec<_>>>()?
        };

        Ok(results)
    }

    /// Execute IR for a single asset
    fn execute_single(
        &self,
        ir: &Ir,
        input_name: &str,
        asset_name: &str,
        values: &[f64],
        n_rows: usize,
    ) -> Result<AssetResult> {
        let mut engine = Engine::new();
        let mut inputs: HashMap<String, Series> = HashMap::new();
        inputs.insert(
            input_name.to_string(),
            Series::new(asset_name.into(), values.to_vec()),
        );

        let outputs = engine.execute(ir, &inputs)?;

        let signal = if !outputs.is_empty() {
            outputs[0]
                .f64()
                .map(|ca| ca.into_iter().map(|v| v.unwrap_or(0.0)).collect())
                .unwrap_or_else(|_| vec![0.0; n_rows])
        } else {
            vec![0.0; n_rows]
        };

        Ok(AssetResult {
            asset: asset_name.to_string(),
            signal,
        })
    }

    /// Execute with batching for very large universes
    pub fn execute_batched(
        &self,
        ir: &Ir,
        prices: &DataFrame,
        input_name: &str,
    ) -> Result<Vec<AssetResult>> {
        let col_names: Vec<String> = prices
            .get_column_names()
            .iter()
            .filter(|&name| *name != "date")
            .map(|s| s.to_string())
            .collect();

        if col_names.len() <= self.config.chunk_size {
            return self.execute_parallel(ir, prices, input_name);
        }

        // Process in batches to manage memory
        let mut all_results = Vec::with_capacity(col_names.len());

        for chunk in col_names.chunks(self.config.chunk_size) {
            // Create subset DataFrame for this chunk
            let mut columns = Vec::with_capacity(chunk.len() + 1);

            // Include date if present
            if let Ok(date_col) = prices.column("date") {
                columns.push(date_col.clone());
            }

            for col_name in chunk {
                if let Ok(col) = prices.column(col_name) {
                    columns.push(col.clone());
                }
            }

            let chunk_df = DataFrame::new(columns)
                .map_err(|e| SigcError::Runtime(format!("Chunk creation failed: {}", e)))?;

            let chunk_results = self.execute_parallel(ir, &chunk_df, input_name)?;
            all_results.extend(chunk_results);
        }

        Ok(all_results)
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new(ParallelConfig::default())
    }
}

/// Helper function for quick parallel execution
pub fn execute_parallel(
    ir: &Ir,
    prices: &DataFrame,
    input_name: &str,
) -> Result<Vec<AssetResult>> {
    let executor = ParallelExecutor::default();
    executor.execute_parallel(ir, prices, input_name)
}

/// Parallel fold operation for aggregating results
pub fn parallel_fold<T, F, R>(items: &[T], init: R, fold_fn: F) -> R
where
    T: Sync,
    F: Fn(R, &T) -> R + Sync + Send,
    R: Clone + Send + Sync,
{
    items.par_iter().fold(
        || init.clone(),
        |acc, item| fold_fn(acc, item),
    ).reduce(
        || init.clone(),
        |a, _b| a, // Simple reduction - caller should combine
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prices(n_assets: usize, n_rows: usize) -> DataFrame {
        let mut columns = Vec::with_capacity(n_assets + 1);

        // Date column
        let dates: Vec<String> = (0..n_rows).map(|i| format!("2020-01-{:02}", i % 28 + 1)).collect();
        columns.push(Column::new("date".into(), dates));

        // Asset columns
        for i in 0..n_assets {
            let values: Vec<f64> = (0..n_rows).map(|j| 100.0 + (i + j) as f64 * 0.1).collect();
            columns.push(Column::new(format!("asset_{}", i).into(), values));
        }

        DataFrame::new(columns).unwrap()
    }

    #[test]
    fn test_empty_prices() {
        use sig_types::*;
        let ir = Ir {
            nodes: vec![],
            outputs: vec![],
            metadata: IrMetadata {
                source_hash: "test".to_string(),
                compiled_at: 0,
                compiler_version: "0.1".to_string(),
                parameters: vec![],
                data_sources: vec![],
            },
        };

        let dates: Vec<String> = vec!["2020-01-01".to_string()];
        let prices = DataFrame::new(vec![Column::new("date".into(), dates)]).unwrap();

        let results = execute_parallel(&ir, &prices, "prices").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_config_threads() {
        let config = ParallelConfig::default()
            .with_threads(4)
            .with_chunk_size(50);

        assert_eq!(config.num_threads, 4);
        assert_eq!(config.chunk_size, 50);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_disabled() {
        let config = ParallelConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_asset_result_creation() {
        let result = AssetResult {
            asset: "AAPL".to_string(),
            signal: vec![1.0, 2.0, 3.0],
        };
        assert_eq!(result.asset, "AAPL");
        assert_eq!(result.signal.len(), 3);
    }

    #[test]
    fn test_executor_default() {
        let executor = ParallelExecutor::default();
        assert!(executor.config.enabled);
    }

    #[test]
    fn test_sample_prices_structure() {
        let prices = sample_prices(3, 10);
        assert_eq!(prices.width(), 4); // date + 3 assets
        assert_eq!(prices.height(), 10);
    }
}
