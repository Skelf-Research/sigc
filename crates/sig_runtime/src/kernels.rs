//! SIMD-accelerated kernel implementations

use polars::prelude::*;
use sig_types::{Result, SigcError};

/// Compute lagged values
pub fn lag(series: &Series, periods: i32) -> Result<Series> {
    Ok(series.shift(periods as i64))
}

/// Compute returns over a period
pub fn ret(series: &Series, periods: i32) -> Result<Series> {
    let lagged = lag(series, periods)?;
    let current = series.clone();

    // (current / lagged) - 1
    let ratio = (&current / &lagged)
        .map_err(|e| SigcError::Runtime(format!("Division failed: {}", e)))?;

    Ok(&ratio - 1.0)
}

/// Rolling mean (simplified implementation)
pub fn rolling_mean(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        result[i] = slice.iter().sum::<f64>() / slice.len() as f64;
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Rolling standard deviation (simplified implementation)
pub fn rolling_std(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let mean = slice.iter().sum::<f64>() / slice.len() as f64;
        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / slice.len() as f64;
        result[i] = variance.sqrt();
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Cross-sectional z-score (standardize across assets at each time)
pub fn zscore(series: &Series) -> Result<Series> {
    let mean = series.mean().unwrap_or(0.0);
    let std = series.std(1).unwrap_or(1.0);

    if std == 0.0 {
        return Ok(series.clone() * 0.0);
    }

    let centered = series - mean;
    Ok(&centered / std)
}

/// Cross-sectional rank (0 to 1)
pub fn rank(series: &Series) -> Result<Series> {
    let n = series.len() as f64;
    if n <= 1.0 {
        return Ok(series.clone());
    }

    // Simple rank implementation
    let f64_series = series
        .cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    // Get values with indices
    let mut indexed: Vec<(usize, f64)> = ca
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i, v.unwrap_or(f64::NAN)))
        .collect();

    // Sort by value
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Assign ranks
    let mut ranks = vec![0.0; indexed.len()];
    for (rank, (idx, _)) in indexed.iter().enumerate() {
        ranks[*idx] = (rank + 1) as f64 / n;
    }

    Ok(Series::new(series.name().clone(), ranks))
}

/// Winsorize values at percentiles
pub fn winsor(series: &Series, lower_pct: f64, upper_pct: f64) -> Result<Series> {
    let sorted = series.sort(SortOptions::default())
        .map_err(|e| SigcError::Runtime(format!("Sort failed: {}", e)))?;

    let n = series.len();
    let lower_idx = (n as f64 * lower_pct) as usize;
    let upper_idx = (n as f64 * upper_pct) as usize;

    let lower_val = sorted.get(lower_idx.min(n.saturating_sub(1)))
        .map_err(|e| SigcError::Runtime(format!("Get lower failed: {}", e)))?;
    let upper_val = sorted.get(upper_idx.min(n.saturating_sub(1)))
        .map_err(|e| SigcError::Runtime(format!("Get upper failed: {}", e)))?;

    // Extract f64 values
    let lower_f = match lower_val {
        AnyValue::Float64(v) => v,
        AnyValue::Float32(v) => v as f64,
        AnyValue::Int64(v) => v as f64,
        AnyValue::Int32(v) => v as f64,
        _ => 0.0,
    };
    let upper_f = match upper_val {
        AnyValue::Float64(v) => v,
        AnyValue::Float32(v) => v as f64,
        AnyValue::Int64(v) => v as f64,
        AnyValue::Int32(v) => v as f64,
        _ => 0.0,
    };

    // Clip values
    clip(series, lower_f, upper_f)
}

/// Clip values to range
pub fn clip(series: &Series, min: f64, max: f64) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.max(min).min(max)))
        .into_series();

    Ok(result)
}

/// Long-short portfolio weights
pub fn long_short(series: &Series, long_pct: f64, short_pct: f64, cap: f64) -> Result<Series> {
    let ranked = rank(series)?;
    let n = series.len();

    // Top long_pct get positive weights, bottom short_pct get negative
    let weights = ranked.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| {
            v.map(|r| {
                if r >= 1.0 - long_pct {
                    // Long position
                    (cap).min(1.0 / (n as f64 * long_pct))
                } else if r <= short_pct {
                    // Short position
                    (-cap).max(-1.0 / (n as f64 * short_pct))
                } else {
                    0.0
                }
            })
        })
        .into_series();

    Ok(weights)
}

/// Absolute value
pub fn abs(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.abs()))
        .into_series();
    Ok(result)
}

/// Sign function (-1, 0, or 1)
pub fn sign(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }))
        .into_series();
    Ok(result)
}

/// Natural logarithm
pub fn log(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.ln()))
        .into_series();
    Ok(result)
}

/// Exponential
pub fn exp(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.exp()))
        .into_series();
    Ok(result)
}

/// Power
pub fn pow(series: &Series, exponent: f64) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.powf(exponent)))
        .into_series();
    Ok(result)
}

/// Square root
pub fn sqrt(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.sqrt()))
        .into_series();
    Ok(result)
}

/// Floor
pub fn floor(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.floor()))
        .into_series();
    Ok(result)
}

/// Ceiling
pub fn ceil(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| x.ceil()))
        .into_series();
    Ok(result)
}

/// Round to decimals
pub fn round(series: &Series, decimals: i32) -> Result<Series> {
    let factor = 10f64.powi(decimals);
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| (x * factor).round() / factor))
        .into_series();
    Ok(result)
}

/// Element-wise minimum of two series
pub fn min_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let result: Vec<f64> = a_f64.into_iter()
        .zip(b_f64.into_iter())
        .map(|(x, y)| {
            match (x, y) {
                (Some(a), Some(b)) => a.min(b),
                _ => f64::NAN,
            }
        })
        .collect();

    Ok(Series::new(a.name().clone(), result))
}

/// Element-wise maximum of two series
pub fn max_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let result: Vec<f64> = a_f64.into_iter()
        .zip(b_f64.into_iter())
        .map(|(x, y)| {
            match (x, y) {
                (Some(a), Some(b)) => a.max(b),
                _ => f64::NAN,
            }
        })
        .collect();

    Ok(Series::new(a.name().clone(), result))
}

/// Compute difference over a period
pub fn delta(series: &Series, periods: i32) -> Result<Series> {
    let lagged = lag(series, periods)?;
    (series - &lagged).map_err(|e| SigcError::Runtime(format!("Delta failed: {}", e)))
}

// Comparison operators (return 0.0 or 1.0)

/// Greater than
pub fn gt(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a > b { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Less than
pub fn lt(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a < b { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Greater or equal
pub fn ge(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a >= b { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Less or equal
pub fn le(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a <= b { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Equal
pub fn eq_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if (a - b).abs() < 1e-10 { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Not equal
pub fn ne_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if (a - b).abs() >= 1e-10 { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Logical and (treats non-zero as true)
pub fn and_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a != 0.0 && b != 0.0 { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Logical or
pub fn or_series(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) { (Some(a), Some(b)) => if a != 0.0 || b != 0.0 { 1.0 } else { 0.0 }, _ => f64::NAN })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Logical not
pub fn not_series(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| if x == 0.0 { 1.0 } else { 0.0 }))
        .into_series();
    Ok(result)
}

/// Conditional select: where(cond, x, y) returns x where cond != 0, else y
pub fn where_series(cond: &Series, x: &Series, y: &Series) -> Result<Series> {
    let c = cond.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let x_f64 = x.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let y_f64 = y.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let result: Vec<f64> = c.into_iter()
        .zip(x_f64.into_iter())
        .zip(y_f64.into_iter())
        .map(|((c, x), y)| {
            match (c, x, y) {
                (Some(cond), Some(xv), Some(yv)) => if cond != 0.0 { xv } else { yv },
                _ => f64::NAN,
            }
        })
        .collect();
    Ok(Series::new(cond.name().clone(), result))
}

// Data handling

/// Check for NaN (returns 1.0 if NaN, 0.0 otherwise)
pub fn is_nan(series: &Series) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| if x.is_nan() { 1.0 } else { 0.0 }))
        .into_series();
    Ok(result)
}

/// Fill NaN values with a constant
pub fn fill_nan(series: &Series, value: f64) -> Result<Series> {
    let result = series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|x| if x.is_nan() { value } else { x }))
        .into_series();
    Ok(result)
}

/// Coalesce: return first non-NaN value from two series
pub fn coalesce(a: &Series, b: &Series) -> Result<Series> {
    let a_f64 = a.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let b_f64 = b.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let result: Vec<f64> = a_f64.into_iter().zip(b_f64.into_iter())
        .map(|(x, y)| match (x, y) {
            (Some(av), _) if !av.is_nan() => av,
            (_, Some(bv)) => bv,
            _ => f64::NAN,
        })
        .collect();
    Ok(Series::new(a.name().clone(), result))
}

/// Cumulative sum
pub fn cumsum(series: &Series) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];
    let mut sum = 0.0;
    for (i, &v) in values.iter().enumerate() {
        sum += v;
        result[i] = sum;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Cumulative product
pub fn cumprod(series: &Series) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(1.0)).collect();
    let mut result = vec![0.0; values.len()];
    let mut prod = 1.0;
    for (i, &v) in values.iter().enumerate() {
        prod *= v;
        result[i] = prod;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Cumulative maximum
pub fn cummax(series: &Series) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NEG_INFINITY)).collect();
    let mut result = vec![0.0; values.len()];
    let mut max_val = f64::NEG_INFINITY;
    for (i, &v) in values.iter().enumerate() {
        max_val = max_val.max(v);
        result[i] = max_val;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Cumulative minimum
pub fn cummin(series: &Series) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::INFINITY)).collect();
    let mut result = vec![0.0; values.len()];
    let mut min_val = f64::INFINITY;
    for (i, &v) in values.iter().enumerate() {
        min_val = min_val.min(v);
        result[i] = min_val;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rolling sum
pub fn rolling_sum(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series.cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        result[i] = values[start..=i].iter().sum();
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Rolling minimum
pub fn rolling_min(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series.cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let mut result = vec![0.0; values.len()];

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        result[i] = values[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Rolling maximum
pub fn rolling_max(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series.cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let mut result = vec![0.0; values.len()];

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        result[i] = values[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Linear decay weighted average
pub fn decay_linear(series: &Series, window: usize) -> Result<Series> {
    let f64_series = series.cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    // Weights: 1, 2, 3, ..., window
    let weight_sum: f64 = (1..=window).map(|i| i as f64).sum();

    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let mut weighted_sum = 0.0;
        for (j, &val) in slice.iter().enumerate() {
            weighted_sum += val * (j + 1) as f64;
        }
        result[i] = weighted_sum / weight_sum;
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Exponential moving average
pub fn ema(series: &Series, span: usize) -> Result<Series> {
    let f64_series = series.cast(&DataType::Float64)
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let ca = f64_series.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    let alpha = 2.0 / (span as f64 + 1.0);

    if !values.is_empty() {
        result[0] = values[0];
        for i in 1..values.len() {
            result[i] = alpha * values[i] + (1.0 - alpha) * result[i - 1];
        }
    }

    Ok(Series::new(series.name().clone(), result))
}

/// Index of max value in rolling window (0-indexed from window start)
pub fn ts_argmax(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let max_idx = slice.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx).unwrap_or(0);
        result[i] = max_idx as f64;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Index of min value in rolling window
pub fn ts_argmin(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let min_idx = slice.iter().enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx).unwrap_or(0);
        result[i] = min_idx as f64;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rank within rolling window (0 to 1)
pub fn ts_rank(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let current = values[i];
        let rank = slice.iter().filter(|&&x| x < current).count() as f64;
        result[i] = rank / (slice.len() as f64 - 1.0).max(1.0);
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rolling skewness
pub fn ts_skew(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let n = slice.len() as f64;
        let mean = slice.iter().sum::<f64>() / n;
        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();
        if std > 1e-10 {
            let m3 = slice.iter().map(|x| ((x - mean) / std).powi(3)).sum::<f64>() / n;
            result[i] = m3;
        }
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rolling kurtosis
pub fn ts_kurt(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let n = slice.len() as f64;
        let mean = slice.iter().sum::<f64>() / n;
        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();
        if std > 1e-10 {
            let m4 = slice.iter().map(|x| ((x - mean) / std).powi(4)).sum::<f64>() / n;
            result[i] = m4 - 3.0; // excess kurtosis
        }
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rolling product
pub fn ts_product(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(1.0)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        result[i] = values[start..=i].iter().product();
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Rolling z-score (time series, not cross-sectional)
pub fn ts_zscore(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];
        let mean = slice.iter().sum::<f64>() / slice.len() as f64;
        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / slice.len() as f64;
        let std = variance.sqrt();
        if std > 1e-10 {
            result[i] = (values[i] - mean) / std;
        }
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Percentile rank (0 to 100)
pub fn rank_pct(series: &Series) -> Result<Series> {
    let ranked = rank(series)?;
    Ok(&ranked * 100.0)
}

/// Scale to sum to 1
pub fn scale(series: &Series) -> Result<Series> {
    let sum = series.sum::<f64>().unwrap_or(1.0);
    if sum.abs() < 1e-10 {
        return Ok(series.clone());
    }
    Ok(series / sum)
}

/// Demean (subtract mean)
pub fn demean(series: &Series) -> Result<Series> {
    let mean = series.mean().unwrap_or(0.0);
    Ok(series - mean)
}

/// Cross-sectional quantile value
pub fn quantile(series: &Series, q: f64) -> Result<Series> {
    let sorted = series.sort(SortOptions::default())
        .map_err(|e| SigcError::Runtime(format!("Sort failed: {}", e)))?;
    let n = series.len();
    let idx = ((n as f64 - 1.0) * q) as usize;
    let val = sorted.get(idx.min(n.saturating_sub(1)))
        .map_err(|e| SigcError::Runtime(format!("Get failed: {}", e)))?;
    let q_val = match val {
        AnyValue::Float64(v) => v,
        AnyValue::Float32(v) => v as f64,
        AnyValue::Int64(v) => v as f64,
        AnyValue::Int32(v) => v as f64,
        _ => 0.0,
    };
    // Return series of same length with the quantile value
    Ok(Series::new(series.name().clone(), vec![q_val; n]))
}

/// Assign to N equal-sized buckets (1 to N)
pub fn bucket(series: &Series, n: usize) -> Result<Series> {
    let ranked = rank(series)?;
    let result = ranked.f64()
        .map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?
        .apply(|v| v.map(|r| ((r * n as f64).ceil()).max(1.0).min(n as f64)))
        .into_series();
    Ok(result)
}

/// Cross-sectional median
pub fn median(series: &Series) -> Result<Series> {
    let med = series.median().unwrap_or(0.0);
    let n = series.len();
    Ok(Series::new(series.name().clone(), vec![med; n]))
}

/// Median absolute deviation
pub fn mad(series: &Series) -> Result<Series> {
    let med = series.median().unwrap_or(0.0);
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let deviations: Vec<f64> = ca.into_iter()
        .map(|v| (v.unwrap_or(0.0) - med).abs())
        .collect();
    let dev_series = Series::new("dev".into(), deviations);
    let mad_val = dev_series.median().unwrap_or(0.0);
    let n = series.len();
    Ok(Series::new(series.name().clone(), vec![mad_val; n]))
}

// Technical indicators

/// Relative Strength Index
pub fn rsi(series: &Series, window: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![50.0; values.len()]; // default to neutral

    if values.len() < 2 { return Ok(Series::new(series.name().clone(), result)); }

    // Calculate price changes
    let mut gains = vec![0.0; values.len()];
    let mut losses = vec![0.0; values.len()];
    for i in 1..values.len() {
        let change = values[i] - values[i - 1];
        if change > 0.0 { gains[i] = change; }
        else { losses[i] = -change; }
    }

    // Calculate RSI using EMA-style smoothing
    let alpha = 1.0 / window as f64;
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for i in 1..values.len() {
        avg_gain = alpha * gains[i] + (1.0 - alpha) * avg_gain;
        avg_loss = alpha * losses[i] + (1.0 - alpha) * avg_loss;

        if avg_loss > 1e-10 {
            let rs = avg_gain / avg_loss;
            result[i] = 100.0 - (100.0 / (1.0 + rs));
        } else if avg_gain > 1e-10 {
            result[i] = 100.0;
        } else {
            result[i] = 50.0;
        }
    }
    Ok(Series::new(series.name().clone(), result))
}

/// MACD (returns the MACD line = fast EMA - slow EMA)
pub fn macd(series: &Series, fast: usize, slow: usize, _signal: usize) -> Result<Series> {
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    if values.is_empty() { return Ok(Series::new(series.name().clone(), result)); }

    let alpha_fast = 2.0 / (fast as f64 + 1.0);
    let alpha_slow = 2.0 / (slow as f64 + 1.0);

    let mut ema_fast = values[0];
    let mut ema_slow = values[0];

    for i in 0..values.len() {
        ema_fast = alpha_fast * values[i] + (1.0 - alpha_fast) * ema_fast;
        ema_slow = alpha_slow * values[i] + (1.0 - alpha_slow) * ema_slow;
        result[i] = ema_fast - ema_slow;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Average True Range (simplified - uses only close prices, treating them as range)
pub fn atr(series: &Series, window: usize) -> Result<Series> {
    // Simplified ATR using absolute returns as proxy for true range
    let ca = series.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let values: Vec<f64> = ca.into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let mut result = vec![0.0; values.len()];

    // True range proxy: |close - prev_close|
    let mut tr = vec![0.0; values.len()];
    for i in 1..values.len() {
        tr[i] = (values[i] - values[i - 1]).abs();
    }

    // Rolling mean of true range
    for i in 0..values.len() {
        let start = i.saturating_sub(window - 1);
        let slice = &tr[start..=i];
        result[i] = slice.iter().sum::<f64>() / slice.len() as f64;
    }
    Ok(Series::new(series.name().clone(), result))
}

/// Volume-weighted average price (requires price and volume as two inputs)
pub fn vwap(price: &Series, volume: &Series) -> Result<Series> {
    let p = price.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;
    let v = volume.f64().map_err(|e| SigcError::Runtime(format!("Cast failed: {}", e)))?;

    let mut cum_pv = 0.0;
    let mut cum_v = 0.0;
    let mut result = vec![0.0; price.len()];

    for (i, (pv, vv)) in p.into_iter().zip(v.into_iter()).enumerate() {
        let price_val = pv.unwrap_or(0.0);
        let vol_val = vv.unwrap_or(0.0);
        cum_pv += price_val * vol_val;
        cum_v += vol_val;
        if cum_v > 1e-10 {
            result[i] = cum_pv / cum_v;
        }
    }
    Ok(Series::new(price.name().clone(), result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lag() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let lagged = lag(&s, 1).unwrap();
        assert_eq!(lagged.len(), 5);
    }

    #[test]
    fn test_zscore() {
        let s = Series::new("test".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let z = zscore(&s).unwrap();
        assert_eq!(z.len(), 5);
        // Mean should be ~0
        assert!(z.mean().unwrap().abs() < 1e-10);
    }

    #[test]
    fn test_rank() {
        let s = Series::new("test".into(), vec![3.0, 1.0, 4.0, 1.0, 5.0]);
        let r = rank(&s).unwrap();
        assert_eq!(r.len(), 5);
    }
}
