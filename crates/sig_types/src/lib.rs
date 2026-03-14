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

// ============================================================================
// Type Inference System
// ============================================================================

/// Type category for sigc expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCategory {
    /// Scalar numeric value (single number)
    Scalar,
    /// Time series (vector indexed by time)
    TimeSeries,
    /// Cross-sectional data (vector indexed by asset)
    CrossSection,
    /// Panel data (matrix: time x asset)
    Panel,
    /// Boolean condition
    Boolean,
    /// Unknown/unresolved type
    Unknown,
}

/// Inferred type information for an expression
#[derive(Debug, Clone, PartialEq)]
pub struct InferredType {
    pub dtype: DType,
    pub category: TypeCategory,
}

impl InferredType {
    pub fn scalar(dtype: DType) -> Self {
        Self { dtype, category: TypeCategory::Scalar }
    }

    pub fn time_series(dtype: DType) -> Self {
        Self { dtype, category: TypeCategory::TimeSeries }
    }

    pub fn cross_section(dtype: DType) -> Self {
        Self { dtype, category: TypeCategory::CrossSection }
    }

    pub fn panel(dtype: DType) -> Self {
        Self { dtype, category: TypeCategory::Panel }
    }

    pub fn boolean() -> Self {
        Self { dtype: DType::Bool, category: TypeCategory::Boolean }
    }

    pub fn numeric() -> Self {
        Self { dtype: DType::Float64, category: TypeCategory::Panel }
    }

    pub fn unknown() -> Self {
        Self { dtype: DType::Float64, category: TypeCategory::Unknown }
    }
}

/// Operator type signature for type checking
#[derive(Debug, Clone)]
pub struct OpSignature {
    /// Number of required inputs
    pub arity: OpArity,
    /// Input type requirements
    pub input_types: Vec<TypeRequirement>,
    /// Output type (derived from inputs)
    pub output_type: OutputType,
    /// Human-readable description
    pub description: &'static str,
}

/// Operator arity (number of arguments)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpArity {
    /// Exactly N arguments
    Exact(usize),
    /// At least N arguments
    AtLeast(usize),
    /// Range of arguments (min, max)
    Range(usize, usize),
}

impl OpArity {
    pub fn check(&self, count: usize) -> bool {
        match self {
            OpArity::Exact(n) => count == *n,
            OpArity::AtLeast(n) => count >= *n,
            OpArity::Range(min, max) => count >= *min && count <= *max,
        }
    }

    pub fn expected_str(&self) -> String {
        match self {
            OpArity::Exact(n) => format!("{}", n),
            OpArity::AtLeast(n) => format!("at least {}", n),
            OpArity::Range(min, max) => format!("{}-{}", min, max),
        }
    }
}

/// Type requirement for an input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRequirement {
    /// Any numeric type
    Numeric,
    /// Must be boolean
    Boolean,
    /// Must be time series
    TimeSeries,
    /// Must be cross-sectional
    CrossSection,
    /// Must be panel data
    Panel,
    /// Any type (passthrough)
    Any,
}

impl TypeRequirement {
    pub fn check(&self, inferred: &InferredType) -> bool {
        match self {
            TypeRequirement::Numeric => matches!(inferred.dtype, DType::Float32 | DType::Float64 | DType::Int32 | DType::Int64),
            TypeRequirement::Boolean => inferred.dtype == DType::Bool,
            TypeRequirement::TimeSeries => inferred.category == TypeCategory::TimeSeries,
            TypeRequirement::CrossSection => inferred.category == TypeCategory::CrossSection,
            TypeRequirement::Panel => inferred.category == TypeCategory::Panel,
            TypeRequirement::Any => true,
        }
    }
}

/// How to derive output type from inputs
#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    /// Same as first input
    SameAsFirst,
    /// Always boolean
    OutBoolean,
    /// Always numeric (Float64)
    OutNumeric,
    /// Time series output
    OutTimeSeries,
    /// Cross-sectional output
    OutCrossSection,
    /// Panel output
    OutPanel,
}

impl Operator {
    /// Get the type signature for this operator
    pub fn signature(&self) -> OpSignature {
        use OpArity::*;
        use TypeRequirement as TR;
        use OutputType as OT;

        match self {
            // Arithmetic (binary)
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Numeric, TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Binary arithmetic operation",
            },

            // Arithmetic (unary)
            Operator::Abs | Operator::Sign | Operator::Log | Operator::Exp |
            Operator::Sqrt | Operator::Floor | Operator::Ceil => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Unary arithmetic operation",
            },

            Operator::Pow { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Raise to power",
            },

            Operator::Clip { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Clip values to range",
            },

            Operator::Round { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Round to decimals",
            },

            Operator::Min | Operator::Max => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Numeric, TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Element-wise min/max",
            },

            // Comparison
            Operator::Gt | Operator::Lt | Operator::Ge | Operator::Le |
            Operator::Eq | Operator::Ne => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Any, TR::Any],
                output_type: OT::OutBoolean,
                description: "Comparison operation",
            },

            // Logical
            Operator::And | Operator::Or => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Boolean, TR::Boolean],
                output_type: OT::OutBoolean,
                description: "Logical operation",
            },

            Operator::Not => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Boolean],
                output_type: OT::OutBoolean,
                description: "Logical negation",
            },

            Operator::Where => OpSignature {
                arity: Exact(3),
                input_types: vec![TR::Boolean, TR::Any, TR::Any],
                output_type: OT::SameAsFirst,
                description: "Conditional select: where(cond, if_true, if_false)",
            },

            // Data handling
            Operator::IsNan => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutBoolean,
                description: "Check for NaN values",
            },

            Operator::FillNan { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::SameAsFirst,
                description: "Fill NaN with value",
            },

            Operator::Coalesce => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Any, TR::Any],
                output_type: OT::SameAsFirst,
                description: "Return first non-NaN value",
            },

            Operator::Cumsum | Operator::Cumprod | Operator::Cummax | Operator::Cummin => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Cumulative operation",
            },

            // Time series (unary with window)
            Operator::Lag { .. } | Operator::Ret { .. } | Operator::Delta { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Time series shift/return",
            },

            Operator::RollingMean { .. } | Operator::RollingStd { .. } |
            Operator::RollingSum { .. } | Operator::RollingMin { .. } |
            Operator::RollingMax { .. } | Operator::DecayLinear { .. } |
            Operator::Ema { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Rolling window operation",
            },

            Operator::RollingCorr { .. } => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Numeric, TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Rolling correlation",
            },

            Operator::TsArgmax { .. } | Operator::TsArgmin { .. } |
            Operator::TsRank { .. } | Operator::TsSkew { .. } |
            Operator::TsKurt { .. } | Operator::TsProduct { .. } |
            Operator::TsZscore { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Time series statistic",
            },

            // Cross-sectional
            Operator::Rank | Operator::RankPct | Operator::Zscore |
            Operator::Scale | Operator::Demean | Operator::Median |
            Operator::Mad => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Cross-sectional operation",
            },

            Operator::Winsor { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Winsorize at percentiles",
            },

            Operator::Neutralize { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Group neutralization",
            },

            Operator::Quantile { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Quantile value",
            },

            Operator::Bucket { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Assign to buckets",
            },

            // Portfolio
            Operator::LongShort { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutCrossSection,
                description: "Long-short portfolio weights",
            },

            // Technical indicators
            Operator::Rsi { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Relative Strength Index",
            },

            Operator::Macd { .. } => OpSignature {
                arity: Exact(1),
                input_types: vec![TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "MACD indicator",
            },

            Operator::Atr { .. } => OpSignature {
                arity: Exact(3),
                input_types: vec![TR::Numeric, TR::Numeric, TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Average True Range (high, low, close)",
            },

            Operator::Vwap => OpSignature {
                arity: Exact(2),
                input_types: vec![TR::Numeric, TR::Numeric],
                output_type: OT::OutTimeSeries,
                description: "Volume Weighted Average Price (price, volume)",
            },
        }
    }

    /// Check if the given number of inputs is valid for this operator
    pub fn check_arity(&self, input_count: usize) -> std::result::Result<(), String> {
        let sig = self.signature();
        if sig.arity.check(input_count) {
            Ok(())
        } else {
            Err(format!(
                "{} requires {} argument(s), got {}",
                self.name(),
                sig.arity.expected_str(),
                input_count
            ))
        }
    }

    /// Get a human-readable name for the operator
    pub fn name(&self) -> &'static str {
        match self {
            Operator::Add => "add",
            Operator::Sub => "sub",
            Operator::Mul => "mul",
            Operator::Div => "div",
            Operator::Abs => "abs",
            Operator::Sign => "sign",
            Operator::Log => "log",
            Operator::Exp => "exp",
            Operator::Pow { .. } => "pow",
            Operator::Clip { .. } => "clip",
            Operator::Sqrt => "sqrt",
            Operator::Floor => "floor",
            Operator::Ceil => "ceil",
            Operator::Round { .. } => "round",
            Operator::Min => "min",
            Operator::Max => "max",
            Operator::Gt => "gt",
            Operator::Lt => "lt",
            Operator::Ge => "ge",
            Operator::Le => "le",
            Operator::Eq => "eq",
            Operator::Ne => "ne",
            Operator::And => "and",
            Operator::Or => "or",
            Operator::Not => "not",
            Operator::Where => "where",
            Operator::IsNan => "is_nan",
            Operator::FillNan { .. } => "fill_nan",
            Operator::Coalesce => "coalesce",
            Operator::Cumsum => "cumsum",
            Operator::Cumprod => "cumprod",
            Operator::Cummax => "cummax",
            Operator::Cummin => "cummin",
            Operator::Lag { .. } => "lag",
            Operator::Ret { .. } => "ret",
            Operator::Delta { .. } => "delta",
            Operator::RollingMean { .. } => "rolling_mean",
            Operator::RollingStd { .. } => "rolling_std",
            Operator::RollingSum { .. } => "rolling_sum",
            Operator::RollingMin { .. } => "rolling_min",
            Operator::RollingMax { .. } => "rolling_max",
            Operator::RollingCorr { .. } => "rolling_corr",
            Operator::DecayLinear { .. } => "decay_linear",
            Operator::Ema { .. } => "ema",
            Operator::TsArgmax { .. } => "ts_argmax",
            Operator::TsArgmin { .. } => "ts_argmin",
            Operator::TsRank { .. } => "ts_rank",
            Operator::TsSkew { .. } => "ts_skew",
            Operator::TsKurt { .. } => "ts_kurt",
            Operator::TsProduct { .. } => "ts_product",
            Operator::TsZscore { .. } => "ts_zscore",
            Operator::Rank => "rank",
            Operator::RankPct => "rank_pct",
            Operator::Zscore => "zscore",
            Operator::Scale => "scale",
            Operator::Demean => "demean",
            Operator::Winsor { .. } => "winsor",
            Operator::Neutralize { .. } => "neutralize",
            Operator::Quantile { .. } => "quantile",
            Operator::Bucket { .. } => "bucket",
            Operator::Median => "median",
            Operator::Mad => "mad",
            Operator::LongShort { .. } => "long_short",
            Operator::Rsi { .. } => "rsi",
            Operator::Macd { .. } => "macd",
            Operator::Atr { .. } => "atr",
            Operator::Vwap => "vwap",
        }
    }
}

/// Type error with location information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub operator: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub suggestion: Option<String>,
}

impl TypeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            operator: None,
            expected: None,
            actual: None,
            suggestion: None,
        }
    }

    pub fn arity_mismatch(op: &str, expected: &str, actual: usize) -> Self {
        Self {
            message: format!("Wrong number of arguments for '{}'", op),
            operator: Some(op.to_string()),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
            suggestion: Some(format!("'{}' requires {} argument(s)", op, expected)),
        }
    }

    pub fn type_mismatch(op: &str, position: usize, expected: &str, actual: &str) -> Self {
        Self {
            message: format!("Type mismatch in argument {} of '{}'", position + 1, op),
            operator: Some(op.to_string()),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
            suggestion: None,
        }
    }

    pub fn format(&self) -> String {
        let mut result = format!("Type error: {}", self.message);
        if let (Some(exp), Some(act)) = (&self.expected, &self.actual) {
            result.push_str(&format!("\n  Expected: {}", exp));
            result.push_str(&format!("\n  Got: {}", act));
        }
        if let Some(ref sug) = self.suggestion {
            result.push_str(&format!("\n  Suggestion: {}", sug));
        }
        result
    }
}

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

// ============================================================================
// Built-in Macro Patterns
// ============================================================================

/// Built-in macro definitions that can be used in sigc programs
pub struct BuiltinMacros;

impl BuiltinMacros {
    /// Get the source code for all built-in macros
    pub fn all() -> &'static str {
        r#"
// Momentum signal pattern
// Usage: momentum!(px, lookback=20)
macro momentum(prices: expr, lookback: number = 20):
  let r = ret(prices, lookback)
  emit zscore(r)

// Mean reversion signal pattern
// Usage: mean_reversion!(px, window=20, threshold=2.0)
macro mean_reversion(prices: expr, window: number = 20):
  let ma = rolling_mean(prices, window)
  let std = rolling_std(prices, window)
  let z = (prices - ma) / std
  emit -z

// Volatility-adjusted momentum
// Usage: vol_adj_momentum!(px, ret_window=20, vol_window=60)
macro vol_adj_momentum(prices: expr, ret_window: number = 20, vol_window: number = 60):
  let r = ret(prices, ret_window)
  let vol = rolling_std(r, vol_window)
  emit zscore(r / vol)

// Trend following signal
// Usage: trend!(px, fast=10, slow=50)
macro trend(prices: expr, fast: number = 10, slow: number = 50):
  let fast_ma = ema(prices, fast)
  let slow_ma = ema(prices, slow)
  emit zscore(fast_ma - slow_ma)

// RSI-based signal
// Usage: rsi_signal!(px, period=14, overbought=70, oversold=30)
macro rsi_signal(prices: expr, period: number = 14):
  let r = rsi(prices, period)
  emit zscore(50 - r)

// Breakout signal
// Usage: breakout!(px, window=20)
macro breakout(prices: expr, window: number = 20):
  let high = rolling_max(prices, window)
  let low = rolling_min(prices, window)
  let range = high - low
  let pos = (prices - low) / range
  emit zscore(pos - 0.5)

// Cross-sectional momentum
// Usage: cs_momentum!(px, lookback=252)
macro cs_momentum(prices: expr, lookback: number = 252):
  let r = ret(prices, lookback)
  emit rank(r)

// Quality filter (low volatility)
// Usage: quality!(px, window=60)
macro quality(prices: expr, window: number = 60):
  let r = ret(prices, 1)
  let vol = rolling_std(r, window)
  emit -rank(vol)
"#
    }

    /// Get individual macro names and descriptions
    pub fn list() -> Vec<(&'static str, &'static str)> {
        vec![
            ("momentum", "Time-series momentum signal with z-score normalization"),
            ("mean_reversion", "Mean reversion signal based on deviation from moving average"),
            ("vol_adj_momentum", "Volatility-adjusted momentum (risk-parity style)"),
            ("trend", "Trend following based on EMA crossover"),
            ("rsi_signal", "RSI-based contrarian signal"),
            ("breakout", "Channel breakout signal"),
            ("cs_momentum", "Cross-sectional momentum (relative returns)"),
            ("quality", "Quality/low-volatility factor"),
        ]
    }
}
