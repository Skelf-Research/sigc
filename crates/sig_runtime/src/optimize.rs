//! Parameter optimization via grid search

use crate::Runtime;
use sig_types::{BacktestMetrics, BacktestPlan, BacktestReport, Ir, Result};
use std::collections::HashMap;

/// Result from a single parameter combination
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub parameters: HashMap<String, f64>,
    pub metrics: BacktestMetrics,
}

/// Grid search optimizer
pub struct GridSearch {
    /// Parameter name -> list of values to try
    param_grid: HashMap<String, Vec<f64>>,
}

impl GridSearch {
    pub fn new() -> Self {
        GridSearch {
            param_grid: HashMap::new(),
        }
    }

    /// Add a parameter with specific values to test
    pub fn add_param(&mut self, name: &str, values: Vec<f64>) -> &mut Self {
        self.param_grid.insert(name.to_string(), values);
        self
    }

    /// Add a parameter with a range
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

    /// Generate all combinations of parameters
    fn generate_combinations(&self) -> Vec<HashMap<String, f64>> {
        let param_names: Vec<&String> = self.param_grid.keys().collect();
        let param_values: Vec<&Vec<f64>> = param_names.iter().map(|k| &self.param_grid[*k]).collect();

        if param_names.is_empty() {
            return vec![HashMap::new()];
        }

        let mut combinations = Vec::new();
        let mut indices = vec![0; param_names.len()];

        loop {
            // Create current combination
            let mut combo = HashMap::new();
            for (i, name) in param_names.iter().enumerate() {
                combo.insert((*name).clone(), param_values[i][indices[i]]);
            }
            combinations.push(combo);

            // Increment indices
            let mut carry = true;
            for i in 0..indices.len() {
                if carry {
                    indices[i] += 1;
                    if indices[i] >= param_values[i].len() {
                        indices[i] = 0;
                    } else {
                        carry = false;
                    }
                }
            }

            if carry {
                break; // All combinations generated
            }
        }

        combinations
    }

    /// Run grid search optimization
    pub fn optimize(
        &self,
        ir: &Ir,
        runtime: &mut Runtime,
        metric: &str,
    ) -> Result<Vec<OptimizationResult>> {
        let combinations = self.generate_combinations();
        let n_combos = combinations.len();
        tracing::info!("Running grid search with {} parameter combinations", n_combos);

        let mut results = Vec::new();

        for (i, params) in combinations.into_iter().enumerate() {
            tracing::debug!("Testing combination {}/{}: {:?}", i + 1, n_combos, params);

            // Create plan with these parameters
            let plan = BacktestPlan {
                ir: ir.clone(),
                start_date: "2020-01-01".to_string(),
                end_date: "2024-12-31".to_string(),
                universe: "default".to_string(),
                parameters: params.clone(),
            };

            // Run backtest
            match runtime.execute(&plan) {
                Ok(report) => {
                    results.push(OptimizationResult {
                        parameters: params,
                        metrics: report.metrics,
                    });
                }
                Err(e) => {
                    tracing::warn!("Combination {:?} failed: {}", params, e);
                }
            }
        }

        // Sort results by the requested metric
        results.sort_by(|a, b| {
            let va = get_metric_value(&a.metrics, metric);
            let vb = get_metric_value(&b.metrics, metric);
            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
        });

        tracing::info!(
            "Grid search complete. Best {}: {:.4} with params {:?}",
            metric,
            results.first().map(|r| get_metric_value(&r.metrics, metric)).unwrap_or(0.0),
            results.first().map(|r| &r.parameters)
        );

        Ok(results)
    }
}

impl Default for GridSearch {
    fn default() -> Self {
        Self::new()
    }
}

fn get_metric_value(metrics: &BacktestMetrics, name: &str) -> f64 {
    match name {
        "sharpe" | "sharpe_ratio" => metrics.sharpe_ratio,
        "return" | "total_return" => metrics.total_return,
        "drawdown" | "max_drawdown" => -metrics.max_drawdown, // Negate so higher is better
        "turnover" => -metrics.turnover, // Negate so lower is better
        _ => metrics.sharpe_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_combinations() {
        let mut gs = GridSearch::new();
        gs.add_param("lookback", vec![20.0, 60.0]);
        gs.add_param("hold", vec![5.0, 10.0]);

        let combos = gs.generate_combinations();
        assert_eq!(combos.len(), 4);
    }

    #[test]
    fn test_add_range() {
        let mut gs = GridSearch::new();
        gs.add_range("x", 1.0, 3.0, 1.0);

        let values = &gs.param_grid["x"];
        assert_eq!(values, &vec![1.0, 2.0, 3.0]);
    }
}
