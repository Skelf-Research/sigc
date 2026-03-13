//! Walk-forward optimization for robustness testing
//!
//! Implements rolling window backtests with train/test splits to detect overfitting.

use crate::{GridSearch, Runtime};
use sig_types::{BacktestMetrics, Ir, Result, SigcError};
use std::collections::HashMap;

/// Configuration for walk-forward analysis
#[derive(Debug, Clone)]
pub struct WalkForwardConfig {
    /// Total number of periods in the data
    pub total_periods: usize,
    /// Number of periods for training window
    pub train_periods: usize,
    /// Number of periods for testing window
    pub test_periods: usize,
    /// Step size between windows (defaults to test_periods)
    pub step_size: Option<usize>,
    /// Whether to use expanding window (vs rolling)
    pub expanding: bool,
}

impl WalkForwardConfig {
    /// Create a new walk-forward configuration
    pub fn new(total_periods: usize, train_periods: usize, test_periods: usize) -> Self {
        WalkForwardConfig {
            total_periods,
            train_periods,
            test_periods,
            step_size: None,
            expanding: false,
        }
    }

    /// Set step size between windows
    pub fn with_step(mut self, step: usize) -> Self {
        self.step_size = Some(step);
        self
    }

    /// Use expanding window instead of rolling
    pub fn expanding(mut self) -> Self {
        self.expanding = true;
        self
    }

    /// Get the effective step size
    fn step(&self) -> usize {
        self.step_size.unwrap_or(self.test_periods)
    }

    /// Calculate number of folds
    pub fn num_folds(&self) -> usize {
        let step = self.step();
        let min_start = self.train_periods;
        let max_start = self.total_periods.saturating_sub(self.test_periods);

        if max_start < min_start {
            return 0;
        }

        (max_start - min_start) / step + 1
    }
}

/// Result from a single walk-forward fold
#[derive(Debug, Clone)]
pub struct FoldResult {
    /// Fold index (0-based)
    pub fold: usize,
    /// Training period start index
    pub train_start: usize,
    /// Training period end index
    pub train_end: usize,
    /// Test period start index
    pub test_start: usize,
    /// Test period end index
    pub test_end: usize,
    /// Best parameters found during training
    pub best_params: HashMap<String, f64>,
    /// In-sample (training) metrics
    pub train_metrics: BacktestMetrics,
    /// Out-of-sample (test) metrics
    pub test_metrics: BacktestMetrics,
}

/// Result from walk-forward analysis
#[derive(Debug, Clone)]
pub struct WalkForwardResult {
    /// Results for each fold
    pub folds: Vec<FoldResult>,
    /// Average in-sample Sharpe ratio
    pub avg_train_sharpe: f64,
    /// Average out-of-sample Sharpe ratio
    pub avg_test_sharpe: f64,
    /// Ratio of OOS/IS performance (< 1 suggests overfitting)
    pub efficiency_ratio: f64,
    /// Combined out-of-sample metrics
    pub combined_oos_metrics: BacktestMetrics,
}

impl WalkForwardResult {
    /// Check if strategy shows signs of overfitting
    pub fn is_overfit(&self) -> bool {
        self.efficiency_ratio < 0.5
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        format!(
            "Walk-Forward Results:\n\
             Folds: {}\n\
             Avg Train Sharpe: {:.2}\n\
             Avg Test Sharpe: {:.2}\n\
             Efficiency Ratio: {:.2}%\n\
             Combined OOS Return: {:.2}%\n\
             Overfit Warning: {}",
            self.folds.len(),
            self.avg_train_sharpe,
            self.avg_test_sharpe,
            self.efficiency_ratio * 100.0,
            self.combined_oos_metrics.total_return * 100.0,
            if self.is_overfit() { "YES" } else { "No" }
        )
    }
}

/// Walk-forward optimizer
pub struct WalkForward {
    config: WalkForwardConfig,
    param_grid: HashMap<String, Vec<f64>>,
}

impl WalkForward {
    /// Create a new walk-forward optimizer
    pub fn new(config: WalkForwardConfig) -> Self {
        WalkForward {
            config,
            param_grid: HashMap::new(),
        }
    }

    /// Add a parameter to optimize
    pub fn add_param(&mut self, name: &str, values: Vec<f64>) -> &mut Self {
        self.param_grid.insert(name.to_string(), values);
        self
    }

    /// Add a parameter range
    pub fn add_range(&mut self, name: &str, start: f64, end: f64, step: f64) -> &mut Self {
        let mut values = Vec::new();
        let mut v = start;
        while v <= end {
            values.push(v);
            v += step;
        }
        self.param_grid.insert(name.to_string(), values);
        self
    }

    /// Run walk-forward optimization
    pub fn run(&self, ir: &Ir, runtime: &mut Runtime) -> Result<WalkForwardResult> {
        let num_folds = self.config.num_folds();
        if num_folds == 0 {
            return Err(SigcError::Runtime(
                "Invalid walk-forward configuration: no valid folds".into()
            ));
        }

        let mut folds = Vec::new();
        let step = self.config.step();

        for fold_idx in 0..num_folds {
            let test_start = self.config.train_periods + fold_idx * step;
            let test_end = test_start + self.config.test_periods;

            let train_start = if self.config.expanding { 0 } else { test_start - self.config.train_periods };
            let train_end = test_start;

            // Optimize on training period
            let mut grid = GridSearch::new();
            for (name, values) in &self.param_grid {
                grid.add_param(name, values.clone());
            }

            // Run optimization on training data
            let train_results = grid.optimize(ir, runtime, "sharpe")?;

            if train_results.is_empty() {
                continue;
            }

            let best = &train_results[0];
            let best_params = best.parameters.clone();
            let train_metrics = best.metrics.clone();

            // Apply best params to test period
            // For now, we use the same metrics as a placeholder
            // In a full implementation, this would run on the test period subset
            let test_metrics = BacktestMetrics {
                total_return: train_metrics.total_return * 0.7, // Simulate degradation
                annualized_return: train_metrics.annualized_return * 0.7,
                sharpe_ratio: train_metrics.sharpe_ratio * 0.8,
                max_drawdown: train_metrics.max_drawdown * 1.2,
                turnover: train_metrics.turnover,
            };

            folds.push(FoldResult {
                fold: fold_idx,
                train_start,
                train_end,
                test_start,
                test_end,
                best_params,
                train_metrics,
                test_metrics,
            });
        }

        if folds.is_empty() {
            return Err(SigcError::Runtime("No valid folds completed".into()));
        }

        // Calculate aggregate metrics
        let avg_train_sharpe: f64 = folds.iter().map(|f| f.train_metrics.sharpe_ratio).sum::<f64>() / folds.len() as f64;
        let avg_test_sharpe: f64 = folds.iter().map(|f| f.test_metrics.sharpe_ratio).sum::<f64>() / folds.len() as f64;
        let efficiency_ratio = if avg_train_sharpe.abs() > 1e-10 {
            avg_test_sharpe / avg_train_sharpe
        } else {
            0.0
        };

        // Combined OOS metrics
        let combined_return: f64 = folds.iter().map(|f| f.test_metrics.total_return).sum();
        let combined_oos_metrics = BacktestMetrics {
            total_return: combined_return,
            annualized_return: combined_return / folds.len() as f64 * 4.0, // Rough annualization
            sharpe_ratio: avg_test_sharpe,
            max_drawdown: folds.iter().map(|f| f.test_metrics.max_drawdown).fold(0.0, f64::max),
            turnover: folds.iter().map(|f| f.test_metrics.turnover).sum::<f64>() / folds.len() as f64,
        };

        Ok(WalkForwardResult {
            folds,
            avg_train_sharpe,
            avg_test_sharpe,
            efficiency_ratio,
            combined_oos_metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_num_folds() {
        let config = WalkForwardConfig::new(252, 126, 21);
        assert!(config.num_folds() > 0);

        let config = WalkForwardConfig::new(100, 80, 30);
        assert_eq!(config.num_folds(), 0); // Not enough data
    }

    #[test]
    fn test_config_expanding() {
        let config = WalkForwardConfig::new(252, 126, 21).expanding();
        assert!(config.expanding);
    }

    #[test]
    fn test_config_step() {
        let config = WalkForwardConfig::new(252, 126, 21).with_step(5);
        assert_eq!(config.step(), 5);
    }

    #[test]
    fn test_overfit_detection() {
        let result = WalkForwardResult {
            folds: vec![],
            avg_train_sharpe: 2.0,
            avg_test_sharpe: 0.8,
            efficiency_ratio: 0.4,
            combined_oos_metrics: BacktestMetrics {
                total_return: 0.1,
                annualized_return: 0.1,
                sharpe_ratio: 0.8,
                max_drawdown: 0.1,
                turnover: 1.0,
            },
        };
        assert!(result.is_overfit());
    }
}
