//! Incremental computation support
//!
//! Enables efficient updates when new data arrives without full recomputation.

use polars::prelude::*;
use sig_types::{Result, SigcError};
use std::collections::HashMap;

/// State for incremental rolling mean
#[derive(Debug, Clone)]
pub struct RollingMeanState {
    pub window: usize,
    pub buffer: Vec<f64>,
    pub sum: f64,
    pub count: usize,
}

impl RollingMeanState {
    pub fn new(window: usize) -> Self {
        RollingMeanState {
            window,
            buffer: Vec::with_capacity(window),
            sum: 0.0,
            count: 0,
        }
    }

    /// Update with a new value and return the new mean
    pub fn update(&mut self, value: f64) -> f64 {
        self.buffer.push(value);
        self.sum += value;
        self.count += 1;

        if self.buffer.len() > self.window {
            let removed = self.buffer.remove(0);
            self.sum -= removed;
        }

        self.sum / self.buffer.len() as f64
    }

    /// Update with multiple values
    pub fn update_batch(&mut self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.update(v)).collect()
    }

    /// Get current mean without adding a value
    pub fn current(&self) -> f64 {
        if self.buffer.is_empty() {
            0.0
        } else {
            self.sum / self.buffer.len() as f64
        }
    }
}

/// State for incremental rolling standard deviation
#[derive(Debug, Clone)]
pub struct RollingStdState {
    pub window: usize,
    pub buffer: Vec<f64>,
    pub sum: f64,
    pub sum_sq: f64,
}

impl RollingStdState {
    pub fn new(window: usize) -> Self {
        RollingStdState {
            window,
            buffer: Vec::with_capacity(window),
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) -> f64 {
        self.buffer.push(value);
        self.sum += value;
        self.sum_sq += value * value;

        if self.buffer.len() > self.window {
            let removed = self.buffer.remove(0);
            self.sum -= removed;
            self.sum_sq -= removed * removed;
        }

        let n = self.buffer.len() as f64;
        let mean = self.sum / n;
        let variance = (self.sum_sq / n) - (mean * mean);
        variance.max(0.0).sqrt()
    }

    pub fn update_batch(&mut self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.update(v)).collect()
    }
}

/// State for incremental EMA
#[derive(Debug, Clone)]
pub struct EmaState {
    pub alpha: f64,
    pub value: f64,
    pub initialized: bool,
}

impl EmaState {
    pub fn new(span: usize) -> Self {
        EmaState {
            alpha: 2.0 / (span as f64 + 1.0),
            value: 0.0,
            initialized: false,
        }
    }

    pub fn update(&mut self, value: f64) -> f64 {
        if !self.initialized {
            self.value = value;
            self.initialized = true;
        } else {
            self.value = self.alpha * value + (1.0 - self.alpha) * self.value;
        }
        self.value
    }

    pub fn update_batch(&mut self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.update(v)).collect()
    }
}

/// State for incremental cumulative sum
#[derive(Debug, Clone)]
pub struct CumsumState {
    pub sum: f64,
}

impl CumsumState {
    pub fn new() -> Self {
        CumsumState { sum: 0.0 }
    }

    pub fn update(&mut self, value: f64) -> f64 {
        self.sum += value;
        self.sum
    }

    pub fn update_batch(&mut self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.update(v)).collect()
    }
}

impl Default for CumsumState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for incremental RSI
#[derive(Debug, Clone)]
pub struct RsiState {
    pub alpha: f64,
    pub avg_gain: f64,
    pub avg_loss: f64,
    pub prev_value: f64,
    pub initialized: bool,
}

impl RsiState {
    pub fn new(window: usize) -> Self {
        RsiState {
            alpha: 1.0 / window as f64,
            avg_gain: 0.0,
            avg_loss: 0.0,
            prev_value: 0.0,
            initialized: false,
        }
    }

    pub fn update(&mut self, value: f64) -> f64 {
        if !self.initialized {
            self.prev_value = value;
            self.initialized = true;
            return 50.0;
        }

        let change = value - self.prev_value;
        self.prev_value = value;

        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };

        self.avg_gain = self.alpha * gain + (1.0 - self.alpha) * self.avg_gain;
        self.avg_loss = self.alpha * loss + (1.0 - self.alpha) * self.avg_loss;

        if self.avg_loss > 1e-10 {
            let rs = self.avg_gain / self.avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        } else if self.avg_gain > 1e-10 {
            100.0
        } else {
            50.0
        }
    }
}

/// Incremental computation manager
pub struct IncrementalCompute {
    /// Named states for different computations
    rolling_means: HashMap<String, RollingMeanState>,
    rolling_stds: HashMap<String, RollingStdState>,
    emas: HashMap<String, EmaState>,
    cumsums: HashMap<String, CumsumState>,
    rsis: HashMap<String, RsiState>,
}

impl IncrementalCompute {
    pub fn new() -> Self {
        IncrementalCompute {
            rolling_means: HashMap::new(),
            rolling_stds: HashMap::new(),
            emas: HashMap::new(),
            cumsums: HashMap::new(),
            rsis: HashMap::new(),
        }
    }

    /// Register a rolling mean computation
    pub fn register_rolling_mean(&mut self, name: &str, window: usize) {
        self.rolling_means.insert(name.to_string(), RollingMeanState::new(window));
    }

    /// Register a rolling std computation
    pub fn register_rolling_std(&mut self, name: &str, window: usize) {
        self.rolling_stds.insert(name.to_string(), RollingStdState::new(window));
    }

    /// Register an EMA computation
    pub fn register_ema(&mut self, name: &str, span: usize) {
        self.emas.insert(name.to_string(), EmaState::new(span));
    }

    /// Register a cumsum computation
    pub fn register_cumsum(&mut self, name: &str) {
        self.cumsums.insert(name.to_string(), CumsumState::new());
    }

    /// Register an RSI computation
    pub fn register_rsi(&mut self, name: &str, window: usize) {
        self.rsis.insert(name.to_string(), RsiState::new(window));
    }

    /// Update rolling mean with new value
    pub fn update_rolling_mean(&mut self, name: &str, value: f64) -> Result<f64> {
        self.rolling_means
            .get_mut(name)
            .map(|state| state.update(value))
            .ok_or_else(|| SigcError::Runtime(format!("Rolling mean '{}' not found", name)))
    }

    /// Update rolling std with new value
    pub fn update_rolling_std(&mut self, name: &str, value: f64) -> Result<f64> {
        self.rolling_stds
            .get_mut(name)
            .map(|state| state.update(value))
            .ok_or_else(|| SigcError::Runtime(format!("Rolling std '{}' not found", name)))
    }

    /// Update EMA with new value
    pub fn update_ema(&mut self, name: &str, value: f64) -> Result<f64> {
        self.emas
            .get_mut(name)
            .map(|state| state.update(value))
            .ok_or_else(|| SigcError::Runtime(format!("EMA '{}' not found", name)))
    }

    /// Update cumsum with new value
    pub fn update_cumsum(&mut self, name: &str, value: f64) -> Result<f64> {
        self.cumsums
            .get_mut(name)
            .map(|state| state.update(value))
            .ok_or_else(|| SigcError::Runtime(format!("Cumsum '{}' not found", name)))
    }

    /// Update RSI with new value
    pub fn update_rsi(&mut self, name: &str, value: f64) -> Result<f64> {
        self.rsis
            .get_mut(name)
            .map(|state| state.update(value))
            .ok_or_else(|| SigcError::Runtime(format!("RSI '{}' not found", name)))
    }

    /// Batch update for multiple values
    pub fn update_batch(&mut self, name: &str, values: &[f64], op: &str) -> Result<Vec<f64>> {
        match op {
            "rolling_mean" => {
                let state = self.rolling_means.get_mut(name)
                    .ok_or_else(|| SigcError::Runtime(format!("Rolling mean '{}' not found", name)))?;
                Ok(state.update_batch(values))
            }
            "rolling_std" => {
                let state = self.rolling_stds.get_mut(name)
                    .ok_or_else(|| SigcError::Runtime(format!("Rolling std '{}' not found", name)))?;
                Ok(state.update_batch(values))
            }
            "ema" => {
                let state = self.emas.get_mut(name)
                    .ok_or_else(|| SigcError::Runtime(format!("EMA '{}' not found", name)))?;
                Ok(state.update_batch(values))
            }
            "cumsum" => {
                let state = self.cumsums.get_mut(name)
                    .ok_or_else(|| SigcError::Runtime(format!("Cumsum '{}' not found", name)))?;
                Ok(state.update_batch(values))
            }
            _ => Err(SigcError::Runtime(format!("Unknown operation: {}", op))),
        }
    }

    /// Reset a specific state
    pub fn reset(&mut self, name: &str, op: &str) {
        match op {
            "rolling_mean" => {
                if let Some(state) = self.rolling_means.get_mut(name) {
                    let window = state.window;
                    *state = RollingMeanState::new(window);
                }
            }
            "rolling_std" => {
                if let Some(state) = self.rolling_stds.get_mut(name) {
                    let window = state.window;
                    *state = RollingStdState::new(window);
                }
            }
            "ema" => {
                if let Some(state) = self.emas.get_mut(name) {
                    let alpha = state.alpha;
                    state.value = 0.0;
                    state.initialized = false;
                    state.alpha = alpha;
                }
            }
            "cumsum" => {
                if let Some(state) = self.cumsums.get_mut(name) {
                    state.sum = 0.0;
                }
            }
            "rsi" => {
                if let Some(state) = self.rsis.get_mut(name) {
                    let alpha = state.alpha;
                    *state = RsiState { alpha, ..RsiState::new(1) };
                }
            }
            _ => {}
        }
    }

    /// Clear all states
    pub fn clear(&mut self) {
        self.rolling_means.clear();
        self.rolling_stds.clear();
        self.emas.clear();
        self.cumsums.clear();
        self.rsis.clear();
    }
}

impl Default for IncrementalCompute {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental DataFrame processor
pub struct IncrementalProcessor {
    /// Current state of the data
    current_data: Option<DataFrame>,
    /// Incremental compute states per column
    computes: HashMap<String, IncrementalCompute>,
}

impl IncrementalProcessor {
    pub fn new() -> Self {
        IncrementalProcessor {
            current_data: None,
            computes: HashMap::new(),
        }
    }

    /// Initialize with initial data
    pub fn initialize(&mut self, df: DataFrame) {
        self.current_data = Some(df);
    }

    /// Append new data and compute incremental updates
    pub fn append(&mut self, new_data: DataFrame) -> Result<()> {
        match &mut self.current_data {
            Some(current) => {
                *current = current
                    .vstack(&new_data)
                    .map_err(|e| SigcError::Runtime(format!("Failed to append: {}", e)))?;
            }
            None => {
                self.current_data = Some(new_data);
            }
        }
        Ok(())
    }

    /// Get current data
    pub fn data(&self) -> Option<&DataFrame> {
        self.current_data.as_ref()
    }

    /// Register a computation for a column
    pub fn register_computation(&mut self, column: &str, name: &str, op: &str, params: &[usize]) {
        let compute = self.computes.entry(column.to_string()).or_default();

        match op {
            "rolling_mean" => compute.register_rolling_mean(name, params.first().copied().unwrap_or(20)),
            "rolling_std" => compute.register_rolling_std(name, params.first().copied().unwrap_or(20)),
            "ema" => compute.register_ema(name, params.first().copied().unwrap_or(20)),
            "cumsum" => compute.register_cumsum(name),
            "rsi" => compute.register_rsi(name, params.first().copied().unwrap_or(14)),
            _ => {}
        }
    }

    /// Get number of rows
    pub fn len(&self) -> usize {
        self.current_data.as_ref().map(|df| df.height()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for IncrementalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_mean_state() {
        let mut state = RollingMeanState::new(3);

        assert_eq!(state.update(1.0), 1.0);
        assert_eq!(state.update(2.0), 1.5);
        assert_eq!(state.update(3.0), 2.0);
        assert_eq!(state.update(4.0), 3.0); // (2+3+4)/3
        assert_eq!(state.update(5.0), 4.0); // (3+4+5)/3
    }

    #[test]
    fn test_rolling_std_state() {
        let mut state = RollingStdState::new(3);

        state.update(1.0);
        state.update(2.0);
        let std = state.update(3.0);
        assert!((std - 0.816).abs() < 0.01); // std of [1,2,3]
    }

    #[test]
    fn test_ema_state() {
        let mut state = EmaState::new(3);

        let v1 = state.update(1.0);
        assert_eq!(v1, 1.0);

        let v2 = state.update(2.0);
        // EMA = 0.5 * 2 + 0.5 * 1 = 1.5
        assert!((v2 - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_cumsum_state() {
        let mut state = CumsumState::new();

        assert_eq!(state.update(1.0), 1.0);
        assert_eq!(state.update(2.0), 3.0);
        assert_eq!(state.update(3.0), 6.0);
    }

    #[test]
    fn test_rsi_state() {
        let mut state = RsiState::new(14);

        // Initial value
        let rsi = state.update(100.0);
        assert_eq!(rsi, 50.0);

        // Price increase
        let rsi = state.update(105.0);
        assert!(rsi > 50.0);
    }

    #[test]
    fn test_incremental_compute() {
        let mut compute = IncrementalCompute::new();

        compute.register_rolling_mean("sma_20", 20);
        compute.register_ema("ema_10", 10);
        compute.register_cumsum("cumret");

        let result = compute.update_rolling_mean("sma_20", 100.0).unwrap();
        assert_eq!(result, 100.0);

        let result = compute.update_ema("ema_10", 100.0).unwrap();
        assert_eq!(result, 100.0);

        let result = compute.update_cumsum("cumret", 0.01).unwrap();
        assert_eq!(result, 0.01);
    }

    #[test]
    fn test_batch_update() {
        let mut compute = IncrementalCompute::new();
        compute.register_rolling_mean("test", 3);

        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let results = compute.update_batch("test", &values, "rolling_mean").unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[4], 4.0); // (3+4+5)/3
    }

    #[test]
    fn test_incremental_processor() {
        let mut processor = IncrementalProcessor::new();

        let df = DataFrame::new(vec![
            Column::new("close".into(), vec![100.0, 101.0, 102.0]),
        ])
        .unwrap();

        processor.initialize(df);
        assert_eq!(processor.len(), 3);

        let new_data = DataFrame::new(vec![
            Column::new("close".into(), vec![103.0, 104.0]),
        ])
        .unwrap();

        processor.append(new_data).unwrap();
        assert_eq!(processor.len(), 5);
    }
}
