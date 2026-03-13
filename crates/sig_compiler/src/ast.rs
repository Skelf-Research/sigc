//! Abstract Syntax Tree definitions for the sigc DSL

use std::ops::Range;

/// Source span for error reporting
pub type Span = Range<usize>;

/// A spanned value
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }
}

/// Top-level program structure
#[derive(Debug, Clone)]
pub struct Program {
    pub data: Vec<Spanned<DataDecl>>,
    pub params: Vec<Spanned<ParamDecl>>,
    pub signals: Vec<Spanned<SignalBlock>>,
    pub portfolios: Vec<Spanned<PortfolioBlock>>,
}

/// Data loading declaration
#[derive(Debug, Clone)]
pub struct DataDecl {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub options: Vec<(String, Expr)>,
}

/// Parameter declaration
#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub name: String,
    pub value: Expr,
}

/// Signal block definition
#[derive(Debug, Clone)]
pub struct SignalBlock {
    pub name: String,
    pub statements: Vec<Spanned<Statement>>,
}

/// Portfolio block definition
#[derive(Debug, Clone)]
pub struct PortfolioBlock {
    pub name: String,
    pub weights: Spanned<Expr>,
    pub costs: Option<Spanned<Expr>>,
    pub backtest: Option<Spanned<BacktestConfig>>,
}

/// Backtest configuration
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub rebal: String,
    pub benchmark: Option<String>,
    pub from_date: String,
    pub to_date: String,
}

/// Statement in a signal block
#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { name: String, value: Expr },
    Emit(Expr),
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expr {
    /// Numeric literal
    Number(f64),
    /// String literal
    String(String),
    /// Identifier
    Ident(String),
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Spanned<Expr>>,
        right: Box<Spanned<Expr>>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        expr: Box<Spanned<Expr>>,
    },
    /// Function call
    Call {
        func: String,
        args: Vec<Spanned<Expr>>,
        kwargs: Vec<(String, Spanned<Expr>)>,
    },
    /// Method call
    MethodCall {
        object: Box<Spanned<Expr>>,
        method: String,
        args: Vec<Spanned<Expr>>,
        kwargs: Vec<(String, Spanned<Expr>)>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
}
