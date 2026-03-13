//! Backtesting engine for portfolio strategies

use polars::prelude::*;
use sig_types::{BacktestMetrics, BacktestPlan, BacktestReport, Result, SigcError};

/// Backtester for evaluating portfolio strategies
pub struct Backtester {
    /// Transaction cost in basis points
    pub cost_bps: f64,
    /// Slippage model coefficient
    pub slippage_coef: f64,
}

impl Backtester {
    pub fn new() -> Self {
        Backtester {
            cost_bps: 5.0,
            slippage_coef: 0.1,
        }
    }

    /// Run a backtest given weights and prices
    pub fn run(
        &self,
        weights: &DataFrame,
        prices: &DataFrame,
        _plan: &BacktestPlan,
    ) -> Result<BacktestReport> {
        // Calculate returns from prices
        let returns = self.calculate_returns(prices)?;

        // Calculate portfolio returns
        let port_returns = self.calculate_portfolio_returns(weights, &returns)?;

        // Calculate metrics
        let metrics = self.calculate_metrics(&port_returns)?;

        Ok(BacktestReport {
            plan_hash: String::new(),
            executed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics,
        })
    }

    fn calculate_returns(&self, prices: &DataFrame) -> Result<DataFrame> {
        let mut return_cols = Vec::new();

        for col in prices.get_columns() {
            if col.name().as_str() == "date" {
                return_cols.push(col.clone());
                continue;
            }

            let shifted = col.shift(1);
            let returns = (col / &shifted)
                .map_err(|e| SigcError::Runtime(format!("Return calc failed: {}", e)))?
                - 1.0;

            return_cols.push(returns.with_name(col.name().clone()));
        }

        DataFrame::new(return_cols)
            .map_err(|e| SigcError::Runtime(format!("DataFrame creation failed: {}", e)))
    }

    fn calculate_portfolio_returns(
        &self,
        weights: &DataFrame,
        returns: &DataFrame,
    ) -> Result<Series> {
        // Simplified: equal weighted for now
        let n_assets = returns.width() - 1; // Exclude date column
        if n_assets == 0 {
            return Ok(Series::new("port_returns".into(), vec![0.0f64; returns.height()]));
        }

        let mut port_returns = vec![0.0f64; returns.height()];

        for col in returns.get_columns() {
            if col.name().as_str() == "date" {
                continue;
            }

            let col_f64 = col.f64()
                .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

            for (i, val) in col_f64.into_iter().enumerate() {
                if let Some(v) = val {
                    port_returns[i] += v / n_assets as f64;
                }
            }
        }

        // Apply transaction costs
        let turnover = 0.1; // Assumed turnover per period
        let cost_per_period = turnover * self.cost_bps / 10000.0;

        for ret in &mut port_returns {
            *ret -= cost_per_period;
        }

        Ok(Series::new("port_returns".into(), port_returns))
    }

    fn calculate_metrics(&self, returns: &Series) -> Result<BacktestMetrics> {
        let returns_f64 = returns.f64()
            .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

        // Total return (cumulative)
        let mut cum_return: f64 = 1.0;
        let mut max_cum: f64 = 1.0;
        let mut max_drawdown: f64 = 0.0;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut count = 0;

        for val in returns_f64.into_iter().flatten() {
            cum_return *= 1.0 + val;
            max_cum = max_cum.max(cum_return);
            let drawdown = 1.0 - cum_return / max_cum;
            max_drawdown = max_drawdown.max(drawdown);

            sum += val;
            sum_sq += val * val;
            count += 1;
        }

        let total_return = cum_return - 1.0;

        // Annualized metrics (assuming daily data, 252 trading days)
        let periods_per_year = 252.0;
        let years = count as f64 / periods_per_year;
        let annualized_return = if years > 0.0 {
            (1.0 + total_return).powf(1.0 / years) - 1.0
        } else {
            0.0
        };

        // Sharpe ratio
        let mean = if count > 0 { sum / count as f64 } else { 0.0 };
        let variance = if count > 1 {
            (sum_sq - sum * sum / count as f64) / (count - 1) as f64
        } else {
            0.0
        };
        let std = variance.sqrt();
        let sharpe_ratio = if std > 0.0 {
            mean / std * periods_per_year.sqrt()
        } else {
            0.0
        };

        Ok(BacktestMetrics {
            total_return,
            annualized_return,
            sharpe_ratio,
            max_drawdown,
            turnover: 0.1, // Placeholder
        })
    }
}

impl Default for Backtester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataLoader;

    #[test]
    fn test_backtest_metrics() {
        let returns = Series::new("returns".into(), vec![0.01, -0.005, 0.02, -0.01, 0.015]);
        let backtester = Backtester::new();
        let metrics = backtester.calculate_metrics(&returns).unwrap();

        assert!(metrics.total_return > 0.0);
        assert!(metrics.max_drawdown >= 0.0);
    }
}
