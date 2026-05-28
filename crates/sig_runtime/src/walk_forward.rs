//! Walk-forward optimization for robustness testing
//!
//! Implements rolling window backtests with train/test splits to detect overfitting.

use crate::Runtime;
use sig_types::{BacktestMetrics, BacktestPlan, Ir, Result, SigcError};
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

    /// Run honest walk-forward evaluation.
    ///
    /// Loads the price panel once via `runtime.load_prices_for`, then for each
    /// fold slices it into in-sample and out-of-sample row ranges and runs the
    /// IR through `runtime.execute_on_prices` on each. Both `train_metrics`
    /// and `test_metrics` are now computed from real, non-overlapping data
    /// windows — no simulated degradation.
    ///
    /// Note: this implementation does **not** sweep parameters across folds.
    /// sigc bakes parameter values into the IR at compile time, and the
    /// runtime does not yet honour `BacktestPlan::parameters`, so any per-fold
    /// "best parameters" would be vacuous. Sweeping requires recompiling the
    /// `.sig` source with different values; this is left as a follow-up. The
    /// `param_grid` field is retained for API compatibility but is unused.
    pub fn run(&self, ir: &Ir, runtime: &mut Runtime) -> Result<WalkForwardResult> {
        let num_folds = self.config.num_folds();
        if num_folds == 0 {
            return Err(SigcError::Runtime(
                "Invalid walk-forward configuration: no valid folds".into()
            ));
        }

        let plan = BacktestPlan {
            ir: ir.clone(),
            start_date: String::new(),
            end_date: String::new(),
            universe: "default".into(),
            parameters: HashMap::new(),
        };
        let prices = runtime.load_prices_for(&plan)?;
        let n_rows = prices.height();
        let need = self.config.train_periods + self.config.test_periods;
        if n_rows < need {
            return Err(SigcError::Runtime(format!(
                "walk-forward needs at least {need} rows of data, loaded {n_rows}"
            )));
        }

        let mut folds = Vec::new();
        let step = self.config.step();

        for fold_idx in 0..num_folds {
            let test_start = self.config.train_periods + fold_idx * step;
            let test_end = (test_start + self.config.test_periods).min(n_rows);
            if test_end <= test_start {
                break;
            }
            let train_start = if self.config.expanding {
                0
            } else {
                test_start - self.config.train_periods
            };
            let train_end = test_start;

            // Real in-sample evaluation on the IS slice.
            let is_prices = prices.slice(train_start as i64, train_end - train_start);
            let train_report = runtime.execute_on_prices(&plan, is_prices)?;

            // Real out-of-sample evaluation on the (disjoint) OOS slice.
            let oos_prices = prices.slice(test_start as i64, test_end - test_start);
            let test_report = runtime.execute_on_prices(&plan, oos_prices)?;

            folds.push(FoldResult {
                fold: fold_idx,
                train_start,
                train_end,
                test_start,
                test_end,
                best_params: HashMap::new(),
                train_metrics: train_report.metrics,
                test_metrics: test_report.metrics,
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
            sortino_ratio: folds.iter().map(|f| f.test_metrics.sortino_ratio).sum::<f64>() / folds.len() as f64,
            calmar_ratio: folds.iter().map(|f| f.test_metrics.calmar_ratio).sum::<f64>() / folds.len() as f64,
            win_rate: folds.iter().map(|f| f.test_metrics.win_rate).sum::<f64>() / folds.len() as f64,
            profit_factor: folds.iter().map(|f| f.test_metrics.profit_factor).sum::<f64>() / folds.len() as f64,
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
    fn run_evaluates_real_is_oos_slices() {
        // Smoke test: with the honest implementation, IS and OOS metrics come
        // from disjoint data windows, so they should not be byte-identical the
        // way the old simulated test_metrics = train_metrics * 0.7 path made
        // them be.
        use sig_compiler::Compiler;
        let src = "data:\n  prices: load parquet from \"prices.parquet\"\n\
                   \n\
                   signal s:\n  emit zscore(ret(prices, periods=5))\n";
        let ir = Compiler::new().compile(src).expect("compile");
        let wf = WalkForward::new(WalkForwardConfig::new(252, 126, 21));
        let mut rt = Runtime::new();
        let result = wf.run(&ir, &mut rt).expect("walk-forward run");
        assert!(!result.folds.is_empty(), "expected at least one fold");
        let identical = result.folds.iter()
            .filter(|f| f.train_metrics.sharpe_ratio.to_bits()
                     == f.test_metrics.sharpe_ratio.to_bits())
            .count();
        assert!(
            identical < result.folds.len(),
            "IS == OOS for every fold ({}/{}); simulation regression?",
            identical, result.folds.len()
        );
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
                sortino_ratio: 1.0,
                calmar_ratio: 1.0,
                win_rate: 0.5,
                profit_factor: 1.0,
            },
        };
        assert!(result.is_overfit());
    }
}
