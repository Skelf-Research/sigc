//! Regime detection for market state identification
//!
//! Provides HMM, clustering, and rule-based regime detection.

use polars::prelude::*;
use sig_types::{Result, SigcError};

/// Market regime types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketRegime {
    /// Low volatility, trending up
    BullQuiet,
    /// High volatility, trending up
    BullVolatile,
    /// Low volatility, trending down
    BearQuiet,
    /// High volatility, trending down
    BearVolatile,
    /// Sideways/ranging market
    Neutral,
}

impl std::fmt::Display for MarketRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketRegime::BullQuiet => write!(f, "Bull (Quiet)"),
            MarketRegime::BullVolatile => write!(f, "Bull (Volatile)"),
            MarketRegime::BearQuiet => write!(f, "Bear (Quiet)"),
            MarketRegime::BearVolatile => write!(f, "Bear (Volatile)"),
            MarketRegime::Neutral => write!(f, "Neutral"),
        }
    }
}

/// Simple Hidden Markov Model for regime detection
pub struct HiddenMarkovModel {
    /// Number of states
    pub n_states: usize,
    /// Transition probability matrix
    pub transition: Vec<Vec<f64>>,
    /// Emission means for each state
    pub emission_means: Vec<f64>,
    /// Emission standard deviations for each state
    pub emission_stds: Vec<f64>,
    /// Initial state probabilities
    pub initial: Vec<f64>,
}

impl HiddenMarkovModel {
    /// Create a new HMM with specified number of states
    pub fn new(n_states: usize) -> Self {
        let uniform = 1.0 / n_states as f64;
        HiddenMarkovModel {
            n_states,
            transition: vec![vec![uniform; n_states]; n_states],
            emission_means: vec![0.0; n_states],
            emission_stds: vec![1.0; n_states],
            initial: vec![uniform; n_states],
        }
    }

    /// Create a 2-state bull/bear model
    pub fn bull_bear() -> Self {
        let mut hmm = Self::new(2);
        // Bull state (0): positive mean, lower vol
        hmm.emission_means[0] = 0.0005; // ~12% annual
        hmm.emission_stds[0] = 0.008;   // ~12% annual vol
        // Bear state (1): negative mean, higher vol
        hmm.emission_means[1] = -0.001; // -25% annual
        hmm.emission_stds[1] = 0.020;   // ~32% annual vol

        // Transition: tend to stay in current state
        hmm.transition = vec![
            vec![0.98, 0.02], // Bull -> Bull, Bull -> Bear
            vec![0.05, 0.95], // Bear -> Bull, Bear -> Bear
        ];
        hmm.initial = vec![0.8, 0.2];
        hmm
    }

    /// Create a 3-state model (bull, neutral, bear)
    pub fn three_state() -> Self {
        let mut hmm = Self::new(3);
        hmm.emission_means = vec![0.0008, 0.0, -0.0008];
        hmm.emission_stds = vec![0.008, 0.012, 0.020];
        hmm.transition = vec![
            vec![0.90, 0.08, 0.02],
            vec![0.10, 0.80, 0.10],
            vec![0.05, 0.10, 0.85],
        ];
        hmm.initial = vec![0.4, 0.4, 0.2];
        hmm
    }

    /// Fit model to data using EM algorithm (simplified)
    pub fn fit(&mut self, returns: &Series, max_iter: usize) -> Result<()> {
        let values = series_to_vec(returns)?;
        let clean: Vec<f64> = values.iter().filter(|v| !v.is_nan()).copied().collect();

        if clean.len() < 10 {
            return Err(SigcError::Runtime("Not enough data for HMM fitting".into()));
        }

        // Simple initialization based on quantiles
        let mut sorted = clean.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len();

        for state in 0..self.n_states {
            let start = state * n / self.n_states;
            let end = (state + 1) * n / self.n_states;
            let segment = &sorted[start..end];

            self.emission_means[state] = segment.iter().sum::<f64>() / segment.len() as f64;
            let var: f64 = segment
                .iter()
                .map(|x| (x - self.emission_means[state]).powi(2))
                .sum::<f64>()
                / segment.len() as f64;
            self.emission_stds[state] = var.sqrt().max(0.001);
        }

        // Sort means (lowest to highest for bear to bull)
        let mut state_order: Vec<usize> = (0..self.n_states).collect();
        state_order.sort_by(|&a, &b| {
            self.emission_means[a]
                .partial_cmp(&self.emission_means[b])
                .unwrap()
        });

        // Simplified EM iterations (forward-backward would be full implementation)
        for _ in 0..max_iter {
            // E-step: compute state probabilities
            let probs = self.state_probabilities(&clean)?;

            // M-step: update parameters
            for state in 0..self.n_states {
                let mut sum_prob = 0.0;
                let mut sum_val = 0.0;
                let mut sum_sq = 0.0;

                for (t, &val) in clean.iter().enumerate() {
                    let p = probs[t][state];
                    sum_prob += p;
                    sum_val += p * val;
                }

                if sum_prob > 0.0 {
                    self.emission_means[state] = sum_val / sum_prob;

                    for (t, &val) in clean.iter().enumerate() {
                        let p = probs[t][state];
                        sum_sq += p * (val - self.emission_means[state]).powi(2);
                    }
                    self.emission_stds[state] = (sum_sq / sum_prob).sqrt().max(0.001);
                }
            }
        }

        Ok(())
    }

    /// Calculate state probabilities for each time point
    fn state_probabilities(&self, data: &[f64]) -> Result<Vec<Vec<f64>>> {
        let n = data.len();
        let mut probs = vec![vec![0.0; self.n_states]; n];

        for (t, &val) in data.iter().enumerate() {
            let mut total = 0.0;
            for state in 0..self.n_states {
                let prob = gaussian_pdf(val, self.emission_means[state], self.emission_stds[state]);
                probs[t][state] = prob * if t == 0 {
                    self.initial[state]
                } else {
                    // Sum of transition probs from previous states
                    (0..self.n_states)
                        .map(|prev| probs[t - 1][prev] * self.transition[prev][state])
                        .sum()
                };
                total += probs[t][state];
            }
            // Normalize
            if total > 0.0 {
                for state in 0..self.n_states {
                    probs[t][state] /= total;
                }
            }
        }

        Ok(probs)
    }

    /// Predict most likely state sequence (Viterbi)
    pub fn predict(&self, returns: &Series) -> Result<Vec<usize>> {
        let values = series_to_vec(returns)?;
        let probs = self.state_probabilities(&values)?;

        let states: Vec<usize> = probs
            .iter()
            .map(|p| {
                p.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            })
            .collect();

        Ok(states)
    }

    /// Get state probabilities as DataFrame
    pub fn state_probabilities_df(&self, returns: &Series) -> Result<DataFrame> {
        let values = series_to_vec(returns)?;
        let probs = self.state_probabilities(&values)?;

        let mut columns = Vec::new();
        for state in 0..self.n_states {
            let col: Vec<f64> = probs.iter().map(|p| p[state]).collect();
            columns.push(Column::new(format!("state_{}", state).into(), col));
        }

        DataFrame::new(columns)
            .map_err(|e| SigcError::Runtime(format!("Failed to create DataFrame: {}", e)))
    }
}

/// Volatility regime detector
pub struct VolatilityRegime {
    /// Short-term volatility window
    pub short_window: usize,
    /// Long-term volatility window
    pub long_window: usize,
    /// High volatility threshold (multiple of long-term)
    pub high_threshold: f64,
    /// Low volatility threshold
    pub low_threshold: f64,
}

impl VolatilityRegime {
    pub fn new(short_window: usize, long_window: usize) -> Self {
        VolatilityRegime {
            short_window,
            long_window,
            high_threshold: 1.5,
            low_threshold: 0.7,
        }
    }

    /// Detect volatility regime
    pub fn detect(&self, returns: &Series) -> Result<Series> {
        let values = series_to_vec(returns)?;
        let n = values.len();

        let mut regimes = vec![1.0; n]; // 0=low, 1=normal, 2=high

        for i in self.long_window..n {
            // Short-term volatility
            let short_slice = &values[(i + 1 - self.short_window)..=i];
            let short_mean: f64 = short_slice.iter().sum::<f64>() / self.short_window as f64;
            let short_var: f64 = short_slice
                .iter()
                .map(|x| (x - short_mean).powi(2))
                .sum::<f64>()
                / self.short_window as f64;
            let short_vol = short_var.sqrt();

            // Long-term volatility
            let long_slice = &values[(i + 1 - self.long_window)..=i];
            let long_mean: f64 = long_slice.iter().sum::<f64>() / self.long_window as f64;
            let long_var: f64 = long_slice
                .iter()
                .map(|x| (x - long_mean).powi(2))
                .sum::<f64>()
                / self.long_window as f64;
            let long_vol = long_var.sqrt();

            // Regime classification
            if long_vol > 0.0 {
                let ratio = short_vol / long_vol;
                if ratio > self.high_threshold {
                    regimes[i] = 2.0; // High volatility
                } else if ratio < self.low_threshold {
                    regimes[i] = 0.0; // Low volatility
                } else {
                    regimes[i] = 1.0; // Normal
                }
            }
        }

        Ok(Series::new("vol_regime".into(), regimes))
    }

    /// Get regime labels
    pub fn detect_labeled(&self, returns: &Series) -> Result<Vec<String>> {
        let regimes = self.detect(returns)?;
        let values = series_to_vec(&regimes)?;

        let labels: Vec<String> = values
            .iter()
            .map(|&v| match v as i32 {
                0 => "Low Vol".to_string(),
                2 => "High Vol".to_string(),
                _ => "Normal".to_string(),
            })
            .collect();

        Ok(labels)
    }
}

/// Trend regime detector
pub struct TrendRegime {
    /// Fast moving average window
    pub fast_window: usize,
    /// Slow moving average window
    pub slow_window: usize,
}

impl TrendRegime {
    pub fn new(fast_window: usize, slow_window: usize) -> Self {
        TrendRegime {
            fast_window,
            slow_window,
        }
    }

    /// Detect trend regime
    pub fn detect(&self, prices: &Series) -> Result<Series> {
        let values = series_to_vec(prices)?;
        let n = values.len();

        let mut regimes = vec![0.0; n]; // -1=downtrend, 0=neutral, 1=uptrend

        for i in self.slow_window..n {
            // Fast MA
            let fast_slice = &values[(i + 1 - self.fast_window)..=i];
            let fast_ma: f64 = fast_slice.iter().sum::<f64>() / self.fast_window as f64;

            // Slow MA
            let slow_slice = &values[(i + 1 - self.slow_window)..=i];
            let slow_ma: f64 = slow_slice.iter().sum::<f64>() / self.slow_window as f64;

            // Trend classification
            let diff = (fast_ma - slow_ma) / slow_ma;
            if diff > 0.02 {
                regimes[i] = 1.0; // Uptrend
            } else if diff < -0.02 {
                regimes[i] = -1.0; // Downtrend
            } else {
                regimes[i] = 0.0; // Neutral
            }
        }

        Ok(Series::new("trend_regime".into(), regimes))
    }
}

/// Combined regime detector
pub struct RegimeDetector {
    pub vol_detector: VolatilityRegime,
    pub trend_detector: TrendRegime,
}

impl RegimeDetector {
    pub fn new() -> Self {
        RegimeDetector {
            vol_detector: VolatilityRegime::new(20, 60),
            trend_detector: TrendRegime::new(20, 60),
        }
    }

    /// Detect combined regime
    pub fn detect(&self, prices: &Series, returns: &Series) -> Result<Vec<MarketRegime>> {
        let vol_regimes = self.vol_detector.detect(returns)?;
        let trend_regimes = self.trend_detector.detect(prices)?;

        let vol_values = series_to_vec(&vol_regimes)?;
        let trend_values = series_to_vec(&trend_regimes)?;

        let n = vol_values.len().min(trend_values.len());
        let mut regimes = Vec::with_capacity(n);

        for i in 0..n {
            let vol = vol_values[i] as i32;
            let trend = trend_values[i] as i32;

            let regime = match (trend, vol) {
                (1, 0) | (1, 1) => MarketRegime::BullQuiet,
                (1, 2) => MarketRegime::BullVolatile,
                (-1, 0) | (-1, 1) => MarketRegime::BearQuiet,
                (-1, 2) => MarketRegime::BearVolatile,
                _ => MarketRegime::Neutral,
            };

            regimes.push(regime);
        }

        Ok(regimes)
    }

    /// Get regime as numeric series
    pub fn detect_numeric(&self, prices: &Series, returns: &Series) -> Result<Series> {
        let regimes = self.detect(prices, returns)?;
        let values: Vec<f64> = regimes
            .iter()
            .map(|r| match r {
                MarketRegime::BullQuiet => 2.0,
                MarketRegime::BullVolatile => 1.0,
                MarketRegime::Neutral => 0.0,
                MarketRegime::BearQuiet => -1.0,
                MarketRegime::BearVolatile => -2.0,
            })
            .collect();

        Ok(Series::new("regime".into(), values))
    }
}

impl Default for RegimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// K-means clustering for regime detection
pub struct KMeansRegime {
    /// Number of clusters
    pub k: usize,
    /// Cluster centers
    pub centers: Vec<Vec<f64>>,
    /// Features to use
    pub features: Vec<String>,
}

impl KMeansRegime {
    pub fn new(k: usize) -> Self {
        KMeansRegime {
            k,
            centers: Vec::new(),
            features: vec!["return".to_string(), "volatility".to_string()],
        }
    }

    /// Fit k-means to return and volatility data
    pub fn fit(&mut self, returns: &Series, window: usize, max_iter: usize) -> Result<()> {
        let values = series_to_vec(returns)?;
        let n = values.len();

        if n < window + self.k {
            return Err(SigcError::Runtime("Not enough data for clustering".into()));
        }

        // Create feature vectors [return, volatility]
        let mut data = Vec::new();
        for i in window..n {
            let slice = &values[(i + 1 - window)..=i];
            let mean: f64 = slice.iter().sum::<f64>() / window as f64;
            let var: f64 = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;
            let vol = var.sqrt();
            data.push(vec![mean, vol]);
        }

        // Initialize centers using k-means++
        self.centers = self.kmeans_pp_init(&data);

        // K-means iterations
        for _ in 0..max_iter {
            // Assign points to clusters
            let assignments: Vec<usize> = data
                .iter()
                .map(|point| {
                    self.centers
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            let da = euclidean_dist(point, a);
                            let db = euclidean_dist(point, b);
                            da.partial_cmp(&db).unwrap()
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0)
                })
                .collect();

            // Update centers
            let mut new_centers = vec![vec![0.0; 2]; self.k];
            let mut counts = vec![0; self.k];

            for (i, point) in data.iter().enumerate() {
                let cluster = assignments[i];
                for j in 0..2 {
                    new_centers[cluster][j] += point[j];
                }
                counts[cluster] += 1;
            }

            for c in 0..self.k {
                if counts[c] > 0 {
                    for j in 0..2 {
                        new_centers[c][j] /= counts[c] as f64;
                    }
                }
            }

            self.centers = new_centers;
        }

        // Sort centers by return (first feature)
        self.centers.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

        Ok(())
    }

    /// K-means++ initialization
    fn kmeans_pp_init(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let mut centers = Vec::with_capacity(self.k);

        // First center: random point
        centers.push(data[0].clone());

        // Remaining centers: probability proportional to distance squared
        for _ in 1..self.k {
            let mut max_dist = 0.0;
            let mut best_point = data[0].clone();

            for point in data {
                let min_dist = centers
                    .iter()
                    .map(|c| euclidean_dist(point, c))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                if min_dist > max_dist {
                    max_dist = min_dist;
                    best_point = point.clone();
                }
            }

            centers.push(best_point);
        }

        centers
    }

    /// Predict cluster assignments
    pub fn predict(&self, returns: &Series, window: usize) -> Result<Series> {
        let values = series_to_vec(returns)?;
        let n = values.len();

        let mut assignments = vec![0.0; n];

        for i in window..n {
            let slice = &values[(i + 1 - window)..=i];
            let mean: f64 = slice.iter().sum::<f64>() / window as f64;
            let var: f64 = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;
            let vol = var.sqrt();
            let point = vec![mean, vol];

            let cluster = self
                .centers
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = euclidean_dist(&point, a);
                    let db = euclidean_dist(&point, b);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|(i, _)| i)
                .unwrap_or(0);

            assignments[i] = cluster as f64;
        }

        Ok(Series::new("cluster".into(), assignments))
    }
}

// Helper functions

fn series_to_vec(series: &Series) -> Result<Vec<f64>> {
    series
        .f64()
        .map_err(|e| SigcError::Runtime(format!("Failed to convert series: {}", e)))?
        .into_iter()
        .map(|v| Ok(v.unwrap_or(f64::NAN)))
        .collect()
}

fn gaussian_pdf(x: f64, mean: f64, std: f64) -> f64 {
    let z = (x - mean) / std;
    (-z * z / 2.0).exp() / (std * (2.0 * std::f64::consts::PI).sqrt())
}

fn euclidean_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_returns() -> Series {
        Series::new(
            "returns".into(),
            vec![
                0.01, 0.02, -0.01, 0.015, 0.005, -0.02, 0.03, -0.01, 0.02, 0.01,
                -0.015, 0.025, 0.01, -0.005, 0.02, 0.015, -0.01, 0.005, 0.02, -0.02,
            ],
        )
    }

    #[test]
    fn test_hmm_creation() {
        let hmm = HiddenMarkovModel::bull_bear();
        assert_eq!(hmm.n_states, 2);
        assert!(hmm.emission_means[0] > hmm.emission_means[1]);
    }

    #[test]
    fn test_hmm_predict() {
        let hmm = HiddenMarkovModel::bull_bear();
        let returns = sample_returns();
        let states = hmm.predict(&returns).unwrap();
        assert_eq!(states.len(), returns.len());
    }

    #[test]
    fn test_volatility_regime() {
        let detector = VolatilityRegime::new(5, 10);
        let returns = sample_returns();
        let regimes = detector.detect(&returns).unwrap();
        assert_eq!(regimes.len(), returns.len());
    }

    #[test]
    fn test_trend_regime() {
        let prices = Series::new(
            "prices".into(),
            vec![
                100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
                110.0, 109.0, 108.0, 107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0,
            ],
        );
        let detector = TrendRegime::new(5, 10);
        let regimes = detector.detect(&prices).unwrap();
        assert_eq!(regimes.len(), prices.len());
    }

    #[test]
    fn test_combined_regime() {
        let prices = Series::new(
            "prices".into(),
            (0..20).map(|i| 100.0 + i as f64).collect::<Vec<_>>(),
        );
        let returns = sample_returns();

        let detector = RegimeDetector::new();
        let regimes = detector.detect(&prices, &returns).unwrap();
        assert_eq!(regimes.len(), 20);
    }

    #[test]
    fn test_market_regime_display() {
        let regime = MarketRegime::BullQuiet;
        assert_eq!(format!("{}", regime), "Bull (Quiet)");
    }

    #[test]
    fn test_kmeans_regime() {
        let returns = sample_returns();
        let mut kmeans = KMeansRegime::new(3);
        kmeans.fit(&returns, 5, 10).unwrap();

        let clusters = kmeans.predict(&returns, 5).unwrap();
        assert_eq!(clusters.len(), returns.len());
    }
}
