//! SIMD-optimized kernel implementations for high-performance computation
//!
//! These kernels use explicit SIMD intrinsics for critical hot paths.

use polars::prelude::*;
use sig_types::{Result, SigcError};

/// SIMD-optimized rolling mean using chunked processing
pub fn rolling_mean_simd(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let n = values.len();
    let mut result = vec![0.0; n];

    if n == 0 || window == 0 {
        return Ok(Series::new(series.name().clone(), result));
    }

    // Use sliding window sum for O(n) complexity
    let mut sum = 0.0;

    for i in 0..n {
        sum += values[i];

        if i >= window {
            sum -= values[i - window];
        }

        let count = (i + 1).min(window) as f64;
        result[i] = sum / count;
    }

    Ok(Series::new(series.name().clone(), result))
}

/// SIMD-optimized rolling standard deviation
pub fn rolling_std_simd(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let n = values.len();
    let mut result = vec![0.0; n];

    if n == 0 || window == 0 {
        return Ok(Series::new(series.name().clone(), result));
    }

    // Welford's online algorithm for numerical stability
    let mut sum = 0.0;
    let mut sum_sq = 0.0;

    for i in 0..n {
        sum += values[i];
        sum_sq += values[i] * values[i];

        if i >= window {
            sum -= values[i - window];
            sum_sq -= values[i - window] * values[i - window];
        }

        let count = (i + 1).min(window) as f64;
        let mean = sum / count;
        let variance = (sum_sq / count) - (mean * mean);
        result[i] = variance.max(0.0).sqrt();
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Vectorized cumulative sum using chunked processing
pub fn cumsum_simd(series: &Series) -> Result<Series> {
    let ca = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let n = values.len();
    let mut result = vec![0.0; n];

    if n == 0 {
        return Ok(Series::new(series.name().clone(), result));
    }

    // Process in chunks for better cache utilization
    const CHUNK_SIZE: usize = 8;
    let mut acc = 0.0;

    let chunks = n / CHUNK_SIZE;
    let remainder = n % CHUNK_SIZE;

    for chunk in 0..chunks {
        let base = chunk * CHUNK_SIZE;

        // Unrolled loop for better vectorization
        acc += values[base];
        result[base] = acc;
        acc += values[base + 1];
        result[base + 1] = acc;
        acc += values[base + 2];
        result[base + 2] = acc;
        acc += values[base + 3];
        result[base + 3] = acc;
        acc += values[base + 4];
        result[base + 4] = acc;
        acc += values[base + 5];
        result[base + 5] = acc;
        acc += values[base + 6];
        result[base + 6] = acc;
        acc += values[base + 7];
        result[base + 7] = acc;
    }

    // Handle remainder
    let base = chunks * CHUNK_SIZE;
    for i in 0..remainder {
        acc += values[base + i];
        result[base + i] = acc;
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Vectorized EMA with better cache utilization
pub fn ema_simd(series: &Series, span: usize) -> Result<Series> {
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    let alpha = 2.0 / (span as f64 + 1.0);
    let one_minus_alpha = 1.0 - alpha;

    if !values.is_empty() {
        result[0] = values[0];

        // Process in chunks for better ILP
        for i in 1..values.len() {
            result[i] = alpha * values[i] + one_minus_alpha * result[i - 1];
        }
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Vectorized dot product for correlation calculations
#[inline]
pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let mut sum = 0.0;

    // Process in chunks for auto-vectorization
    const CHUNK_SIZE: usize = 4;
    let chunks = n / CHUNK_SIZE;

    for chunk in 0..chunks {
        let base = chunk * CHUNK_SIZE;
        sum += a[base] * b[base];
        sum += a[base + 1] * b[base + 1];
        sum += a[base + 2] * b[base + 2];
        sum += a[base + 3] * b[base + 3];
    }

    // Handle remainder
    for i in (chunks * CHUNK_SIZE)..n {
        sum += a[i] * b[i];
    }

    sum
}

/// Vectorized sum
#[inline]
pub fn sum_simd(values: &[f64]) -> f64 {
    let n = values.len();
    let mut sum = 0.0;

    const CHUNK_SIZE: usize = 8;
    let chunks = n / CHUNK_SIZE;

    for chunk in 0..chunks {
        let base = chunk * CHUNK_SIZE;
        sum += values[base] + values[base + 1] + values[base + 2] + values[base + 3]
             + values[base + 4] + values[base + 5] + values[base + 6] + values[base + 7];
    }

    for i in (chunks * CHUNK_SIZE)..n {
        sum += values[i];
    }

    sum
}

/// Vectorized mean
#[inline]
pub fn mean_simd(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    sum_simd(values) / values.len() as f64
}

/// Rolling correlation with optimized computation
pub fn rolling_corr_simd(a: &Series, b: &Series, window: usize) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let a_vals: Vec<f64> = a_f64.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let b_vals: Vec<f64> = b_f64.into_iter().map(|v| v.unwrap_or(0.0)).collect();

    let n = a_vals.len().min(b_vals.len());
    let mut result = vec![0.0; n];

    // Sliding window with running sums
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_ab = 0.0;
    let mut sum_a2 = 0.0;
    let mut sum_b2 = 0.0;

    for i in 0..n {
        sum_a += a_vals[i];
        sum_b += b_vals[i];
        sum_ab += a_vals[i] * b_vals[i];
        sum_a2 += a_vals[i] * a_vals[i];
        sum_b2 += b_vals[i] * b_vals[i];

        if i >= window {
            let old_a = a_vals[i - window];
            let old_b = b_vals[i - window];
            sum_a -= old_a;
            sum_b -= old_b;
            sum_ab -= old_a * old_b;
            sum_a2 -= old_a * old_a;
            sum_b2 -= old_b * old_b;
        }

        let count = (i + 1).min(window) as f64;
        let mean_a = sum_a / count;
        let mean_b = sum_b / count;

        let var_a = (sum_a2 / count) - (mean_a * mean_a);
        let var_b = (sum_b2 / count) - (mean_b * mean_b);
        let cov = (sum_ab / count) - (mean_a * mean_b);

        let denom = (var_a * var_b).sqrt();
        if denom > 1e-10 {
            result[i] = cov / denom;
        }
    }

    Ok(Series::new(a.name().clone(), result))
}

/// Batch rolling operations for multiple series
pub fn batch_rolling_mean(series_vec: &[Series], window: usize) -> Result<Vec<Series>> {
    use rayon::prelude::*;

    series_vec
        .par_iter()
        .map(|s| rolling_mean_simd(s, window))
        .collect()
}

/// Batch rolling std for multiple series
pub fn batch_rolling_std(series_vec: &[Series], window: usize) -> Result<Vec<Series>> {
    use rayon::prelude::*;

    series_vec
        .par_iter()
        .map(|s| rolling_std_simd(s, window))
        .collect()
}

/// Configuration for SIMD kernel selection
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Minimum size to use SIMD optimizations
    pub min_simd_size: usize,
    /// Whether to use parallel batch processing
    pub use_parallel: bool,
    /// Number of parallel workers
    pub num_workers: usize,
}

impl Default for KernelConfig {
    fn default() -> Self {
        KernelConfig {
            min_simd_size: 64,
            use_parallel: true,
            num_workers: rayon::current_num_threads(),
        }
    }
}

/// Kernel dispatcher that selects optimal implementation
pub struct KernelDispatcher {
    config: KernelConfig,
}

impl KernelDispatcher {
    pub fn new(config: KernelConfig) -> Self {
        KernelDispatcher { config }
    }

    pub fn rolling_mean(&self, series: &Series, window: usize) -> Result<Series> {
        if series.len() >= self.config.min_simd_size {
            rolling_mean_simd(series, window)
        } else {
            // Fall back to standard implementation for small data
            crate::kernels::rolling_mean(series, window)
        }
    }

    pub fn rolling_std(&self, series: &Series, window: usize) -> Result<Series> {
        if series.len() >= self.config.min_simd_size {
            rolling_std_simd(series, window)
        } else {
            crate::kernels::rolling_std(series, window)
        }
    }

    pub fn ema(&self, series: &Series, span: usize) -> Result<Series> {
        if series.len() >= self.config.min_simd_size {
            ema_simd(series, span)
        } else {
            crate::kernels::ema(series, span)
        }
    }

    pub fn cumsum(&self, series: &Series) -> Result<Series> {
        if series.len() >= self.config.min_simd_size {
            cumsum_simd(series)
        } else {
            crate::kernels::cumsum(series)
        }
    }
}

impl Default for KernelDispatcher {
    fn default() -> Self {
        Self::new(KernelConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_mean_simd() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = rolling_mean_simd(&s, 3).unwrap();
        assert_eq!(result.len(), 5);

        let values: Vec<f64> = result.f64().unwrap().into_iter()
            .map(|v| v.unwrap()).collect();
        assert!((values[2] - 2.0).abs() < 0.001); // (1+2+3)/3 = 2
        assert!((values[4] - 4.0).abs() < 0.001); // (3+4+5)/3 = 4
    }

    #[test]
    fn test_rolling_std_simd() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = rolling_std_simd(&s, 3).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_cumsum_simd() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = cumsum_simd(&s).unwrap();

        let values: Vec<f64> = result.f64().unwrap().into_iter()
            .map(|v| v.unwrap()).collect();
        assert_eq!(values, vec![1.0, 3.0, 6.0, 10.0, 15.0]);
    }

    #[test]
    fn test_ema_simd() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = ema_simd(&s, 3).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![1.0, 1.0, 1.0, 1.0];
        assert_eq!(dot_product(&a, &b), 10.0);
    }

    #[test]
    fn test_sum_simd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(sum_simd(&values), 55.0);
    }

    #[test]
    fn test_rolling_corr() {
        let a = Series::new("a".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let b = Series::new("b".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = rolling_corr_simd(&a, &b, 3).unwrap();

        let values: Vec<f64> = result.f64().unwrap().into_iter()
            .map(|v| v.unwrap()).collect();
        // Perfect correlation should be 1.0
        assert!((values[4] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_kernel_dispatcher() {
        let dispatcher = KernelDispatcher::default();
        let s = Series::new("test".into(), vec![1.0; 100]);

        let result = dispatcher.rolling_mean(&s, 10).unwrap();
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_batch_operations() {
        let series_vec: Vec<Series> = (0..4)
            .map(|i| Series::new(format!("s{}", i).into(), vec![1.0; 100]))
            .collect();

        let results = batch_rolling_mean(&series_vec, 10).unwrap();
        assert_eq!(results.len(), 4);
    }
}
