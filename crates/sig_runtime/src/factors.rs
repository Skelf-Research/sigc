//! Factor models for risk decomposition and alpha generation
//!
//! Provides Fama-French, Barra-style, and custom factor implementations.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Fama-French factor model
pub struct FamaFrench {
    /// Market excess return (Mkt-RF)
    pub market: Series,
    /// Small minus Big (SMB)
    pub smb: Series,
    /// High minus Low (HML)
    pub hml: Series,
    /// Risk-free rate
    pub rf: Series,
    /// Momentum factor (optional, for 4-factor)
    pub mom: Option<Series>,
    /// Profitability (RMW) for 5-factor
    pub rmw: Option<Series>,
    /// Investment (CMA) for 5-factor
    pub cma: Option<Series>,
}

impl FamaFrench {
    /// Create a 3-factor model
    pub fn three_factor(market: Series, smb: Series, hml: Series, rf: Series) -> Self {
        FamaFrench {
            market,
            smb,
            hml,
            rf,
            mom: None,
            rmw: None,
            cma: None,
        }
    }

    /// Create a 5-factor model
    pub fn five_factor(
        market: Series,
        smb: Series,
        hml: Series,
        rf: Series,
        rmw: Series,
        cma: Series,
    ) -> Self {
        FamaFrench {
            market,
            smb,
            hml,
            rf,
            mom: None,
            rmw: Some(rmw),
            cma: Some(cma),
        }
    }

    /// Add momentum factor (Carhart 4-factor)
    pub fn with_momentum(mut self, mom: Series) -> Self {
        self.mom = Some(mom);
        self
    }

    /// Calculate factor exposures (betas) for a return series
    pub fn calculate_exposures(&self, returns: &Series, window: usize) -> Result<FactorExposures> {
        let n = returns.len();
        if n < window {
            return Err(SigcError::Runtime("Not enough data for factor regression".into()));
        }

        let ret_values = series_to_vec(returns)?;
        let mkt_values = series_to_vec(&self.market)?;
        let smb_values = series_to_vec(&self.smb)?;
        let hml_values = series_to_vec(&self.hml)?;
        let rf_values = series_to_vec(&self.rf)?;

        // Calculate excess returns
        let excess_returns: Vec<f64> = ret_values
            .iter()
            .zip(rf_values.iter())
            .map(|(r, rf)| r - rf)
            .collect();

        // Run rolling regression
        let mut alpha = vec![f64::NAN; window - 1];
        let mut beta_mkt = vec![f64::NAN; window - 1];
        let mut beta_smb = vec![f64::NAN; window - 1];
        let mut beta_hml = vec![f64::NAN; window - 1];

        for i in (window - 1)..n {
            let start = i + 1 - window;
            let y: Vec<f64> = excess_returns[start..=i].to_vec();
            let x_mkt: Vec<f64> = mkt_values[start..=i].to_vec();
            let x_smb: Vec<f64> = smb_values[start..=i].to_vec();
            let x_hml: Vec<f64> = hml_values[start..=i].to_vec();

            // Simple OLS regression
            let result = ols_regression_3factor(&y, &x_mkt, &x_smb, &x_hml);
            alpha.push(result.0);
            beta_mkt.push(result.1);
            beta_smb.push(result.2);
            beta_hml.push(result.3);
        }

        Ok(FactorExposures {
            alpha: Series::new("alpha".into(), alpha),
            beta_market: Series::new("beta_mkt".into(), beta_mkt),
            beta_smb: Series::new("beta_smb".into(), beta_smb),
            beta_hml: Series::new("beta_hml".into(), beta_hml),
            beta_mom: None,
            beta_rmw: None,
            beta_cma: None,
        })
    }

    /// Decompose returns into factor contributions
    pub fn decompose_returns(&self, returns: &Series, exposures: &FactorExposures) -> Result<ReturnDecomposition> {
        let n = returns.len();

        let ret_values = series_to_vec(returns)?;
        let rf_values = series_to_vec(&self.rf)?;
        let mkt_values = series_to_vec(&self.market)?;
        let smb_values = series_to_vec(&self.smb)?;
        let hml_values = series_to_vec(&self.hml)?;

        let alpha_values = series_to_vec(&exposures.alpha)?;
        let beta_mkt_values = series_to_vec(&exposures.beta_market)?;
        let beta_smb_values = series_to_vec(&exposures.beta_smb)?;
        let beta_hml_values = series_to_vec(&exposures.beta_hml)?;

        let mut market_contrib = Vec::with_capacity(n);
        let mut smb_contrib = Vec::with_capacity(n);
        let mut hml_contrib = Vec::with_capacity(n);
        let mut alpha_contrib = Vec::with_capacity(n);
        let mut residual = Vec::with_capacity(n);

        for i in 0..n {
            let mkt_c = beta_mkt_values[i] * mkt_values[i];
            let smb_c = beta_smb_values[i] * smb_values[i];
            let hml_c = beta_hml_values[i] * hml_values[i];
            let alpha_c = alpha_values[i];

            let excess_ret = ret_values[i] - rf_values[i];
            let explained = mkt_c + smb_c + hml_c + alpha_c;
            let resid = excess_ret - explained;

            market_contrib.push(mkt_c);
            smb_contrib.push(smb_c);
            hml_contrib.push(hml_c);
            alpha_contrib.push(alpha_c);
            residual.push(resid);
        }

        Ok(ReturnDecomposition {
            market: Series::new("market_contrib".into(), market_contrib),
            smb: Series::new("smb_contrib".into(), smb_contrib),
            hml: Series::new("hml_contrib".into(), hml_contrib),
            mom: None,
            rmw: None,
            cma: None,
            alpha: Series::new("alpha_contrib".into(), alpha_contrib),
            residual: Series::new("residual".into(), residual),
        })
    }
}

/// Factor exposures (betas)
#[derive(Debug, Clone)]
pub struct FactorExposures {
    pub alpha: Series,
    pub beta_market: Series,
    pub beta_smb: Series,
    pub beta_hml: Series,
    pub beta_mom: Option<Series>,
    pub beta_rmw: Option<Series>,
    pub beta_cma: Option<Series>,
}

/// Return decomposition by factor
#[derive(Debug, Clone)]
pub struct ReturnDecomposition {
    pub market: Series,
    pub smb: Series,
    pub hml: Series,
    pub mom: Option<Series>,
    pub rmw: Option<Series>,
    pub cma: Option<Series>,
    pub alpha: Series,
    pub residual: Series,
}

/// Barra-style risk model
pub struct BarraModel {
    /// Style factors
    pub style_factors: HashMap<String, Series>,
    /// Industry factors
    pub industry_factors: HashMap<String, Series>,
    /// Factor covariance matrix
    pub factor_covariance: Option<Vec<Vec<f64>>>,
    /// Specific risk
    pub specific_risk: Option<Series>,
}

impl BarraModel {
    /// Create a new Barra model
    pub fn new() -> Self {
        BarraModel {
            style_factors: HashMap::new(),
            industry_factors: HashMap::new(),
            factor_covariance: None,
            specific_risk: None,
        }
    }

    /// Add a style factor
    pub fn add_style_factor(&mut self, name: &str, values: Series) {
        self.style_factors.insert(name.to_string(), values);
    }

    /// Add an industry factor
    pub fn add_industry_factor(&mut self, name: &str, values: Series) {
        self.industry_factors.insert(name.to_string(), values);
    }

    /// Common style factors
    pub fn add_common_style_factors(
        &mut self,
        momentum: Series,
        value: Series,
        size: Series,
        volatility: Series,
        quality: Series,
        growth: Series,
    ) {
        self.style_factors.insert("momentum".to_string(), momentum);
        self.style_factors.insert("value".to_string(), value);
        self.style_factors.insert("size".to_string(), size);
        self.style_factors.insert("volatility".to_string(), volatility);
        self.style_factors.insert("quality".to_string(), quality);
        self.style_factors.insert("growth".to_string(), growth);
    }

    /// Calculate factor exposures for a portfolio
    pub fn portfolio_exposures(&self, weights: &Series) -> Result<HashMap<String, f64>> {
        let mut exposures = HashMap::new();
        let weight_values = series_to_vec(weights)?;

        // Style factor exposures
        for (name, factor) in &self.style_factors {
            let factor_values = series_to_vec(factor)?;
            let exposure: f64 = weight_values
                .iter()
                .zip(factor_values.iter())
                .map(|(w, f)| w * f)
                .sum();
            exposures.insert(name.clone(), exposure);
        }

        // Industry factor exposures
        for (name, factor) in &self.industry_factors {
            let factor_values = series_to_vec(factor)?;
            let exposure: f64 = weight_values
                .iter()
                .zip(factor_values.iter())
                .map(|(w, f)| w * f)
                .sum();
            exposures.insert(format!("industry_{}", name), exposure);
        }

        Ok(exposures)
    }

    /// Calculate portfolio factor risk
    pub fn portfolio_factor_risk(&self, weights: &Series) -> Result<f64> {
        let exposures = self.portfolio_exposures(weights)?;

        // If we have covariance matrix, use it
        if let Some(ref cov) = self.factor_covariance {
            let factor_names: Vec<String> = self.style_factors.keys().cloned().collect();
            let n = factor_names.len();

            let mut variance = 0.0;
            for i in 0..n {
                for j in 0..n {
                    let exp_i = exposures.get(&factor_names[i]).unwrap_or(&0.0);
                    let exp_j = exposures.get(&factor_names[j]).unwrap_or(&0.0);
                    variance += exp_i * exp_j * cov[i][j];
                }
            }

            Ok(variance.sqrt())
        } else {
            // Simple sum of squared exposures
            let var: f64 = exposures.values().map(|e| e * e).sum();
            Ok(var.sqrt())
        }
    }
}

impl Default for BarraModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom factor builder
pub struct FactorBuilder {
    factors: HashMap<String, Series>,
}

impl FactorBuilder {
    pub fn new() -> Self {
        FactorBuilder {
            factors: HashMap::new(),
        }
    }

    /// Add a custom factor
    pub fn add_factor(mut self, name: &str, values: Series) -> Self {
        self.factors.insert(name.to_string(), values);
        self
    }

    /// Build momentum factor from prices
    pub fn momentum_factor(mut self, prices: &Series, window: usize) -> Result<Self> {
        let values = series_to_vec(prices)?;
        let n = values.len();
        let mut momentum = vec![f64::NAN; window];

        for i in window..n {
            momentum.push(values[i] / values[i - window] - 1.0);
        }

        self.factors.insert(
            "momentum".to_string(),
            Series::new("momentum".into(), momentum),
        );
        Ok(self)
    }

    /// Build value factor from price/earnings
    pub fn value_factor(mut self, prices: &Series, earnings: &Series) -> Result<Self> {
        let price_values = series_to_vec(prices)?;
        let earnings_values = series_to_vec(earnings)?;

        let value: Vec<f64> = earnings_values
            .iter()
            .zip(price_values.iter())
            .map(|(e, p)| if *p != 0.0 { e / p } else { 0.0 })
            .collect();

        self.factors.insert(
            "value".to_string(),
            Series::new("value".into(), value),
        );
        Ok(self)
    }

    /// Build size factor from market cap
    pub fn size_factor(mut self, market_cap: &Series) -> Result<Self> {
        let values = series_to_vec(market_cap)?;
        let size: Vec<f64> = values.iter().map(|v| v.ln()).collect();

        self.factors.insert(
            "size".to_string(),
            Series::new("size".into(), size),
        );
        Ok(self)
    }

    /// Build volatility factor
    pub fn volatility_factor(mut self, returns: &Series, window: usize) -> Result<Self> {
        let values = series_to_vec(returns)?;
        let n = values.len();
        let mut vol = vec![f64::NAN; window - 1];

        for i in (window - 1)..n {
            let slice = &values[(i + 1 - window)..=i];
            let mean: f64 = slice.iter().sum::<f64>() / window as f64;
            let variance: f64 = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (window - 1) as f64;
            vol.push(variance.sqrt());
        }

        self.factors.insert(
            "volatility".to_string(),
            Series::new("volatility".into(), vol),
        );
        Ok(self)
    }

    /// Get all factors
    pub fn build(self) -> HashMap<String, Series> {
        self.factors
    }
}

impl Default for FactorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Factor analysis results
#[derive(Debug, Clone)]
pub struct FactorAnalysis {
    pub factor_returns: HashMap<String, f64>,
    pub factor_sharpe: HashMap<String, f64>,
    pub factor_correlation: Vec<Vec<f64>>,
    pub factor_names: Vec<String>,
}

/// Analyze factor performance
pub fn analyze_factors(factors: &HashMap<String, Series>, _window: usize) -> Result<FactorAnalysis> {
    let factor_names: Vec<String> = factors.keys().cloned().collect();
    let n_factors = factor_names.len();

    let mut factor_returns = HashMap::new();
    let mut factor_sharpe = HashMap::new();

    // Calculate returns and Sharpe for each factor
    for name in &factor_names {
        let values = series_to_vec(factors.get(name).unwrap())?;
        let mean: f64 = values.iter().filter(|v| !v.is_nan()).sum::<f64>()
            / values.iter().filter(|v| !v.is_nan()).count() as f64;

        let variance: f64 = values.iter()
            .filter(|v| !v.is_nan())
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.iter().filter(|v| !v.is_nan()).count() as f64;

        let std = variance.sqrt();
        let sharpe = if std > 0.0 { mean / std * (252.0_f64).sqrt() } else { 0.0 };

        factor_returns.insert(name.clone(), mean * 252.0);
        factor_sharpe.insert(name.clone(), sharpe);
    }

    // Calculate correlation matrix
    let mut correlation = vec![vec![0.0; n_factors]; n_factors];
    for i in 0..n_factors {
        for j in 0..n_factors {
            if i == j {
                correlation[i][j] = 1.0;
            } else {
                let series_i = series_to_vec(factors.get(&factor_names[i]).unwrap())?;
                let series_j = series_to_vec(factors.get(&factor_names[j]).unwrap())?;
                correlation[i][j] = calculate_correlation(&series_i, &series_j);
            }
        }
    }

    Ok(FactorAnalysis {
        factor_returns,
        factor_sharpe,
        factor_correlation: correlation,
        factor_names,
    })
}

// Helper functions

fn series_to_vec(series: &Series) -> Result<Vec<f64>> {
    series
        .f64()
        .map_err(|e| SigcError::Runtime(format!("Failed to convert series to f64: {}", e)))?
        .into_iter()
        .map(|v| Ok(v.unwrap_or(f64::NAN)))
        .collect()
}

fn ols_regression_3factor(y: &[f64], x1: &[f64], x2: &[f64], x3: &[f64]) -> (f64, f64, f64, f64) {
    let n = y.len() as f64;

    // Means
    let y_mean: f64 = y.iter().sum::<f64>() / n;
    let x1_mean: f64 = x1.iter().sum::<f64>() / n;
    let x2_mean: f64 = x2.iter().sum::<f64>() / n;
    let x3_mean: f64 = x3.iter().sum::<f64>() / n;

    // Simple approximation using separate regressions
    // (Full multivariate OLS would need matrix operations)
    let mut cov_y_x1 = 0.0;
    let mut var_x1 = 0.0;
    let mut cov_y_x2 = 0.0;
    let mut var_x2 = 0.0;
    let mut cov_y_x3 = 0.0;
    let mut var_x3 = 0.0;

    for i in 0..y.len() {
        let dy = y[i] - y_mean;
        let dx1 = x1[i] - x1_mean;
        let dx2 = x2[i] - x2_mean;
        let dx3 = x3[i] - x3_mean;

        cov_y_x1 += dy * dx1;
        var_x1 += dx1 * dx1;
        cov_y_x2 += dy * dx2;
        var_x2 += dx2 * dx2;
        cov_y_x3 += dy * dx3;
        var_x3 += dx3 * dx3;
    }

    let beta1 = if var_x1 > 0.0 { cov_y_x1 / var_x1 } else { 0.0 };
    let beta2 = if var_x2 > 0.0 { cov_y_x2 / var_x2 } else { 0.0 };
    let beta3 = if var_x3 > 0.0 { cov_y_x3 / var_x3 } else { 0.0 };

    let alpha = y_mean - beta1 * x1_mean - beta2 * x2_mean - beta3 * x3_mean;

    (alpha, beta1, beta2, beta3)
}

fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n == 0 {
        return 0.0;
    }

    let x_mean: f64 = x.iter().filter(|v| !v.is_nan()).sum::<f64>()
        / x.iter().filter(|v| !v.is_nan()).count() as f64;
    let y_mean: f64 = y.iter().filter(|v| !v.is_nan()).sum::<f64>()
        / y.iter().filter(|v| !v.is_nan()).count() as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    let mut count = 0;

    for i in 0..n {
        if !x[i].is_nan() && !y[i].is_nan() {
            let dx = x[i] - x_mean;
            let dy = y[i] - y_mean;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
            count += 1;
        }
    }

    if var_x > 0.0 && var_y > 0.0 && count > 0 {
        cov / (var_x.sqrt() * var_y.sqrt())
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fama_french_creation() {
        let market = Series::new("mkt".into(), vec![0.01, 0.02, -0.01, 0.015]);
        let smb = Series::new("smb".into(), vec![0.005, -0.003, 0.002, 0.001]);
        let hml = Series::new("hml".into(), vec![-0.002, 0.004, 0.001, -0.001]);
        let rf = Series::new("rf".into(), vec![0.0001, 0.0001, 0.0001, 0.0001]);

        let ff = FamaFrench::three_factor(market, smb, hml, rf);
        assert!(ff.mom.is_none());
        assert!(ff.rmw.is_none());
    }

    #[test]
    fn test_barra_model() {
        let mut model = BarraModel::new();

        let momentum = Series::new("mom".into(), vec![0.1, 0.2, -0.1, 0.15]);
        model.add_style_factor("momentum", momentum);

        assert!(model.style_factors.contains_key("momentum"));
    }

    #[test]
    fn test_factor_builder() {
        let prices = Series::new("prices".into(), vec![100.0, 102.0, 101.0, 105.0, 103.0]);

        let builder = FactorBuilder::new()
            .momentum_factor(&prices, 2)
            .unwrap();

        let factors = builder.build();
        assert!(factors.contains_key("momentum"));
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let corr = calculate_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_portfolio_exposures() {
        let mut model = BarraModel::new();

        model.add_style_factor("momentum", Series::new("mom".into(), vec![0.5, -0.3, 0.2]));
        model.add_style_factor("value", Series::new("val".into(), vec![-0.2, 0.4, 0.1]));

        let weights = Series::new("w".into(), vec![0.4, 0.3, 0.3]);
        let exposures = model.portfolio_exposures(&weights).unwrap();

        assert!(exposures.contains_key("momentum"));
        assert!(exposures.contains_key("value"));
    }
}
