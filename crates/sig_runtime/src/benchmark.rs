//! Benchmark-relative metrics calculation
//!
//! Calculates alpha, beta, information ratio, and other relative performance metrics.

use sig_types::{BenchmarkMetrics, Result, SigcError};

/// Calculator for benchmark-relative performance metrics
pub struct BenchmarkAnalyzer;

impl BenchmarkAnalyzer {
    /// Calculate all benchmark-relative metrics
    pub fn analyze(
        portfolio_returns: &[f64],
        benchmark_returns: &[f64],
    ) -> Result<BenchmarkMetrics> {
        if portfolio_returns.len() != benchmark_returns.len() {
            return Err(SigcError::Runtime(
                "Portfolio and benchmark return series must have same length".into(),
            ));
        }

        if portfolio_returns.is_empty() {
            return Ok(BenchmarkMetrics::default());
        }

        let n = portfolio_returns.len() as f64;
        let periods_per_year = 252.0;

        // Calculate means
        let port_mean: f64 = portfolio_returns.iter().sum::<f64>() / n;
        let bench_mean: f64 = benchmark_returns.iter().sum::<f64>() / n;

        // Calculate variances and covariance
        let mut port_var = 0.0;
        let mut bench_var = 0.0;
        let mut covar = 0.0;

        for i in 0..portfolio_returns.len() {
            let port_dev = portfolio_returns[i] - port_mean;
            let bench_dev = benchmark_returns[i] - bench_mean;
            port_var += port_dev * port_dev;
            bench_var += bench_dev * bench_dev;
            covar += port_dev * bench_dev;
        }

        port_var /= n - 1.0;
        bench_var /= n - 1.0;
        covar /= n - 1.0;

        let port_std = port_var.sqrt();
        let bench_std = bench_var.sqrt();

        // Beta = Cov(portfolio, benchmark) / Var(benchmark)
        let beta = if bench_var > 1e-10 {
            covar / bench_var
        } else {
            1.0
        };

        // Alpha (Jensen's) = Portfolio return - Beta * Benchmark return
        // Annualized
        let alpha = (port_mean - beta * bench_mean) * periods_per_year;

        // Correlation
        let correlation = if port_std > 1e-10 && bench_std > 1e-10 {
            covar / (port_std * bench_std)
        } else {
            0.0
        };

        // Active returns (tracking difference)
        let active_returns: Vec<f64> = portfolio_returns
            .iter()
            .zip(benchmark_returns.iter())
            .map(|(p, b)| p - b)
            .collect();

        let active_mean: f64 = active_returns.iter().sum::<f64>() / n;
        let active_var: f64 = active_returns
            .iter()
            .map(|r| (r - active_mean).powi(2))
            .sum::<f64>()
            / (n - 1.0);
        let tracking_error = active_var.sqrt() * periods_per_year.sqrt();

        // Information ratio = Active return / Tracking error
        let information_ratio = if tracking_error > 1e-10 {
            active_mean * periods_per_year / tracking_error
        } else {
            0.0
        };

        // Capture ratios
        let (up_capture, down_capture) = Self::calculate_capture_ratios(
            portfolio_returns,
            benchmark_returns,
        );

        Ok(BenchmarkMetrics {
            alpha,
            beta,
            information_ratio,
            tracking_error,
            correlation,
            up_capture,
            down_capture,
        })
    }

    /// Calculate up and down capture ratios
    fn calculate_capture_ratios(
        portfolio_returns: &[f64],
        benchmark_returns: &[f64],
    ) -> (f64, f64) {
        let mut up_port_sum = 0.0;
        let mut up_bench_sum = 0.0;
        let mut down_port_sum = 0.0;
        let mut down_bench_sum = 0.0;
        let mut up_count = 0;
        let mut down_count = 0;

        for i in 0..benchmark_returns.len() {
            if benchmark_returns[i] > 0.0 {
                up_port_sum += portfolio_returns[i];
                up_bench_sum += benchmark_returns[i];
                up_count += 1;
            } else if benchmark_returns[i] < 0.0 {
                down_port_sum += portfolio_returns[i];
                down_bench_sum += benchmark_returns[i];
                down_count += 1;
            }
        }

        let up_capture = if up_count > 0 && up_bench_sum.abs() > 1e-10 {
            (up_port_sum / up_count as f64) / (up_bench_sum / up_count as f64)
        } else {
            1.0
        };

        let down_capture = if down_count > 0 && down_bench_sum.abs() > 1e-10 {
            (down_port_sum / down_count as f64) / (down_bench_sum / down_count as f64)
        } else {
            1.0
        };

        (up_capture, down_capture)
    }

    /// Calculate rolling beta over a window
    pub fn rolling_beta(
        portfolio_returns: &[f64],
        benchmark_returns: &[f64],
        window: usize,
    ) -> Vec<f64> {
        if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.len() < window {
            return vec![0.0; portfolio_returns.len()];
        }

        let mut result = vec![0.0; window - 1];

        for i in window..=portfolio_returns.len() {
            let port_window = &portfolio_returns[i - window..i];
            let bench_window = &benchmark_returns[i - window..i];

            let port_mean: f64 = port_window.iter().sum::<f64>() / window as f64;
            let bench_mean: f64 = bench_window.iter().sum::<f64>() / window as f64;

            let mut bench_var = 0.0;
            let mut covar = 0.0;

            for j in 0..window {
                let port_dev = port_window[j] - port_mean;
                let bench_dev = bench_window[j] - bench_mean;
                bench_var += bench_dev * bench_dev;
                covar += port_dev * bench_dev;
            }

            let beta = if bench_var > 1e-10 {
                covar / bench_var
            } else {
                1.0
            };

            result.push(beta);
        }

        result
    }

    /// Calculate rolling correlation over a window
    pub fn rolling_correlation(
        portfolio_returns: &[f64],
        benchmark_returns: &[f64],
        window: usize,
    ) -> Vec<f64> {
        if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.len() < window {
            return vec![0.0; portfolio_returns.len()];
        }

        let mut result = vec![0.0; window - 1];

        for i in window..=portfolio_returns.len() {
            let port_window = &portfolio_returns[i - window..i];
            let bench_window = &benchmark_returns[i - window..i];

            let port_mean: f64 = port_window.iter().sum::<f64>() / window as f64;
            let bench_mean: f64 = bench_window.iter().sum::<f64>() / window as f64;

            let mut port_var = 0.0;
            let mut bench_var = 0.0;
            let mut covar = 0.0;

            for j in 0..window {
                let port_dev = port_window[j] - port_mean;
                let bench_dev = bench_window[j] - bench_mean;
                port_var += port_dev * port_dev;
                bench_var += bench_dev * bench_dev;
                covar += port_dev * bench_dev;
            }

            let port_std = (port_var / window as f64).sqrt();
            let bench_std = (bench_var / window as f64).sqrt();

            let corr = if port_std > 1e-10 && bench_std > 1e-10 {
                covar / window as f64 / (port_std * bench_std)
            } else {
                0.0
            };

            result.push(corr);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_returns() -> (Vec<f64>, Vec<f64>) {
        // Portfolio slightly outperforming benchmark with higher beta
        let benchmark = vec![0.01, -0.005, 0.02, -0.01, 0.015, -0.008, 0.012, 0.005, -0.003, 0.008];
        let portfolio = vec![0.012, -0.006, 0.025, -0.012, 0.018, -0.01, 0.014, 0.006, -0.004, 0.01];
        (portfolio, benchmark)
    }

    #[test]
    fn test_benchmark_metrics() {
        let (portfolio, benchmark) = sample_returns();
        let metrics = BenchmarkAnalyzer::analyze(&portfolio, &benchmark).unwrap();

        // Beta should be > 1 (higher volatility than benchmark)
        assert!(metrics.beta > 0.9 && metrics.beta < 1.5);
        // High correlation expected
        assert!(metrics.correlation > 0.9);
        // Tracking error should be small
        assert!(metrics.tracking_error > 0.0);
    }

    #[test]
    fn test_empty_returns() {
        let metrics = BenchmarkAnalyzer::analyze(&[], &[]).unwrap();
        assert_eq!(metrics.beta, 1.0);
        assert_eq!(metrics.alpha, 0.0);
    }

    #[test]
    fn test_mismatched_lengths() {
        let result = BenchmarkAnalyzer::analyze(&[0.01, 0.02], &[0.01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rolling_beta() {
        let (portfolio, benchmark) = sample_returns();
        let rolling = BenchmarkAnalyzer::rolling_beta(&portfolio, &benchmark, 5);

        assert_eq!(rolling.len(), portfolio.len());
        // First 4 values should be 0 (window - 1)
        assert_eq!(rolling[0], 0.0);
        assert_eq!(rolling[3], 0.0);
        // After window, should have calculated values
        assert!(rolling[4] > 0.0);
    }

    #[test]
    fn test_rolling_correlation() {
        let (portfolio, benchmark) = sample_returns();
        let rolling = BenchmarkAnalyzer::rolling_correlation(&portfolio, &benchmark, 5);

        assert_eq!(rolling.len(), portfolio.len());
        // Correlations should be high
        assert!(rolling[9] > 0.8);
    }

    #[test]
    fn test_capture_ratios() {
        let (portfolio, benchmark) = sample_returns();
        let metrics = BenchmarkAnalyzer::analyze(&portfolio, &benchmark).unwrap();

        // Up capture > 1 means capturing more upside
        assert!(metrics.up_capture > 0.5);
        // Down capture > 1 means losing more on downside
        assert!(metrics.down_capture > 0.5);
    }

    #[test]
    fn test_perfect_tracking() {
        // Portfolio = benchmark means zero tracking error
        let benchmark = vec![0.01, -0.005, 0.02];
        let metrics = BenchmarkAnalyzer::analyze(&benchmark, &benchmark).unwrap();

        assert_eq!(metrics.beta, 1.0);
        assert!(metrics.tracking_error < 1e-10);
        assert!(metrics.correlation > 0.999);
    }
}
