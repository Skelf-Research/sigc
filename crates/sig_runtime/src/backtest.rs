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

        // Calculate portfolio returns and turnover
        let (port_returns, turnover) = self.calculate_portfolio_returns(weights, &returns)?;

        // Extract returns series as Vec
        let returns_series: Vec<f64> = port_returns.f64()
            .map(|ca| ca.into_iter().map(|v| v.unwrap_or(0.0)).collect())
            .unwrap_or_default();

        // Calculate metrics
        let metrics = self.calculate_metrics(&port_returns, turnover)?;

        Ok(BacktestReport {
            plan_hash: String::new(),
            executed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            metrics,
            positions: None,  // TODO: Populate from weights
            returns_series,
            benchmark_metrics: None,
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
    ) -> Result<(Series, f64)> {
        let n_rows = returns.height();
        if n_rows == 0 {
            return Ok((Series::new("port_returns".into(), vec![0.0f64]), 0.0));
        }

        let mut port_returns = vec![0.0f64; n_rows];
        let mut total_turnover = 0.0;
        let mut prev_weights: Option<Vec<f64>> = None;

        // Get asset names from weights (excluding date)
        let weight_cols: Vec<String> = weights.get_column_names()
            .iter()
            .filter(|&name| *name != "date")
            .map(|s| s.to_string())
            .collect();

        for t in 0..n_rows {
            let mut period_return = 0.0;
            let mut current_weights = Vec::new();

            for col_name in &weight_cols {
                // Get weight for this asset at time t
                let weight = if let Ok(w_col) = weights.column(col_name) {
                    w_col.f64()
                        .ok()
                        .and_then(|ca| ca.get(t))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                current_weights.push(weight);

                // Get return for this asset at time t
                let ret = if let Ok(r_col) = returns.column(col_name) {
                    r_col.f64()
                        .ok()
                        .and_then(|ca| ca.get(t))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                // Weighted return
                period_return += weight * ret;
            }

            // Calculate turnover (sum of absolute weight changes)
            if let Some(ref prev) = prev_weights {
                let turnover: f64 = current_weights.iter()
                    .zip(prev.iter())
                    .map(|(curr, prev)| (curr - prev).abs())
                    .sum();
                total_turnover += turnover;

                // Apply transaction costs based on turnover
                let cost = turnover * self.cost_bps / 10000.0;
                period_return -= cost;
            }

            port_returns[t] = period_return;
            prev_weights = Some(current_weights);
        }

        // Annualize turnover (assuming daily data)
        let annualized_turnover = if n_rows > 1 {
            total_turnover / n_rows as f64 * 252.0
        } else {
            0.0
        };

        Ok((Series::new("port_returns".into(), port_returns), annualized_turnover))
    }

    fn calculate_metrics(&self, returns: &Series, turnover: f64) -> Result<BacktestMetrics> {
        let returns_f64 = returns.f64()
            .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

        // Collect returns into vec for multiple passes
        let returns_vec: Vec<f64> = returns_f64.into_iter()
            .map(|v| v.unwrap_or(0.0))
            .collect();

        // Total return (cumulative)
        let mut cum_return: f64 = 1.0;
        let mut max_cum: f64 = 1.0;
        let mut max_drawdown: f64 = 0.0;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut downside_sum_sq = 0.0;
        let mut wins = 0;
        let mut losses = 0;
        let mut win_total = 0.0;
        let mut loss_total = 0.0;

        for &val in &returns_vec {
            cum_return *= 1.0 + val;
            max_cum = max_cum.max(cum_return);
            let drawdown = 1.0 - cum_return / max_cum;
            max_drawdown = max_drawdown.max(drawdown);

            sum += val;
            sum_sq += val * val;

            // Downside deviation (for Sortino)
            if val < 0.0 {
                downside_sum_sq += val * val;
                losses += 1;
                loss_total += val.abs();
            } else if val > 0.0 {
                wins += 1;
                win_total += val;
            }
        }

        let count = returns_vec.len();
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

        // Sortino ratio (downside deviation)
        let downside_std = if count > 0 {
            (downside_sum_sq / count as f64).sqrt()
        } else {
            0.0
        };
        let sortino_ratio = if downside_std > 0.0 {
            mean / downside_std * periods_per_year.sqrt()
        } else {
            0.0
        };

        // Calmar ratio (return / max drawdown)
        let calmar_ratio = if max_drawdown > 0.0 {
            annualized_return / max_drawdown
        } else {
            0.0
        };

        // Win rate
        let win_rate = if count > 0 {
            wins as f64 / count as f64
        } else {
            0.0
        };

        // Profit factor (avg win / avg loss)
        let avg_win = if wins > 0 { win_total / wins as f64 } else { 0.0 };
        let avg_loss = if losses > 0 { loss_total / losses as f64 } else { 0.0 };
        let profit_factor = if avg_loss > 0.0 {
            avg_win / avg_loss
        } else {
            0.0
        };

        Ok(BacktestMetrics {
            total_return,
            annualized_return,
            sharpe_ratio,
            max_drawdown,
            turnover,
            sortino_ratio,
            calmar_ratio,
            win_rate,
            profit_factor,
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

    #[test]
    fn test_backtest_metrics() {
        let returns = Series::new("returns".into(), vec![0.01, -0.005, 0.02, -0.01, 0.015]);
        let backtester = Backtester::new();
        let metrics = backtester.calculate_metrics(&returns, 2.5).unwrap();

        assert!(metrics.total_return > 0.0);
        assert!(metrics.max_drawdown >= 0.0);
        assert_eq!(metrics.turnover, 2.5);
    }
}
