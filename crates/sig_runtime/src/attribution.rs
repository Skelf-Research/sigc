//! Returns attribution analysis
//!
//! Decomposes portfolio returns into factor and sector contributions.

use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// Factor definition for attribution
#[derive(Debug, Clone)]
pub struct Factor {
    /// Factor name (e.g., "momentum", "value")
    pub name: String,
    /// Factor returns per period
    pub returns: Vec<f64>,
}

/// Sector mapping for assets
#[derive(Debug, Clone)]
pub struct SectorMapping {
    /// Map of asset name to sector name
    pub asset_sectors: HashMap<String, String>,
}

impl SectorMapping {
    /// Create a new sector mapping
    pub fn new() -> Self {
        SectorMapping {
            asset_sectors: HashMap::new(),
        }
    }

    /// Add an asset-sector mapping
    pub fn add(&mut self, asset: &str, sector: &str) -> &mut Self {
        self.asset_sectors.insert(asset.to_string(), sector.to_string());
        self
    }

    /// Get sector for an asset
    pub fn sector(&self, asset: &str) -> Option<&String> {
        self.asset_sectors.get(asset)
    }
}

impl Default for SectorMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribution analysis results
#[derive(Debug, Clone)]
pub struct AttributionResult {
    /// Factor contributions (factor name -> contribution)
    pub factor_contributions: HashMap<String, f64>,
    /// Sector contributions (sector name -> contribution)
    pub sector_contributions: HashMap<String, f64>,
    /// Factor exposures over time (factor name -> exposure series)
    pub factor_exposures: HashMap<String, Vec<f64>>,
    /// Residual alpha (unexplained return)
    pub alpha: f64,
    /// Total return explained
    pub total_explained: f64,
    /// R-squared (fraction of variance explained)
    pub r_squared: f64,
}

/// Attribution analyzer
pub struct AttributionAnalyzer {
    factors: Vec<Factor>,
    sectors: Option<SectorMapping>,
}

impl AttributionAnalyzer {
    /// Create a new attribution analyzer
    pub fn new() -> Self {
        AttributionAnalyzer {
            factors: Vec::new(),
            sectors: None,
        }
    }

    /// Add a factor for attribution
    pub fn add_factor(&mut self, name: &str, returns: Vec<f64>) -> &mut Self {
        self.factors.push(Factor {
            name: name.to_string(),
            returns,
        });
        self
    }

    /// Set sector mapping
    pub fn with_sectors(&mut self, mapping: SectorMapping) -> &mut Self {
        self.sectors = Some(mapping);
        self
    }

    /// Run attribution analysis
    ///
    /// Uses OLS regression to decompose returns:
    /// portfolio_return = sum(beta_i * factor_i) + alpha
    pub fn analyze(
        &self,
        portfolio_returns: &[f64],
        asset_weights: Option<&HashMap<String, Vec<f64>>>,
        asset_returns: Option<&HashMap<String, Vec<f64>>>,
    ) -> Result<AttributionResult> {
        if portfolio_returns.is_empty() {
            return Ok(AttributionResult {
                factor_contributions: HashMap::new(),
                sector_contributions: HashMap::new(),
                factor_exposures: HashMap::new(),
                alpha: 0.0,
                total_explained: 0.0,
                r_squared: 0.0,
            });
        }

        let n = portfolio_returns.len();

        // Calculate factor contributions via regression
        let (factor_contributions, factor_exposures, alpha, r_squared) =
            self.calculate_factor_attribution(portfolio_returns)?;

        // Calculate sector contributions if we have asset-level data
        let sector_contributions = if let (Some(weights), Some(returns), Some(sectors)) =
            (asset_weights, asset_returns, &self.sectors)
        {
            self.calculate_sector_attribution(weights, returns, sectors, n)
        } else {
            HashMap::new()
        };

        let total_explained: f64 = factor_contributions.values().sum();

        Ok(AttributionResult {
            factor_contributions,
            sector_contributions,
            factor_exposures,
            alpha,
            total_explained,
            r_squared,
        })
    }

    /// Calculate factor contributions using regression
    fn calculate_factor_attribution(
        &self,
        portfolio_returns: &[f64],
    ) -> Result<(HashMap<String, f64>, HashMap<String, Vec<f64>>, f64, f64)> {
        let n = portfolio_returns.len();
        let k = self.factors.len();

        if k == 0 {
            // No factors - all return is alpha
            let alpha = portfolio_returns.iter().sum::<f64>() / n as f64 * 252.0;
            return Ok((HashMap::new(), HashMap::new(), alpha, 0.0));
        }

        // Simple univariate regression for each factor
        // For production, use multivariate regression
        let mut contributions = HashMap::new();
        let mut exposures = HashMap::new();
        let mut total_variance_explained = 0.0;

        let port_mean: f64 = portfolio_returns.iter().sum::<f64>() / n as f64;
        let port_var: f64 = portfolio_returns
            .iter()
            .map(|r| (r - port_mean).powi(2))
            .sum::<f64>()
            / n as f64;

        for factor in &self.factors {
            if factor.returns.len() != n {
                continue;
            }

            let factor_mean: f64 = factor.returns.iter().sum::<f64>() / n as f64;
            let factor_var: f64 = factor.returns
                .iter()
                .map(|r| (r - factor_mean).powi(2))
                .sum::<f64>()
                / n as f64;

            // Covariance
            let covar: f64 = portfolio_returns
                .iter()
                .zip(factor.returns.iter())
                .map(|(p, f)| (p - port_mean) * (f - factor_mean))
                .sum::<f64>()
                / n as f64;

            // Beta = Cov / Var
            let beta = if factor_var > 1e-10 {
                covar / factor_var
            } else {
                0.0
            };

            // Contribution = Beta * Factor mean return * annualization
            let contribution = beta * factor_mean * 252.0;
            contributions.insert(factor.name.clone(), contribution);

            // Store exposures (beta over time - simplified as constant here)
            exposures.insert(factor.name.clone(), vec![beta; n]);

            // Variance explained
            if port_var > 1e-10 {
                total_variance_explained += (covar * covar) / (port_var * factor_var);
            }
        }

        // Alpha = Portfolio return - Factor contributions
        let total_factor_contribution: f64 = contributions.values().sum();
        let alpha = port_mean * 252.0 - total_factor_contribution;

        // R-squared (simplified)
        let r_squared = total_variance_explained.min(1.0);

        Ok((contributions, exposures, alpha, r_squared))
    }

    /// Calculate sector contributions
    fn calculate_sector_attribution(
        &self,
        weights: &HashMap<String, Vec<f64>>,
        returns: &HashMap<String, Vec<f64>>,
        sectors: &SectorMapping,
        n_periods: usize,
    ) -> HashMap<String, f64> {
        let mut sector_returns: HashMap<String, f64> = HashMap::new();

        for (asset, weight_series) in weights {
            if let (Some(return_series), Some(sector)) = (returns.get(asset), sectors.sector(asset)) {
                // Sum weighted returns for this asset
                let contribution: f64 = weight_series
                    .iter()
                    .zip(return_series.iter())
                    .map(|(w, r)| w * r)
                    .sum();

                *sector_returns.entry(sector.clone()).or_insert(0.0) += contribution;
            }
        }

        // Annualize sector contributions
        for contribution in sector_returns.values_mut() {
            *contribution = *contribution / n_periods as f64 * 252.0;
        }

        sector_returns
    }

    /// Calculate Brinson attribution (allocation + selection)
    pub fn brinson_attribution(
        &self,
        portfolio_weights: &HashMap<String, f64>,
        benchmark_weights: &HashMap<String, f64>,
        portfolio_returns: &HashMap<String, f64>,
        benchmark_returns: &HashMap<String, f64>,
        sectors: &SectorMapping,
    ) -> BrinsonResult {
        let mut allocation = HashMap::new();
        let mut selection = HashMap::new();
        let mut interaction = HashMap::new();

        // Group by sector
        let mut port_sector_weights: HashMap<String, f64> = HashMap::new();
        let mut bench_sector_weights: HashMap<String, f64> = HashMap::new();
        let mut port_sector_returns: HashMap<String, (f64, f64)> = HashMap::new(); // (weighted_return, weight)
        let mut bench_sector_returns: HashMap<String, (f64, f64)> = HashMap::new();

        // Aggregate portfolio by sector
        for (asset, weight) in portfolio_weights {
            if let Some(sector) = sectors.sector(asset) {
                *port_sector_weights.entry(sector.clone()).or_insert(0.0) += weight;
                if let Some(ret) = portfolio_returns.get(asset) {
                    let entry = port_sector_returns.entry(sector.clone()).or_insert((0.0, 0.0));
                    entry.0 += weight * ret;
                    entry.1 += weight;
                }
            }
        }

        // Aggregate benchmark by sector
        for (asset, weight) in benchmark_weights {
            if let Some(sector) = sectors.sector(asset) {
                *bench_sector_weights.entry(sector.clone()).or_insert(0.0) += weight;
                if let Some(ret) = benchmark_returns.get(asset) {
                    let entry = bench_sector_returns.entry(sector.clone()).or_insert((0.0, 0.0));
                    entry.0 += weight * ret;
                    entry.1 += weight;
                }
            }
        }

        // Total benchmark return
        let total_bench_return: f64 = bench_sector_returns
            .values()
            .map(|(wr, _)| wr)
            .sum();

        // Calculate Brinson components for each sector
        let all_sectors: std::collections::HashSet<_> = port_sector_weights
            .keys()
            .chain(bench_sector_weights.keys())
            .cloned()
            .collect();

        for sector in all_sectors {
            let port_weight = *port_sector_weights.get(&sector).unwrap_or(&0.0);
            let bench_weight = *bench_sector_weights.get(&sector).unwrap_or(&0.0);

            let port_ret = port_sector_returns
                .get(&sector)
                .map(|(wr, w)| if *w > 1e-10 { wr / w } else { 0.0 })
                .unwrap_or(0.0);
            let bench_ret = bench_sector_returns
                .get(&sector)
                .map(|(wr, w)| if *w > 1e-10 { wr / w } else { 0.0 })
                .unwrap_or(0.0);

            // Allocation = (Port Weight - Bench Weight) * (Sector Bench Return - Total Bench Return)
            let alloc = (port_weight - bench_weight) * (bench_ret - total_bench_return);
            allocation.insert(sector.clone(), alloc);

            // Selection = Bench Weight * (Port Sector Return - Bench Sector Return)
            let sel = bench_weight * (port_ret - bench_ret);
            selection.insert(sector.clone(), sel);

            // Interaction = (Port Weight - Bench Weight) * (Port Return - Bench Return)
            let inter = (port_weight - bench_weight) * (port_ret - bench_ret);
            interaction.insert(sector.clone(), inter);
        }

        let total_allocation: f64 = allocation.values().sum();
        let total_selection: f64 = selection.values().sum();
        let total_interaction: f64 = interaction.values().sum();

        BrinsonResult {
            allocation,
            selection,
            interaction,
            total_allocation,
            total_selection,
            total_interaction,
        }
    }
}

impl Default for AttributionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Brinson attribution result
#[derive(Debug, Clone)]
pub struct BrinsonResult {
    /// Allocation effect by sector
    pub allocation: HashMap<String, f64>,
    /// Selection effect by sector
    pub selection: HashMap<String, f64>,
    /// Interaction effect by sector
    pub interaction: HashMap<String, f64>,
    /// Total allocation effect
    pub total_allocation: f64,
    /// Total selection effect
    pub total_selection: f64,
    /// Total interaction effect
    pub total_interaction: f64,
}

impl BrinsonResult {
    /// Total active return
    pub fn total_active_return(&self) -> f64 {
        self.total_allocation + self.total_selection + self.total_interaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factor_attribution() {
        let mut analyzer = AttributionAnalyzer::new();

        // Create simple factor
        let momentum = vec![0.01, -0.005, 0.015, 0.008, -0.003];
        analyzer.add_factor("momentum", momentum);

        // Portfolio returns correlated with momentum
        let portfolio = vec![0.012, -0.004, 0.018, 0.01, -0.002];

        let result = analyzer.analyze(&portfolio, None, None).unwrap();

        assert!(result.factor_contributions.contains_key("momentum"));
        assert!(result.r_squared >= 0.0 && result.r_squared <= 1.0);
    }

    #[test]
    fn test_no_factors() {
        let analyzer = AttributionAnalyzer::new();
        let portfolio = vec![0.01, 0.02, -0.01];

        let result = analyzer.analyze(&portfolio, None, None).unwrap();

        // All return is alpha when no factors
        assert!(result.alpha.abs() > 0.0);
        assert_eq!(result.r_squared, 0.0);
    }

    #[test]
    fn test_sector_mapping() {
        let mut mapping = SectorMapping::new();
        mapping.add("AAPL", "Tech").add("MSFT", "Tech").add("JPM", "Finance");

        assert_eq!(mapping.sector("AAPL"), Some(&"Tech".to_string()));
        assert_eq!(mapping.sector("JPM"), Some(&"Finance".to_string()));
        assert_eq!(mapping.sector("UNKNOWN"), None);
    }

    #[test]
    fn test_brinson_attribution() {
        let analyzer = AttributionAnalyzer::new();

        let mut sectors = SectorMapping::new();
        sectors.add("AAPL", "Tech").add("JPM", "Finance");

        let mut port_weights = HashMap::new();
        port_weights.insert("AAPL".to_string(), 0.6);
        port_weights.insert("JPM".to_string(), 0.4);

        let mut bench_weights = HashMap::new();
        bench_weights.insert("AAPL".to_string(), 0.5);
        bench_weights.insert("JPM".to_string(), 0.5);

        let mut port_returns = HashMap::new();
        port_returns.insert("AAPL".to_string(), 0.10);
        port_returns.insert("JPM".to_string(), 0.05);

        let mut bench_returns = HashMap::new();
        bench_returns.insert("AAPL".to_string(), 0.08);
        bench_returns.insert("JPM".to_string(), 0.06);

        let result = analyzer.brinson_attribution(
            &port_weights,
            &bench_weights,
            &port_returns,
            &bench_returns,
            &sectors,
        );

        // Total active return should be portfolio - benchmark
        // Port: 0.6*0.10 + 0.4*0.05 = 0.08
        // Bench: 0.5*0.08 + 0.5*0.06 = 0.07
        // Active: 0.01
        let active = result.total_active_return();
        assert!((active - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_empty_portfolio() {
        let analyzer = AttributionAnalyzer::new();
        let result = analyzer.analyze(&[], None, None).unwrap();

        assert_eq!(result.alpha, 0.0);
        assert_eq!(result.r_squared, 0.0);
    }
}
