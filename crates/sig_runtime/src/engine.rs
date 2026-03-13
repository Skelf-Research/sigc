//! Execution engine for IR graphs

use polars::prelude::*;
use sig_types::{Ir, IrNode, Operator, OperatorError, Result, SigcError};
use std::collections::HashMap;
use crate::kernels;

/// Helper to check input count and return structured error
fn check_inputs(op: &str, node_id: u64, inputs: &[Series], expected: usize) -> Result<()> {
    if inputs.len() != expected {
        return Err(SigcError::Operator(OperatorError::input_mismatch(
            op, node_id as usize, expected, inputs.len()
        )));
    }
    Ok(())
}

/// Helper to check at least one input
fn check_has_input(op: &str, node_id: u64, inputs: &[Series]) -> Result<()> {
    if inputs.is_empty() {
        return Err(SigcError::Operator(OperatorError::input_mismatch(
            op, node_id as usize, 1, 0
        )));
    }
    Ok(())
}

/// Execution engine that evaluates IR graphs
pub struct Engine {
    /// Cached intermediate results by node ID
    cache: HashMap<u64, Series>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            cache: HashMap::new(),
        }
    }

    /// Execute an IR graph with given input data
    pub fn execute(&mut self, ir: &Ir, inputs: &HashMap<String, Series>) -> Result<Vec<Series>> {
        // Clear cache for fresh execution
        self.cache.clear();

        // Register parameters first (they get IDs 0, 1, 2, ...)
        // Then data sources follow
        let mut next_id = 0u64;

        for param in &ir.metadata.parameters {
            // Create a constant series for the parameter value
            let value = param.default_value;
            // Use a single-element series that will be broadcast
            let series = Series::new(param.name.clone().into(), vec![value]);
            self.cache.insert(next_id, series);
            tracing::debug!("Registered param '{}' = {} with ID {}", param.name, value, next_id);
            next_id += 1;
        }

        for data_source in &ir.metadata.data_sources {
            if let Some(series) = inputs.get(&data_source.name) {
                self.cache.insert(next_id, series.clone());
                tracing::debug!("Registered input '{}' with ID {}", data_source.name, next_id);
            } else {
                tracing::warn!("No input provided for data source '{}'", data_source.name);
            }
            next_id += 1;
        }

        // Execute each node in order
        for node in &ir.nodes {
            let result = self.execute_node(node)?;
            self.cache.insert(node.id, result);
        }

        // Collect outputs
        let outputs: Vec<Series> = ir.outputs
            .iter()
            .filter_map(|id| self.cache.get(id).cloned())
            .collect();

        Ok(outputs)
    }

    fn execute_node(&mut self, node: &IrNode) -> Result<Series> {
        let inputs: Vec<Series> = node.inputs
            .iter()
            .map(|id| {
                self.cache.get(id).cloned().ok_or_else(|| {
                    SigcError::Runtime(format!("Missing input node: {}", id))
                })
            })
            .collect::<Result<_>>()?;

        match &node.operator {
            Operator::Add => {
                check_inputs("Add", node.id, &inputs, 2)?;
                (&inputs[0] + &inputs[1])
                    .map_err(|e| SigcError::Runtime(format!("Add failed: {}", e)))
            }

            Operator::Sub => {
                check_inputs("Sub", node.id, &inputs, 2)?;
                (&inputs[0] - &inputs[1])
                    .map_err(|e| SigcError::Runtime(format!("Sub failed: {}", e)))
            }

            Operator::Mul => {
                check_inputs("Mul", node.id, &inputs, 2)?;
                (&inputs[0] * &inputs[1])
                    .map_err(|e| SigcError::Runtime(format!("Mul failed: {}", e)))
            }

            Operator::Div => {
                check_inputs("Div", node.id, &inputs, 2)?;
                (&inputs[0] / &inputs[1])
                    .map_err(|e| SigcError::Runtime(format!("Div failed: {}", e)))
            }

            Operator::Abs => {
                check_has_input("Abs", node.id, &inputs)?;
                kernels::abs(&inputs[0])
            }

            Operator::Sign => {
                check_has_input("Sign", node.id, &inputs)?;
                kernels::sign(&inputs[0])
            }

            Operator::Log => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Log requires 1 input".into()));
                }
                kernels::log(&inputs[0])
            }

            Operator::Exp => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Exp requires 1 input".into()));
                }
                kernels::exp(&inputs[0])
            }

            Operator::Pow { exponent } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Pow requires 1 input".into()));
                }
                kernels::pow(&inputs[0], *exponent)
            }

            Operator::Sqrt => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Sqrt requires 1 input".into()));
                }
                kernels::sqrt(&inputs[0])
            }

            Operator::Floor => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Floor requires 1 input".into()));
                }
                kernels::floor(&inputs[0])
            }

            Operator::Ceil => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Ceil requires 1 input".into()));
                }
                kernels::ceil(&inputs[0])
            }

            Operator::Round { decimals } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Round requires 1 input".into()));
                }
                kernels::round(&inputs[0], *decimals)
            }

            Operator::Min => {
                if inputs.len() != 2 {
                    return Err(SigcError::Runtime("Min requires 2 inputs".into()));
                }
                kernels::min_series(&inputs[0], &inputs[1])
            }

            Operator::Max => {
                if inputs.len() != 2 {
                    return Err(SigcError::Runtime("Max requires 2 inputs".into()));
                }
                kernels::max_series(&inputs[0], &inputs[1])
            }

            Operator::Clip { min, max } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Clip requires 1 input".into()));
                }
                kernels::clip(&inputs[0], *min, *max)
            }

            // Comparison operators
            Operator::Gt => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Gt requires 2 inputs".into())); }
                kernels::gt(&inputs[0], &inputs[1])
            }
            Operator::Lt => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Lt requires 2 inputs".into())); }
                kernels::lt(&inputs[0], &inputs[1])
            }
            Operator::Ge => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Ge requires 2 inputs".into())); }
                kernels::ge(&inputs[0], &inputs[1])
            }
            Operator::Le => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Le requires 2 inputs".into())); }
                kernels::le(&inputs[0], &inputs[1])
            }
            Operator::Eq => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Eq requires 2 inputs".into())); }
                kernels::eq_series(&inputs[0], &inputs[1])
            }
            Operator::Ne => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Ne requires 2 inputs".into())); }
                kernels::ne_series(&inputs[0], &inputs[1])
            }

            // Logical operators
            Operator::And => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("And requires 2 inputs".into())); }
                kernels::and_series(&inputs[0], &inputs[1])
            }
            Operator::Or => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Or requires 2 inputs".into())); }
                kernels::or_series(&inputs[0], &inputs[1])
            }
            Operator::Not => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Not requires 1 input".into())); }
                kernels::not_series(&inputs[0])
            }
            Operator::Where => {
                if inputs.len() != 3 { return Err(SigcError::Runtime("Where requires 3 inputs".into())); }
                kernels::where_series(&inputs[0], &inputs[1], &inputs[2])
            }

            // Data handling
            Operator::IsNan => {
                if inputs.is_empty() { return Err(SigcError::Runtime("IsNan requires 1 input".into())); }
                kernels::is_nan(&inputs[0])
            }
            Operator::FillNan { value } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("FillNan requires 1 input".into())); }
                kernels::fill_nan(&inputs[0], *value)
            }
            Operator::Coalesce => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Coalesce requires 2 inputs".into())); }
                kernels::coalesce(&inputs[0], &inputs[1])
            }
            Operator::Cumsum => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Cumsum requires 1 input".into())); }
                kernels::cumsum(&inputs[0])
            }
            Operator::Cumprod => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Cumprod requires 1 input".into())); }
                kernels::cumprod(&inputs[0])
            }
            Operator::Cummax => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Cummax requires 1 input".into())); }
                kernels::cummax(&inputs[0])
            }
            Operator::Cummin => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Cummin requires 1 input".into())); }
                kernels::cummin(&inputs[0])
            }

            Operator::Lag { periods } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Lag requires 1 input".into()));
                }
                kernels::lag(&inputs[0], *periods)
            }

            Operator::Ret { periods } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Ret requires 1 input".into()));
                }
                kernels::ret(&inputs[0], *periods)
            }

            Operator::RollingMean { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingMean requires 1 input".into()));
                }
                kernels::rolling_mean(&inputs[0], *window)
            }

            Operator::RollingStd { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingStd requires 1 input".into()));
                }
                kernels::rolling_std(&inputs[0], *window)
            }

            Operator::Delta { periods } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Delta requires 1 input".into()));
                }
                kernels::delta(&inputs[0], *periods)
            }

            Operator::RollingSum { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingSum requires 1 input".into()));
                }
                kernels::rolling_sum(&inputs[0], *window)
            }

            Operator::RollingMin { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingMin requires 1 input".into()));
                }
                kernels::rolling_min(&inputs[0], *window)
            }

            Operator::RollingMax { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingMax requires 1 input".into()));
                }
                kernels::rolling_max(&inputs[0], *window)
            }

            Operator::RollingCorr { window: _ } => {
                // Requires two inputs - placeholder for now
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RollingCorr requires inputs".into()));
                }
                Ok(inputs[0].clone())
            }

            Operator::DecayLinear { window } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("DecayLinear requires 1 input".into()));
                }
                kernels::decay_linear(&inputs[0], *window)
            }

            Operator::Ema { span } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Ema requires 1 input".into()));
                }
                kernels::ema(&inputs[0], *span)
            }

            Operator::TsArgmax { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsArgmax requires 1 input".into())); }
                kernels::ts_argmax(&inputs[0], *window)
            }
            Operator::TsArgmin { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsArgmin requires 1 input".into())); }
                kernels::ts_argmin(&inputs[0], *window)
            }
            Operator::TsRank { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsRank requires 1 input".into())); }
                kernels::ts_rank(&inputs[0], *window)
            }
            Operator::TsSkew { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsSkew requires 1 input".into())); }
                kernels::ts_skew(&inputs[0], *window)
            }
            Operator::TsKurt { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsKurt requires 1 input".into())); }
                kernels::ts_kurt(&inputs[0], *window)
            }
            Operator::TsProduct { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsProduct requires 1 input".into())); }
                kernels::ts_product(&inputs[0], *window)
            }
            Operator::TsZscore { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("TsZscore requires 1 input".into())); }
                kernels::ts_zscore(&inputs[0], *window)
            }

            Operator::Rank => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Rank requires 1 input".into()));
                }
                kernels::rank(&inputs[0])
            }

            Operator::RankPct => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("RankPct requires 1 input".into()));
                }
                kernels::rank_pct(&inputs[0])
            }

            Operator::Zscore => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Zscore requires 1 input".into()));
                }
                kernels::zscore(&inputs[0])
            }

            Operator::Scale => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Scale requires 1 input".into()));
                }
                kernels::scale(&inputs[0])
            }

            Operator::Demean => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Demean requires 1 input".into()));
                }
                kernels::demean(&inputs[0])
            }

            Operator::Winsor { lower, upper } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Winsor requires 1 input".into()));
                }
                kernels::winsor(&inputs[0], *lower, *upper)
            }

            Operator::Neutralize { groups: _ } => {
                // Simplified: just demean for now
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("Neutralize requires 1 input".into()));
                }
                let mean = inputs[0].mean().unwrap_or(0.0);
                Ok(&inputs[0] - mean)
            }

            Operator::Quantile { q } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Quantile requires 1 input".into())); }
                kernels::quantile(&inputs[0], *q)
            }
            Operator::Bucket { n } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Bucket requires 1 input".into())); }
                kernels::bucket(&inputs[0], *n)
            }
            Operator::Median => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Median requires 1 input".into())); }
                kernels::median(&inputs[0])
            }
            Operator::Mad => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Mad requires 1 input".into())); }
                kernels::mad(&inputs[0])
            }

            Operator::LongShort { long_pct, short_pct } => {
                if inputs.is_empty() {
                    return Err(SigcError::Runtime("LongShort requires 1 input".into()));
                }
                kernels::long_short(&inputs[0], *long_pct, *short_pct, 0.05)
            }

            // Technical indicators
            Operator::Rsi { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Rsi requires 1 input".into())); }
                kernels::rsi(&inputs[0], *window)
            }
            Operator::Macd { fast, slow, signal } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Macd requires 1 input".into())); }
                kernels::macd(&inputs[0], *fast, *slow, *signal)
            }
            Operator::Atr { window } => {
                if inputs.is_empty() { return Err(SigcError::Runtime("Atr requires 1 input".into())); }
                kernels::atr(&inputs[0], *window)
            }
            Operator::Vwap => {
                if inputs.len() != 2 { return Err(SigcError::Runtime("Vwap requires 2 inputs (price, volume)".into())); }
                kernels::vwap(&inputs[0], &inputs[1])
            }
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
