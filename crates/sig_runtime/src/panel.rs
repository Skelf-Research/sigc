//! Panel data structure for time x assets operations
//!
//! Supports both time-series (per asset) and cross-sectional (across assets) operations.

use polars::prelude::*;
use rayon::prelude::*;
use sig_types::{Result, SigcError};

/// Panel data: rows = time, columns = assets
#[derive(Debug, Clone)]
pub struct Panel {
    /// DataFrame where each column is an asset's time series
    pub data: DataFrame,
    /// Column names (asset identifiers)
    pub assets: Vec<String>,
    /// Number of time periods
    pub n_periods: usize,
}

impl Panel {
    /// Create a panel from a DataFrame
    pub fn new(data: DataFrame) -> Result<Self> {
        let assets: Vec<String> = data.get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let n_periods = data.height();

        Ok(Panel {
            data,
            assets,
            n_periods,
        })
    }

    /// Create a panel from vectors (assets x time)
    pub fn from_vecs(asset_names: Vec<String>, values: Vec<Vec<f64>>) -> Result<Self> {
        let columns: Vec<Column> = asset_names.iter()
            .zip(values.iter())
            .map(|(name, vals)| Column::new(name.clone().into(), vals.clone()))
            .collect();

        let data = DataFrame::new(columns)
            .map_err(|e| SigcError::Runtime(format!("Failed to create panel: {}", e)))?;

        Self::new(data)
    }

    /// Number of assets
    pub fn n_assets(&self) -> usize {
        self.assets.len()
    }

    /// Apply a time-series operation to each asset independently
    pub fn apply_ts<F>(&self, f: F) -> Result<Panel>
    where
        F: Fn(&Series) -> Result<Series> + Sync,
    {
        let results: Vec<Result<Column>> = self.assets
            .par_iter()
            .map(|asset| {
                let col = self.data.column(asset)
                    .map_err(|e| SigcError::Runtime(format!("Column not found: {}", e)))?;
                let series = col.as_series().unwrap();
                let result = f(series)?;
                Ok(result.into_column())
            })
            .collect();

        let new_columns: Vec<Column> = results.into_iter().collect::<Result<_>>()?;

        let new_data = DataFrame::new(new_columns)
            .map_err(|e| SigcError::Runtime(format!("Failed to create result: {}", e)))?;

        Panel::new(new_data)
    }

    /// Apply a cross-sectional operation across assets at each time point
    pub fn apply_xs<F>(&self, f: F) -> Result<Panel>
    where
        F: Fn(&[f64]) -> Vec<f64> + Sync,
    {
        let n_assets = self.n_assets();
        let n_periods = self.n_periods;

        // Extract all values
        let mut asset_values: Vec<Vec<f64>> = Vec::new();
        for asset in &self.assets {
            let col = self.data.column(asset)
                .map_err(|e| SigcError::Runtime(format!("Column not found: {}", e)))?;
            let values: Vec<f64> = col.f64()
                .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
                .into_iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect();
            asset_values.push(values);
        }

        // Apply cross-sectional function at each time point in parallel
        let time_results: Vec<Vec<f64>> = (0..n_periods)
            .into_par_iter()
            .map(|t| {
                let xs: Vec<f64> = asset_values.iter().map(|v| v[t]).collect();
                f(&xs)
            })
            .collect();

        // Transpose results: time_results[t][asset] -> result_values[asset][t]
        let mut result_values: Vec<Vec<f64>> = vec![vec![0.0; n_periods]; n_assets];
        for (t, row) in time_results.iter().enumerate() {
            for (i, &val) in row.iter().enumerate() {
                result_values[i][t] = val;
            }
        }

        Panel::from_vecs(self.assets.clone(), result_values)
    }

    /// Cross-sectional z-score at each time point
    pub fn xs_zscore(&self) -> Result<Panel> {
        self.apply_xs(|xs| {
            let n = xs.len() as f64;
            let mean = xs.iter().sum::<f64>() / n;
            let variance = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
            let std = variance.sqrt();

            if std > 1e-10 {
                xs.iter().map(|x| (x - mean) / std).collect()
            } else {
                vec![0.0; xs.len()]
            }
        })
    }

    /// Cross-sectional rank at each time point (0 to 1)
    pub fn xs_rank(&self) -> Result<Panel> {
        self.apply_xs(|xs| {
            let n = xs.len();
            let mut indexed: Vec<(usize, f64)> = xs.iter().enumerate().map(|(i, &v)| (i, v)).collect();
            indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            let mut ranks = vec![0.0; n];
            for (rank, (idx, _)) in indexed.iter().enumerate() {
                ranks[*idx] = (rank + 1) as f64 / n as f64;
            }
            ranks
        })
    }

    /// Cross-sectional demean at each time point
    pub fn xs_demean(&self) -> Result<Panel> {
        self.apply_xs(|xs| {
            let mean = xs.iter().sum::<f64>() / xs.len() as f64;
            xs.iter().map(|x| x - mean).collect()
        })
    }

    /// Cross-sectional scale (sum to 1) at each time point
    pub fn xs_scale(&self) -> Result<Panel> {
        self.apply_xs(|xs| {
            let sum: f64 = xs.iter().map(|x| x.abs()).sum();
            if sum > 1e-10 {
                xs.iter().map(|x| x / sum).collect()
            } else {
                vec![0.0; xs.len()]
            }
        })
    }

    /// Get a single asset's time series
    pub fn get_asset(&self, name: &str) -> Result<Series> {
        self.data.column(name)
            .map(|c| c.as_series().unwrap().clone())
            .map_err(|e| SigcError::Runtime(format!("Asset not found: {}", e)))
    }

    /// Get cross-section at a specific time index
    pub fn get_xs(&self, t: usize) -> Result<Vec<f64>> {
        if t >= self.n_periods {
            return Err(SigcError::Runtime(format!("Time index {} out of bounds", t)));
        }

        let mut xs = Vec::new();
        for asset in &self.assets {
            let col = self.data.column(asset)
                .map_err(|e| SigcError::Runtime(format!("Column not found: {}", e)))?;
            let val = col.get(t)
                .map_err(|e| SigcError::Runtime(format!("Get failed: {}", e)))?;
            let f = match val {
                AnyValue::Float64(v) => v,
                AnyValue::Float32(v) => v as f64,
                AnyValue::Int64(v) => v as f64,
                AnyValue::Int32(v) => v as f64,
                _ => f64::NAN,
            };
            xs.push(f);
        }
        Ok(xs)
    }

    /// Convert to DataFrame
    pub fn to_dataframe(&self) -> DataFrame {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel() -> Panel {
        let assets = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let values = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],  // Asset A
            vec![5.0, 4.0, 3.0, 2.0, 1.0],  // Asset B
            vec![3.0, 3.0, 3.0, 3.0, 3.0],  // Asset C
        ];
        Panel::from_vecs(assets, values).unwrap()
    }

    #[test]
    fn test_panel_creation() {
        let panel = sample_panel();
        assert_eq!(panel.n_assets(), 3);
        assert_eq!(panel.n_periods, 5);
    }

    #[test]
    fn test_xs_zscore() {
        let panel = sample_panel();
        let result = panel.xs_zscore().unwrap();

        // At t=0: values are [1, 5, 3], mean=3, std≈1.63
        let xs = result.get_xs(0).unwrap();
        assert!(xs[0] < 0.0); // 1 is below mean
        assert!(xs[1] > 0.0); // 5 is above mean
        assert!(xs[2].abs() < 0.1); // 3 is at mean
    }

    #[test]
    fn test_xs_rank() {
        let panel = sample_panel();
        let result = panel.xs_rank().unwrap();

        // At t=0: values are [1, 5, 3], ranks should be [0.33, 1.0, 0.67]
        let xs = result.get_xs(0).unwrap();
        assert!((xs[0] - 1.0/3.0).abs() < 0.01);
        assert!((xs[1] - 1.0).abs() < 0.01);
        assert!((xs[2] - 2.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_xs_demean() {
        let panel = sample_panel();
        let result = panel.xs_demean().unwrap();

        // At each time point, mean should be ~0
        for t in 0..5 {
            let xs = result.get_xs(t).unwrap();
            let mean: f64 = xs.iter().sum::<f64>() / xs.len() as f64;
            assert!(mean.abs() < 1e-10);
        }
    }
}
