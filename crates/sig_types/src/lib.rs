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
}

/// Performance metrics from a backtest
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub turnover: f64,
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

    // Build the error message
    let mut result = format!("Error at line {}, column {}: {}\n", line_num, col, message);
    result.push_str(&format!("  {} | {}\n", line_num, line));
    result.push_str(&format!("  {} | {}^\n", " ".repeat(line_num.to_string().len()), " ".repeat(col.saturating_sub(1))));

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
