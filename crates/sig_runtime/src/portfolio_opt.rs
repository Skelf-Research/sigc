//! Portfolio optimization algorithms
//!
//! Provides mean-variance, risk parity, and Black-Litterman optimization.

use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Portfolio optimization result
#[derive(Debug, Clone)]
pub struct OptimalPortfolio {
    /// Asset weights
    pub weights: Vec<f64>,
    /// Asset names/symbols
    pub assets: Vec<String>,
    /// Expected return
    pub expected_return: f64,
    /// Expected volatility
    pub expected_volatility: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
}

impl OptimalPortfolio {
    /// Get weights as a HashMap
    pub fn weights_map(&self) -> HashMap<String, f64> {
        self.assets
            .iter()
            .zip(self.weights.iter())
            .map(|(a, w)| (a.clone(), *w))
            .collect()
    }
}

/// Mean-variance optimizer
pub struct MeanVarianceOptimizer {
    /// Risk-free rate
    pub risk_free_rate: f64,
    /// Constraints
    pub constraints: PortfolioConstraints,
}

/// Portfolio constraints
#[derive(Debug, Clone)]
pub struct PortfolioConstraints {
    /// Minimum weight per asset
    pub min_weight: f64,
    /// Maximum weight per asset
    pub max_weight: f64,
    /// Long only (no shorts)
    pub long_only: bool,
    /// Target volatility (optional)
    pub target_vol: Option<f64>,
}

impl Default for PortfolioConstraints {
    fn default() -> Self {
        PortfolioConstraints {
            min_weight: 0.0,
            max_weight: 1.0,
            long_only: true,
            target_vol: None,
        }
    }
}

impl MeanVarianceOptimizer {
    pub fn new(risk_free_rate: f64) -> Self {
        MeanVarianceOptimizer {
            risk_free_rate,
            constraints: PortfolioConstraints::default(),
        }
    }

    /// Set constraints
    pub fn with_constraints(mut self, constraints: PortfolioConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Find minimum variance portfolio
    pub fn minimum_variance(
        &self,
        assets: &[String],
        covariance: &[Vec<f64>],
    ) -> Result<OptimalPortfolio> {
        let n = assets.len();
        if covariance.len() != n || covariance.iter().any(|row| row.len() != n) {
            return Err(SigcError::Runtime("Invalid covariance matrix dimensions".into()));
        }

        // Analytical solution for unconstrained min variance:
        // w = Σ^(-1) * 1 / (1' * Σ^(-1) * 1)

        // For constrained case, use iterative approach
        let mut weights = vec![1.0 / n as f64; n];

        // Simple gradient descent
        for _ in 0..1000 {
            // Calculate gradient of variance
            let mut gradient = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    gradient[i] += 2.0 * covariance[i][j] * weights[j];
                }
            }

            // Update weights
            let lr = 0.01;
            let mut new_weights = vec![0.0; n];
            for i in 0..n {
                new_weights[i] = weights[i] - lr * gradient[i];
            }

            // Apply constraints
            self.apply_constraints(&mut new_weights);

            // Normalize
            let sum: f64 = new_weights.iter().sum();
            if sum > 0.0 {
                for w in &mut new_weights {
                    *w /= sum;
                }
            }

            weights = new_weights;
        }

        // Calculate portfolio metrics
        let variance = portfolio_variance(&weights, covariance);
        let volatility = variance.sqrt();

        Ok(OptimalPortfolio {
            weights,
            assets: assets.to_vec(),
            expected_return: 0.0, // Not specified for min variance
            expected_volatility: volatility,
            sharpe_ratio: 0.0,
        })
    }

    /// Find maximum Sharpe ratio portfolio
    pub fn max_sharpe(
        &self,
        assets: &[String],
        expected_returns: &[f64],
        covariance: &[Vec<f64>],
    ) -> Result<OptimalPortfolio> {
        let n = assets.len();
        if expected_returns.len() != n {
            return Err(SigcError::Runtime("Expected returns length mismatch".into()));
        }

        // Grid search over different target returns
        let min_ret = expected_returns.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ret = expected_returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let mut best_sharpe = f64::NEG_INFINITY;
        let mut best_weights = vec![1.0 / n as f64; n];

        for i in 0..20 {
            let target_return = min_ret + (max_ret - min_ret) * i as f64 / 19.0;

            if let Ok(portfolio) = self.efficient_portfolio(
                assets,
                expected_returns,
                covariance,
                target_return,
            ) {
                if portfolio.sharpe_ratio > best_sharpe {
                    best_sharpe = portfolio.sharpe_ratio;
                    best_weights = portfolio.weights;
                }
            }
        }

        let exp_ret = portfolio_return(&best_weights, expected_returns);
        let variance = portfolio_variance(&best_weights, covariance);
        let volatility = variance.sqrt();
        let sharpe = if volatility > 0.0 {
            (exp_ret - self.risk_free_rate) / volatility
        } else {
            0.0
        };

        Ok(OptimalPortfolio {
            weights: best_weights,
            assets: assets.to_vec(),
            expected_return: exp_ret,
            expected_volatility: volatility,
            sharpe_ratio: sharpe,
        })
    }

    /// Find efficient portfolio for a target return
    pub fn efficient_portfolio(
        &self,
        assets: &[String],
        expected_returns: &[f64],
        covariance: &[Vec<f64>],
        target_return: f64,
    ) -> Result<OptimalPortfolio> {
        let n = assets.len();
        let mut weights = vec![1.0 / n as f64; n];

        // Lagrangian optimization with gradient descent
        let mut lambda = 0.0; // Return constraint multiplier

        for _ in 0..1000 {
            // Gradient of Lagrangian: 2Σw - λμ
            let mut gradient = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    gradient[i] += 2.0 * covariance[i][j] * weights[j];
                }
                gradient[i] -= lambda * expected_returns[i];
            }

            // Update weights
            let lr = 0.01;
            for i in 0..n {
                weights[i] -= lr * gradient[i];
            }

            // Apply constraints
            self.apply_constraints(&mut weights);

            // Normalize
            let sum: f64 = weights.iter().sum();
            if sum > 0.0 {
                for w in &mut weights {
                    *w /= sum;
                }
            }

            // Update lambda based on return constraint
            let current_return = portfolio_return(&weights, expected_returns);
            lambda += 0.1 * (target_return - current_return);
        }

        let exp_ret = portfolio_return(&weights, expected_returns);
        let variance = portfolio_variance(&weights, covariance);
        let volatility = variance.sqrt();
        let sharpe = if volatility > 0.0 {
            (exp_ret - self.risk_free_rate) / volatility
        } else {
            0.0
        };

        Ok(OptimalPortfolio {
            weights,
            assets: assets.to_vec(),
            expected_return: exp_ret,
            expected_volatility: volatility,
            sharpe_ratio: sharpe,
        })
    }

    /// Apply constraints to weights
    fn apply_constraints(&self, weights: &mut [f64]) {
        for w in weights.iter_mut() {
            if self.constraints.long_only && *w < 0.0 {
                *w = 0.0;
            }
            *w = w.clamp(self.constraints.min_weight, self.constraints.max_weight);
        }
    }
}

/// Risk parity optimizer
pub struct RiskParityOptimizer {
    /// Target volatility (optional)
    pub target_vol: Option<f64>,
}

impl RiskParityOptimizer {
    pub fn new() -> Self {
        RiskParityOptimizer { target_vol: None }
    }

    /// Set target volatility
    pub fn with_target_vol(mut self, vol: f64) -> Self {
        self.target_vol = Some(vol);
        self
    }

    /// Find risk parity portfolio (equal risk contribution)
    pub fn optimize(
        &self,
        assets: &[String],
        covariance: &[Vec<f64>],
    ) -> Result<OptimalPortfolio> {
        let n = assets.len();

        // Initialize with equal volatility weights
        let mut weights = vec![1.0 / n as f64; n];

        // Iterative optimization
        for _ in 0..1000 {
            // Calculate risk contributions
            let total_var = portfolio_variance(&weights, covariance);
            let total_vol = total_var.sqrt();

            if total_vol == 0.0 {
                break;
            }

            let mut risk_contrib = vec![0.0; n];
            for i in 0..n {
                let mut marginal = 0.0;
                for j in 0..n {
                    marginal += covariance[i][j] * weights[j];
                }
                risk_contrib[i] = weights[i] * marginal / total_vol;
            }

            // Target: equal risk contribution
            let target_contrib = total_vol / n as f64;

            // Update weights
            for i in 0..n {
                if risk_contrib[i] > 0.0 {
                    let adjustment = target_contrib / risk_contrib[i];
                    weights[i] *= adjustment.powf(0.5); // Dampened update
                }
            }

            // Normalize
            let sum: f64 = weights.iter().sum();
            if sum > 0.0 {
                for w in &mut weights {
                    *w /= sum;
                }
            }
        }

        // Scale to target volatility if specified
        let variance = portfolio_variance(&weights, covariance);
        let volatility = variance.sqrt();

        if let Some(target) = self.target_vol {
            if volatility > 0.0 {
                let scale = target / volatility;
                for w in &mut weights {
                    *w *= scale;
                }
            }
        }

        Ok(OptimalPortfolio {
            weights,
            assets: assets.to_vec(),
            expected_return: 0.0,
            expected_volatility: volatility,
            sharpe_ratio: 0.0,
        })
    }
}

impl Default for RiskParityOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Black-Litterman model
pub struct BlackLitterman {
    /// Risk aversion parameter
    pub risk_aversion: f64,
    /// Confidence in views (tau)
    pub tau: f64,
}

/// A view on expected returns
#[derive(Debug, Clone)]
pub struct View {
    /// Pick matrix row (which assets)
    pub assets: Vec<String>,
    /// Weights in the view (sum to 0 for relative views)
    pub weights: Vec<f64>,
    /// Expected return of the view
    pub expected_return: f64,
    /// Confidence (lower = more confident)
    pub confidence: f64,
}

impl BlackLitterman {
    pub fn new(risk_aversion: f64, tau: f64) -> Self {
        BlackLitterman { risk_aversion, tau }
    }

    /// Calculate equilibrium returns from market cap weights
    pub fn equilibrium_returns(
        &self,
        market_weights: &[f64],
        covariance: &[Vec<f64>],
    ) -> Vec<f64> {
        let n = market_weights.len();
        let mut pi = vec![0.0; n];

        for i in 0..n {
            for j in 0..n {
                pi[i] += self.risk_aversion * covariance[i][j] * market_weights[j];
            }
        }

        pi
    }

    /// Combine equilibrium with views
    pub fn posterior_returns(
        &self,
        assets: &[String],
        equilibrium: &[f64],
        _covariance: &[Vec<f64>],
        views: &[View],
    ) -> Result<Vec<f64>> {
        let n = assets.len();

        if views.is_empty() {
            return Ok(equilibrium.to_vec());
        }

        // Build P matrix (views × assets)
        let k = views.len();
        let mut p = vec![vec![0.0; n]; k];
        let mut q = vec![0.0; k];
        let mut omega = vec![vec![0.0; k]; k];

        for (v_idx, view) in views.iter().enumerate() {
            q[v_idx] = view.expected_return;
            omega[v_idx][v_idx] = view.confidence;

            for (a_idx, asset) in view.assets.iter().enumerate() {
                if let Some(pos) = assets.iter().position(|a| a == asset) {
                    p[v_idx][pos] = view.weights[a_idx];
                }
            }
        }

        // Simplified Black-Litterman formula
        // E[R] = [(τΣ)^(-1) + P'Ω^(-1)P]^(-1) × [(τΣ)^(-1)π + P'Ω^(-1)q]

        // For simplicity, use a weighted average approach
        let mut posterior = equilibrium.to_vec();

        for (_v_idx, view) in views.iter().enumerate() {
            let confidence_weight = 1.0 / (1.0 + view.confidence);

            for (a_idx, asset) in view.assets.iter().enumerate() {
                if let Some(pos) = assets.iter().position(|a| a == asset) {
                    let view_contribution = view.weights[a_idx] * view.expected_return;
                    posterior[pos] = (1.0 - confidence_weight) * posterior[pos]
                        + confidence_weight * view_contribution;
                }
            }
        }

        Ok(posterior)
    }

    /// Optimize portfolio with Black-Litterman
    pub fn optimize(
        &self,
        assets: &[String],
        market_weights: &[f64],
        covariance: &[Vec<f64>],
        views: &[View],
    ) -> Result<OptimalPortfolio> {
        // Calculate equilibrium returns
        let equilibrium = self.equilibrium_returns(market_weights, covariance);

        // Get posterior returns
        let posterior = self.posterior_returns(assets, &equilibrium, covariance, views)?;

        // Optimize using mean-variance
        let optimizer = MeanVarianceOptimizer::new(0.02);
        optimizer.max_sharpe(assets, &posterior, covariance)
    }
}

/// Hierarchical Risk Parity
pub struct HierarchicalRiskParity {
    /// Linkage method
    pub linkage: String,
}

impl HierarchicalRiskParity {
    pub fn new() -> Self {
        HierarchicalRiskParity {
            linkage: "single".to_string(),
        }
    }

    /// Calculate distance matrix from correlation
    fn distance_matrix(correlation: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = correlation.len();
        let mut dist = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                dist[i][j] = ((1.0 - correlation[i][j]) / 2.0).sqrt();
            }
        }

        dist
    }

    /// Simple hierarchical clustering (returns order)
    fn cluster_order(&self, correlation: &[Vec<f64>]) -> Vec<usize> {
        let n = correlation.len();
        let dist = Self::distance_matrix(correlation);

        // Simple ordering by sum of correlations
        let mut order: Vec<(usize, f64)> = (0..n)
            .map(|i| {
                let sum: f64 = (0..n).map(|j| dist[i][j]).sum();
                (i, sum)
            })
            .collect();

        order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        order.iter().map(|(i, _)| *i).collect()
    }

    /// Optimize using HRP
    pub fn optimize(
        &self,
        assets: &[String],
        covariance: &[Vec<f64>],
    ) -> Result<OptimalPortfolio> {
        let n = assets.len();

        // Calculate correlation from covariance
        let mut correlation = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let denom = (covariance[i][i] * covariance[j][j]).sqrt();
                correlation[i][j] = if denom > 0.0 {
                    covariance[i][j] / denom
                } else {
                    0.0
                };
            }
        }

        // Get cluster order
        let order = self.cluster_order(&correlation);

        // Recursive bisection for weights
        let mut weights = vec![1.0; n];
        self.recursive_bisection(&mut weights, &order, covariance, 0, n);

        // Normalize
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }

        let variance = portfolio_variance(&weights, covariance);
        let volatility = variance.sqrt();

        Ok(OptimalPortfolio {
            weights,
            assets: assets.to_vec(),
            expected_return: 0.0,
            expected_volatility: volatility,
            sharpe_ratio: 0.0,
        })
    }

    /// Recursive bisection for HRP
    fn recursive_bisection(
        &self,
        weights: &mut [f64],
        order: &[usize],
        covariance: &[Vec<f64>],
        start: usize,
        end: usize,
    ) {
        if end - start <= 1 {
            return;
        }

        let mid = (start + end) / 2;

        // Calculate cluster variances
        let left_var = self.cluster_variance(order, covariance, start, mid);
        let right_var = self.cluster_variance(order, covariance, mid, end);

        // Allocate inversely proportional to variance
        let total_inv_var = 1.0 / left_var + 1.0 / right_var;
        let left_weight = (1.0 / left_var) / total_inv_var;
        let right_weight = (1.0 / right_var) / total_inv_var;

        // Apply weights
        for i in start..mid {
            weights[order[i]] *= left_weight;
        }
        for i in mid..end {
            weights[order[i]] *= right_weight;
        }

        // Recurse
        self.recursive_bisection(weights, order, covariance, start, mid);
        self.recursive_bisection(weights, order, covariance, mid, end);
    }

    /// Calculate cluster variance
    fn cluster_variance(
        &self,
        order: &[usize],
        covariance: &[Vec<f64>],
        start: usize,
        end: usize,
    ) -> f64 {
        let cluster_size = end - start;
        if cluster_size == 0 {
            return 1.0;
        }

        let equal_weight = 1.0 / cluster_size as f64;
        let mut variance = 0.0;

        for i in start..end {
            for j in start..end {
                variance += equal_weight * equal_weight * covariance[order[i]][order[j]];
            }
        }

        variance.max(1e-10)
    }
}

impl Default for HierarchicalRiskParity {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn portfolio_return(weights: &[f64], returns: &[f64]) -> f64 {
    weights
        .iter()
        .zip(returns.iter())
        .map(|(w, r)| w * r)
        .sum()
}

fn portfolio_variance(weights: &[f64], covariance: &[Vec<f64>]) -> f64 {
    let n = weights.len();
    let mut variance = 0.0;

    for i in 0..n {
        for j in 0..n {
            variance += weights[i] * weights[j] * covariance[i][j];
        }
    }

    variance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> (Vec<String>, Vec<f64>, Vec<Vec<f64>>) {
        let assets = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let returns = vec![0.10, 0.08, 0.06];
        let covariance = vec![
            vec![0.04, 0.01, 0.005],
            vec![0.01, 0.03, 0.01],
            vec![0.005, 0.01, 0.02],
        ];
        (assets, returns, covariance)
    }

    #[test]
    fn test_min_variance() {
        let (assets, _, covariance) = sample_data();
        let optimizer = MeanVarianceOptimizer::new(0.02);
        let portfolio = optimizer.minimum_variance(&assets, &covariance).unwrap();

        // Weights should sum to 1
        let sum: f64 = portfolio.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);

        // Volatility should be positive
        assert!(portfolio.expected_volatility > 0.0);
    }

    #[test]
    fn test_max_sharpe() {
        let (assets, returns, covariance) = sample_data();
        let optimizer = MeanVarianceOptimizer::new(0.02);
        let portfolio = optimizer.max_sharpe(&assets, &returns, &covariance).unwrap();

        // Weights should sum to 1
        let sum: f64 = portfolio.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);

        // Sharpe should be positive for these returns
        assert!(portfolio.sharpe_ratio > 0.0);
    }

    #[test]
    fn test_risk_parity() {
        let (assets, _, covariance) = sample_data();
        let optimizer = RiskParityOptimizer::new();
        let portfolio = optimizer.optimize(&assets, &covariance).unwrap();

        // Weights should sum to 1
        let sum: f64 = portfolio.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_black_litterman() {
        let (assets, _, covariance) = sample_data();
        let market_weights = vec![0.5, 0.3, 0.2];

        let bl = BlackLitterman::new(2.5, 0.05);
        let equilibrium = bl.equilibrium_returns(&market_weights, &covariance);

        assert_eq!(equilibrium.len(), 3);
        // Higher weight assets should have higher equilibrium returns
        assert!(equilibrium[0] > equilibrium[2]);
    }

    #[test]
    fn test_black_litterman_with_views() {
        let (assets, _, covariance) = sample_data();
        let market_weights = vec![0.5, 0.3, 0.2];

        let views = vec![View {
            assets: vec!["A".to_string()],
            weights: vec![1.0],
            expected_return: 0.15,
            confidence: 0.1,
        }];

        let bl = BlackLitterman::new(2.5, 0.05);
        let portfolio = bl.optimize(&assets, &market_weights, &covariance, &views).unwrap();

        // Should favor asset A given the bullish view
        assert!(portfolio.weights[0] > 0.3);
    }

    #[test]
    fn test_hrp() {
        let (assets, _, covariance) = sample_data();
        let hrp = HierarchicalRiskParity::new();
        let portfolio = hrp.optimize(&assets, &covariance).unwrap();

        // Weights should sum to 1
        let sum: f64 = portfolio.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_portfolio_constraints() {
        let (assets, returns, covariance) = sample_data();

        let constraints = PortfolioConstraints {
            min_weight: 0.1,
            max_weight: 0.5,
            long_only: true,
            target_vol: None,
        };

        let optimizer = MeanVarianceOptimizer::new(0.02).with_constraints(constraints);
        let portfolio = optimizer.max_sharpe(&assets, &returns, &covariance).unwrap();

        // All weights should respect constraints
        for w in &portfolio.weights {
            assert!(*w >= 0.0); // Long only
            assert!(*w <= 0.5); // Max weight
        }
    }

    #[test]
    fn test_optimal_portfolio_map() {
        let (assets, returns, covariance) = sample_data();
        let optimizer = MeanVarianceOptimizer::new(0.02);
        let portfolio = optimizer.max_sharpe(&assets, &returns, &covariance).unwrap();

        let map = portfolio.weights_map();
        assert!(map.contains_key("A"));
        assert!(map.contains_key("B"));
        assert!(map.contains_key("C"));
    }
}
