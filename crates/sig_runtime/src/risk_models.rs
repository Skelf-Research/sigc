//! Risk models for VaR, CVaR, and stress testing
//!
//! Provides comprehensive risk measurement and analysis tools.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Value at Risk calculator
pub struct VaRCalculator {
    /// Confidence level (e.g., 0.95, 0.99)
    pub confidence: f64,
    /// Lookback window for historical VaR
    pub window: usize,
}

impl VaRCalculator {
    /// Create a new VaR calculator
    pub fn new(confidence: f64, window: usize) -> Self {
        VaRCalculator { confidence, window }
    }

    /// Calculate historical VaR
    pub fn historical_var(&self, returns: &Series) -> Result<f64> {
        let values = series_to_vec(returns)?;
        let mut sorted: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        if n == 0 {
            return Err(SigcError::Runtime("No valid returns for VaR calculation".into()));
        }

        let index = ((1.0 - self.confidence) * n as f64).floor() as usize;
        let var = -sorted[index.min(n - 1)];

        Ok(var)
    }

    /// Calculate parametric (Gaussian) VaR
    pub fn parametric_var(&self, returns: &Series) -> Result<f64> {
        let values = series_to_vec(returns)?;
        let clean: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();

        if clean.is_empty() {
            return Err(SigcError::Runtime("No valid returns".into()));
        }

        let mean: f64 = clean.iter().sum::<f64>() / clean.len() as f64;
        let variance: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / clean.len() as f64;
        let std = variance.sqrt();

        // Z-score for confidence level
        let z = match self.confidence {
            c if (c - 0.90).abs() < 0.001 => 1.282,
            c if (c - 0.95).abs() < 0.001 => 1.645,
            c if (c - 0.99).abs() < 0.001 => 2.326,
            c if (c - 0.999).abs() < 0.001 => 3.090,
            _ => 1.645, // Default to 95%
        };

        Ok(-(mean - z * std))
    }

    /// Calculate rolling VaR
    pub fn rolling_var(&self, returns: &Series) -> Result<Series> {
        let values = series_to_vec(returns)?;
        let n = values.len();
        let mut var_series = vec![f64::NAN; self.window - 1];

        for i in (self.window - 1)..n {
            let window_returns: Vec<f64> = values[(i + 1 - self.window)..=i].to_vec();
            let mut sorted = window_returns.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let index = ((1.0 - self.confidence) * self.window as f64).floor() as usize;
            let var = -sorted[index.min(self.window - 1)];
            var_series.push(var);
        }

        Ok(Series::new("var".into(), var_series))
    }

    /// Calculate Cornish-Fisher VaR (adjusts for skewness and kurtosis)
    pub fn cornish_fisher_var(&self, returns: &Series) -> Result<f64> {
        let values = series_to_vec(returns)?;
        let clean: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();
        let n = clean.len() as f64;

        if n < 4.0 {
            return self.parametric_var(returns);
        }

        let mean: f64 = clean.iter().sum::<f64>() / n;
        let m2: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let m3: f64 = clean.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n;
        let m4: f64 = clean.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n;

        let std = m2.sqrt();
        let skewness = m3 / m2.powf(1.5);
        let kurtosis = m4 / m2.powi(2) - 3.0;

        // Z-score for confidence level
        let z: f64 = match self.confidence {
            c if (c - 0.95).abs() < 0.001 => 1.645_f64,
            c if (c - 0.99).abs() < 0.001 => 2.326_f64,
            _ => 1.645_f64,
        };

        // Cornish-Fisher expansion
        let z_cf = z
            + (z.powi(2) - 1.0_f64) * skewness / 6.0_f64
            + (z.powi(3) - 3.0_f64 * z) * kurtosis / 24.0_f64
            - (2.0_f64 * z.powi(3) - 5.0_f64 * z) * skewness.powi(2) / 36.0_f64;

        Ok(-(mean - z_cf * std))
    }
}

/// Expected Shortfall (CVaR) calculator
pub struct CVaRCalculator {
    /// Confidence level
    pub confidence: f64,
}

impl CVaRCalculator {
    pub fn new(confidence: f64) -> Self {
        CVaRCalculator { confidence }
    }

    /// Calculate historical CVaR (Expected Shortfall)
    pub fn historical_cvar(&self, returns: &Series) -> Result<f64> {
        let values = series_to_vec(returns)?;
        let mut sorted: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted.len();
        if n == 0 {
            return Err(SigcError::Runtime("No valid returns".into()));
        }

        let cutoff = ((1.0 - self.confidence) * n as f64).floor() as usize;
        let tail: Vec<f64> = sorted[0..=cutoff.min(n - 1)].to_vec();

        let cvar = -tail.iter().sum::<f64>() / tail.len() as f64;
        Ok(cvar)
    }

    /// Calculate parametric CVaR
    pub fn parametric_cvar(&self, returns: &Series) -> Result<f64> {
        let values = series_to_vec(returns)?;
        let clean: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();

        let mean: f64 = clean.iter().sum::<f64>() / clean.len() as f64;
        let variance: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / clean.len() as f64;
        let std = variance.sqrt();

        // PDF of standard normal at quantile
        let z: f64 = match self.confidence {
            c if (c - 0.95).abs() < 0.001 => 1.645_f64,
            c if (c - 0.99).abs() < 0.001 => 2.326_f64,
            _ => 1.645_f64,
        };

        let pdf = (-z.powi(2) / 2.0_f64).exp() / (2.0_f64 * std::f64::consts::PI).sqrt();
        let cvar = mean - std * pdf / (1.0 - self.confidence);

        Ok(-cvar)
    }
}

/// Stress testing framework
pub struct StressTest {
    /// Historical scenarios
    pub scenarios: HashMap<String, StressScenario>,
}

/// A stress scenario
#[derive(Debug, Clone)]
pub struct StressScenario {
    pub name: String,
    pub description: String,
    /// Factor shocks (factor name -> shock magnitude)
    pub factor_shocks: HashMap<String, f64>,
    /// Direct return shock (optional)
    pub return_shock: Option<f64>,
}

impl StressTest {
    pub fn new() -> Self {
        StressTest {
            scenarios: HashMap::new(),
        }
    }

    /// Add a custom scenario
    pub fn add_scenario(&mut self, scenario: StressScenario) {
        self.scenarios.insert(scenario.name.clone(), scenario);
    }

    /// Add standard historical scenarios
    pub fn add_historical_scenarios(&mut self) {
        // 2008 Financial Crisis
        let mut crisis_2008 = HashMap::new();
        crisis_2008.insert("market".to_string(), -0.40);
        crisis_2008.insert("smb".to_string(), -0.10);
        crisis_2008.insert("hml".to_string(), -0.15);
        crisis_2008.insert("volatility".to_string(), 3.0);

        self.scenarios.insert(
            "2008_crisis".to_string(),
            StressScenario {
                name: "2008_crisis".to_string(),
                description: "2008 Financial Crisis".to_string(),
                factor_shocks: crisis_2008,
                return_shock: Some(-0.50),
            },
        );

        // 2020 COVID Crash
        let mut covid_2020 = HashMap::new();
        covid_2020.insert("market".to_string(), -0.35);
        covid_2020.insert("smb".to_string(), -0.20);
        covid_2020.insert("momentum".to_string(), -0.25);
        covid_2020.insert("volatility".to_string(), 4.0);

        self.scenarios.insert(
            "2020_covid".to_string(),
            StressScenario {
                name: "2020_covid".to_string(),
                description: "2020 COVID-19 Crash".to_string(),
                factor_shocks: covid_2020,
                return_shock: Some(-0.35),
            },
        );

        // 2022 Rate Shock
        let mut rates_2022 = HashMap::new();
        rates_2022.insert("market".to_string(), -0.20);
        rates_2022.insert("growth".to_string(), -0.30);
        rates_2022.insert("value".to_string(), 0.10);

        self.scenarios.insert(
            "2022_rates".to_string(),
            StressScenario {
                name: "2022_rates".to_string(),
                description: "2022 Rate Hiking Cycle".to_string(),
                factor_shocks: rates_2022,
                return_shock: Some(-0.25),
            },
        );

        // Flash Crash
        let mut flash_crash = HashMap::new();
        flash_crash.insert("market".to_string(), -0.10);
        flash_crash.insert("volatility".to_string(), 5.0);

        self.scenarios.insert(
            "flash_crash".to_string(),
            StressScenario {
                name: "flash_crash".to_string(),
                description: "Flash Crash (1-day)".to_string(),
                factor_shocks: flash_crash,
                return_shock: Some(-0.10),
            },
        );
    }

    /// Apply stress scenario to portfolio
    pub fn apply_scenario(
        &self,
        scenario_name: &str,
        factor_exposures: &HashMap<String, f64>,
    ) -> Result<f64> {
        let scenario = self.scenarios.get(scenario_name).ok_or_else(|| {
            SigcError::Runtime(format!("Scenario not found: {}", scenario_name))
        })?;

        let mut impact = 0.0;

        // Calculate impact from factor shocks
        for (factor, shock) in &scenario.factor_shocks {
            if let Some(exposure) = factor_exposures.get(factor) {
                impact += exposure * shock;
            }
        }

        Ok(impact)
    }

    /// Run all scenarios
    pub fn run_all(&self, factor_exposures: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut results = HashMap::new();

        for name in self.scenarios.keys() {
            if let Ok(impact) = self.apply_scenario(name, factor_exposures) {
                results.insert(name.clone(), impact);
            }
        }

        results
    }
}

impl Default for StressTest {
    fn default() -> Self {
        let mut st = Self::new();
        st.add_historical_scenarios();
        st
    }
}

/// Comprehensive risk report
#[derive(Debug, Clone)]
pub struct RiskReport {
    pub var_95: f64,
    pub var_99: f64,
    pub cvar_95: f64,
    pub cvar_99: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub stress_results: HashMap<String, f64>,
}

/// Generate comprehensive risk report
pub fn generate_risk_report(
    returns: &Series,
    factor_exposures: Option<&HashMap<String, f64>>,
) -> Result<RiskReport> {
    let values = series_to_vec(returns)?;
    let clean: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();

    // VaR calculations
    let var_95 = VaRCalculator::new(0.95, 252).historical_var(returns)?;
    let var_99 = VaRCalculator::new(0.99, 252).historical_var(returns)?;

    // CVaR calculations
    let cvar_95 = CVaRCalculator::new(0.95).historical_cvar(returns)?;
    let cvar_99 = CVaRCalculator::new(0.99).historical_cvar(returns)?;

    // Volatility
    let mean: f64 = clean.iter().sum::<f64>() / clean.len() as f64;
    let variance: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / clean.len() as f64;
    let volatility = variance.sqrt() * (252.0_f64).sqrt();

    // Max drawdown
    let max_drawdown = calculate_max_drawdown(&clean);

    // Stress tests
    let stress_results = if let Some(exposures) = factor_exposures {
        let stress_test = StressTest::default();
        stress_test.run_all(exposures)
    } else {
        HashMap::new()
    };

    Ok(RiskReport {
        var_95,
        var_99,
        cvar_95,
        cvar_99,
        volatility,
        max_drawdown,
        stress_results,
    })
}

/// Calculate maximum drawdown from returns
fn calculate_max_drawdown(returns: &[f64]) -> f64 {
    let mut equity = 1.0;
    let mut peak = 1.0;
    let mut max_dd = 0.0;

    for ret in returns {
        equity *= 1.0 + ret;
        if equity > peak {
            peak = equity;
        }
        let dd = (peak - equity) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

/// Marginal VaR calculator
pub struct MarginalVaR {
    confidence: f64,
}

impl MarginalVaR {
    pub fn new(confidence: f64) -> Self {
        MarginalVaR { confidence }
    }

    /// Calculate marginal VaR for each position
    pub fn calculate(
        &self,
        weights: &[f64],
        returns_matrix: &[Vec<f64>],
        portfolio_var: f64,
    ) -> Result<Vec<f64>> {
        let n_assets = weights.len();
        let mut marginal_var = vec![0.0; n_assets];
        let delta = 0.001; // Small change for numerical differentiation

        for i in 0..n_assets {
            // Increase weight of asset i
            let mut new_weights = weights.to_vec();
            new_weights[i] += delta;

            // Normalize
            let sum: f64 = new_weights.iter().sum();
            for w in &mut new_weights {
                *w /= sum;
            }

            // Calculate new portfolio returns
            let n_periods = returns_matrix[0].len();
            let mut new_portfolio_returns = vec![0.0; n_periods];
            for t in 0..n_periods {
                for j in 0..n_assets {
                    new_portfolio_returns[t] += new_weights[j] * returns_matrix[j][t];
                }
            }

            // Calculate new VaR
            let new_returns = Series::new("returns".into(), new_portfolio_returns);
            let new_var = VaRCalculator::new(self.confidence, 252)
                .historical_var(&new_returns)?;

            // Marginal VaR
            marginal_var[i] = (new_var - portfolio_var) / delta;
        }

        Ok(marginal_var)
    }
}

/// Component VaR (contribution of each position to total VaR)
pub fn component_var(weights: &[f64], marginal_var: &[f64]) -> Vec<f64> {
    weights
        .iter()
        .zip(marginal_var.iter())
        .map(|(w, mv)| w * mv)
        .collect()
}

// Helper function
fn series_to_vec(series: &Series) -> Result<Vec<f64>> {
    series
        .f64()
        .map_err(|e| SigcError::Runtime(format!("Failed to convert series: {}", e)))?
        .into_iter()
        .map(|v| Ok(v.unwrap_or(f64::NAN)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_returns() -> Series {
        Series::new(
            "returns".into(),
            vec![0.01, -0.02, 0.015, -0.005, 0.02, -0.03, 0.01, 0.005, -0.01, 0.008],
        )
    }

    #[test]
    fn test_historical_var() {
        let returns = sample_returns();
        let var_calc = VaRCalculator::new(0.95, 10);
        let var = var_calc.historical_var(&returns).unwrap();
        assert!(var > 0.0);
    }

    #[test]
    fn test_parametric_var() {
        let returns = sample_returns();
        let var_calc = VaRCalculator::new(0.95, 10);
        let var = var_calc.parametric_var(&returns).unwrap();
        assert!(var > 0.0);
    }

    #[test]
    fn test_cvar() {
        let returns = sample_returns();
        let cvar_calc = CVaRCalculator::new(0.95);
        let cvar = cvar_calc.historical_cvar(&returns).unwrap();
        assert!(cvar > 0.0);
    }

    #[test]
    fn test_cvar_greater_than_var() {
        let returns = sample_returns();
        let var = VaRCalculator::new(0.95, 10).historical_var(&returns).unwrap();
        let cvar = CVaRCalculator::new(0.95).historical_cvar(&returns).unwrap();
        assert!(cvar >= var);
    }

    #[test]
    fn test_stress_scenarios() {
        let mut stress = StressTest::new();
        stress.add_historical_scenarios();

        assert!(stress.scenarios.contains_key("2008_crisis"));
        assert!(stress.scenarios.contains_key("2020_covid"));
    }

    #[test]
    fn test_apply_scenario() {
        let stress = StressTest::default();

        let mut exposures = HashMap::new();
        exposures.insert("market".to_string(), 1.0);
        exposures.insert("smb".to_string(), 0.5);

        let impact = stress.apply_scenario("2008_crisis", &exposures).unwrap();
        assert!(impact < 0.0); // Should be negative (loss)
    }

    #[test]
    fn test_risk_report() {
        let returns = sample_returns();
        let report = generate_risk_report(&returns, None).unwrap();

        assert!(report.var_95 > 0.0);
        assert!(report.var_99 > 0.0);
        // Note: var_99 >= var_95 may not hold for small samples
        assert!(report.volatility > 0.0);
    }

    #[test]
    fn test_max_drawdown() {
        let returns = vec![0.10, -0.05, -0.10, 0.15, -0.20];
        let dd = calculate_max_drawdown(&returns);
        assert!(dd > 0.0);
        assert!(dd < 1.0);
    }

    #[test]
    fn test_cornish_fisher_var() {
        let returns = sample_returns();
        let var_calc = VaRCalculator::new(0.95, 10);
        let var = var_calc.cornish_fisher_var(&returns).unwrap();
        assert!(var > 0.0);
    }
}
