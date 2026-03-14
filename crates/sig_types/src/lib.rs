//! Core types for sigc signal compiler
//!
//! This crate defines the fundamental types used across the sigc ecosystem.

use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

/// Data types supported by sigc
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub enum DType {
    Float32,
    Float64,
    Int32,
    Int64,
    Bool,
    String,
    Date,
    DateTime,
}

/// Shape information for tensor-like data
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Shape {
    pub dims: Vec<usize>,
}

impl Shape {
    pub fn scalar() -> Self {
        Shape { dims: vec![] }
    }

    pub fn vector(len: usize) -> Self {
        Shape { dims: vec![len] }
    }

    pub fn matrix(rows: usize, cols: usize) -> Self {
        Shape { dims: vec![rows, cols] }
    }
}

/// Type annotation combining dtype and shape
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct TypeAnnotation {
    pub dtype: DType,
    pub shape: Shape,
}

/// Operator types in the IR
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub enum Operator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Abs,
    Sign,
    Log,
    Exp,
    Pow { exponent: f64 },
    Clip { min: f64, max: f64 },
    Sqrt,
    Floor,
    Ceil,
    Round { decimals: i32 },
    Min,  // element-wise min of two series
    Max,  // element-wise max of two series

    // Comparison
    Gt,   // greater than
    Lt,   // less than
    Ge,   // greater or equal
    Le,   // less or equal
    Eq,   // equal
    Ne,   // not equal

    // Logical
    And,
    Or,
    Not,
    Where,  // conditional select

    // Data handling
    IsNan,
    FillNan { value: f64 },
    Coalesce,
    Cumsum,
    Cumprod,
    Cummax,
    Cummin,

    // Time series
    Lag { periods: i32 },
    Ret { periods: i32 },
    Delta { periods: i32 },
    RollingMean { window: usize },
    RollingStd { window: usize },
    RollingSum { window: usize },
    RollingMin { window: usize },
    RollingMax { window: usize },
    RollingCorr { window: usize },
    DecayLinear { window: usize },
    Ema { span: usize },
    TsArgmax { window: usize },
    TsArgmin { window: usize },
    TsRank { window: usize },
    TsSkew { window: usize },
    TsKurt { window: usize },
    TsProduct { window: usize },
    TsZscore { window: usize },

    // Cross-sectional
    Rank,
    RankPct,
    Zscore,
    Scale,
    Demean,
    Winsor { lower: f64, upper: f64 },
    Neutralize { groups: Vec<String> },
    Quantile { q: f64 },
    Bucket { n: usize },
    Median,
    Mad, // median absolute deviation

    // Portfolio
    LongShort { long_pct: f64, short_pct: f64 },

    // Technical indicators
    Rsi { window: usize },
    Macd { fast: usize, slow: usize, signal: usize },
    Atr { window: usize },
    Vwap,
}

/// A node in the intermediate representation
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IrNode {
    pub id: u64,
    pub operator: Operator,
    pub inputs: Vec<u64>,
    pub type_info: TypeAnnotation,
}

/// Intermediate representation of a compiled signal
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct Ir {
    pub nodes: Vec<IrNode>,
    pub outputs: Vec<u64>,
    pub metadata: IrMetadata,
}

/// Data source declaration
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct DataDecl {
    pub name: String,
    pub path: String,
}

/// Parameter declaration
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct ParamDecl {
    pub name: String,
    pub default_value: f64,
}

/// Metadata for IR provenance
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IrMetadata {
    pub source_hash: String,
    pub compiled_at: u64,
    pub compiler_version: String,
    pub parameters: Vec<ParamDecl>,
    pub data_sources: Vec<DataDecl>,
}

/// Configuration for a backtest run
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct BacktestPlan {
    pub ir: Ir,
    pub start_date: String,
    pub end_date: String,
    pub universe: String,
    pub parameters: HashMap<String, f64>,
}

/// Results from a backtest execution
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct BacktestReport {
    pub plan_hash: String,
    pub executed_at: u64,
    pub metrics: BacktestMetrics,
    /// Daily positions/weights per asset (optional, for detailed analysis)
    #[with(rkyv::with::Skip)]
    pub positions: Option<PositionHistory>,
    /// Daily portfolio returns series
    pub returns_series: Vec<f64>,
    /// Benchmark-relative metrics (if benchmark provided)
    pub benchmark_metrics: Option<BenchmarkMetrics>,
}

/// Performance metrics from a backtest
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub turnover: f64,
    /// Sortino ratio (downside risk adjusted)
    pub sortino_ratio: f64,
    /// Calmar ratio (return / max drawdown)
    pub calmar_ratio: f64,
    /// Win rate (% of positive days)
    pub win_rate: f64,
    /// Average win / average loss
    pub profit_factor: f64,
}

/// Daily position history for export and analysis
#[derive(Debug, Clone)]
pub struct PositionHistory {
    /// Dates for each row
    pub dates: Vec<String>,
    /// Asset names (column headers)
    pub assets: Vec<String>,
    /// Weights matrix: weights[date_idx][asset_idx]
    pub weights: Vec<Vec<f64>>,
    /// Daily returns matrix: returns[date_idx][asset_idx]
    pub asset_returns: Vec<Vec<f64>>,
}

impl PositionHistory {
    /// Create empty position history
    pub fn new(assets: Vec<String>) -> Self {
        PositionHistory {
            dates: Vec::new(),
            assets,
            weights: Vec::new(),
            asset_returns: Vec::new(),
        }
    }

    /// Add a day's positions
    pub fn add_day(&mut self, date: String, weights: Vec<f64>, returns: Vec<f64>) {
        self.dates.push(date);
        self.weights.push(weights);
        self.asset_returns.push(returns);
    }

    /// Export to CSV format
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("date");
        for asset in &self.assets {
            csv.push_str(&format!(",{}_weight,{}_return", asset, asset));
        }
        csv.push('\n');

        // Data rows
        for (i, date) in self.dates.iter().enumerate() {
            csv.push_str(date);
            for j in 0..self.assets.len() {
                let weight = self.weights.get(i).and_then(|w| w.get(j)).unwrap_or(&0.0);
                let ret = self.asset_returns.get(i).and_then(|r| r.get(j)).unwrap_or(&0.0);
                csv.push_str(&format!(",{:.6},{:.6}", weight, ret));
            }
            csv.push('\n');
        }

        csv
    }

    /// Get weights for a specific date
    pub fn weights_on(&self, date: &str) -> Option<&Vec<f64>> {
        self.dates.iter()
            .position(|d| d == date)
            .and_then(|i| self.weights.get(i))
    }
}

/// Benchmark-relative performance metrics
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Jensen's alpha (excess return over CAPM)
    pub alpha: f64,
    /// Beta to benchmark
    pub beta: f64,
    /// Information ratio (active return / tracking error)
    pub information_ratio: f64,
    /// Tracking error (std of active returns)
    pub tracking_error: f64,
    /// Correlation with benchmark
    pub correlation: f64,
    /// Up capture ratio
    pub up_capture: f64,
    /// Down capture ratio
    pub down_capture: f64,
}

impl Default for BenchmarkMetrics {
    fn default() -> Self {
        BenchmarkMetrics {
            alpha: 0.0,
            beta: 1.0,
            information_ratio: 0.0,
            tracking_error: 0.0,
            correlation: 0.0,
            up_capture: 1.0,
            down_capture: 1.0,
        }
    }
}

/// Errors that can occur in sigc
#[derive(Debug, thiserror::Error)]
pub enum SigcError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{}", .0.format())]
    Operator(OperatorError),

    #[error("{}", .0.format())]
    Data(DataError),
}

/// Structured error for operator execution failures
#[derive(Debug, Clone)]
pub struct OperatorError {
    pub operator: String,
    pub node_id: usize,
    pub message: String,
    pub expected_inputs: Option<usize>,
    pub actual_inputs: Option<usize>,
    pub suggestion: Option<String>,
}

impl OperatorError {
    pub fn new(operator: &str, node_id: usize, message: &str) -> Self {
        Self {
            operator: operator.to_string(),
            node_id,
            message: message.to_string(),
            expected_inputs: None,
            actual_inputs: None,
            suggestion: None,
        }
    }

    pub fn input_mismatch(operator: &str, node_id: usize, expected: usize, actual: usize) -> Self {
        let suggestion = if actual == 0 {
            Some(format!("Check that '{}' has input connected in your signal definition", operator.to_lowercase()))
        } else if actual < expected {
            Some(format!("'{}' requires {} inputs but only {} provided. Check your expression.", operator, expected, actual))
        } else {
            Some(format!("'{}' requires {} inputs but {} provided. Remove extra arguments.", operator, expected, actual))
        };

        Self {
            operator: operator.to_string(),
            node_id,
            message: format!("Input count mismatch for '{}'", operator),
            expected_inputs: Some(expected),
            actual_inputs: Some(actual),
            suggestion,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn format(&self) -> String {
        let mut result = format!("Operator '{}' error (node #{}): {}",
            self.operator, self.node_id, self.message);

        if let (Some(expected), Some(actual)) = (self.expected_inputs, self.actual_inputs) {
            result.push_str(&format!("\n  Expected {} input(s), got {}", expected, actual));
        }

        if let Some(ref suggestion) = self.suggestion {
            result.push_str(&format!("\n  Suggestion: {}", suggestion));
        }

        result
    }
}

/// Structured error for data loading/processing failures
#[derive(Debug, Clone)]
pub struct DataError {
    pub source: String,
    pub operation: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl DataError {
    pub fn new(source: &str, operation: &str, message: &str) -> Self {
        Self {
            source: source.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn format(&self) -> String {
        let mut result = format!("Data error in '{}' ({}): {}",
            self.source, self.operation, self.message);

        if let Some(ref suggestion) = self.suggestion {
            result.push_str(&format!("\n  Suggestion: {}", suggestion));
        }

        result
    }
}

/// Format a source error with line context
pub fn format_error_with_context(source: &str, span: std::ops::Range<usize>, message: &str) -> String {
    let mut line_num = 1;
    let mut line_start = 0;
    let mut col: usize = 1;

    // Find line and column for the span start
    for (i, c) in source.char_indices() {
        if i >= span.start {
            break;
        }
        if c == '\n' {
            line_num += 1;
            line_start = i + 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    // Get the problematic line
    let line_end = source[line_start..].find('\n')
        .map(|i| line_start + i)
        .unwrap_or(source.len());
    let line = &source[line_start..line_end];

    // Calculate underline length (at least 1, but follow span length within line)
    let span_len = span.end.saturating_sub(span.start).max(1);
    let underline_len = span_len.min(line.len().saturating_sub(col.saturating_sub(1))).max(1);

    // Build the error message
    let line_num_str = line_num.to_string();
    let padding = " ".repeat(line_num_str.len());

    let mut result = format!("error: {}\n", message);
    result.push_str(&format!("  --> line {}:{}\n", line_num, col));
    result.push_str(&format!("   {} |\n", padding));
    result.push_str(&format!("   {} | {}\n", line_num_str, line));
    result.push_str(&format!("   {} | {}{}\n", padding, " ".repeat(col.saturating_sub(1)), "^".repeat(underline_len)));

    result
}

pub type Result<T> = std::result::Result<T, SigcError>;

/// Trait for data source connectors
pub trait DataSource: Send + Sync {
    fn load(&self, symbol: &str, start: &str, end: &str) -> Result<Vec<u8>>;
}

/// Trait for calendar providers
pub trait CalendarProvider: Send + Sync {
    fn trading_days(&self, start: &str, end: &str) -> Result<Vec<String>>;
}

/// Trait for secrets resolution
pub trait SecretsResolver: Send + Sync {
    fn resolve(&self, key: &str) -> Result<String>;
}
