//! Compiler for sigc DSL
//!
//! Parses .sig files and produces typed IR.

pub mod ast;
pub mod parser;

use chumsky::Parser;
use sig_types::{
    DataDecl as IrDataDecl, DType, Ir, IrMetadata, IrNode, Operator, ParamDecl as IrParamDecl,
    Result, Shape, SigcError, TypeAnnotation,
};
use std::collections::HashMap;

pub use ast::Program;

/// Compiler state and configuration
pub struct Compiler {
    cache: Option<sig_cache::Cache>,
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new() -> Self {
        Compiler { cache: None }
    }

    /// Create a compiler with caching enabled
    pub fn with_cache(cache: sig_cache::Cache) -> Self {
        Compiler { cache: Some(cache) }
    }

    /// Parse source code to AST
    pub fn parse(&self, source: &str) -> Result<Program> {
        let parser = parser::parser();
        parser.parse(source).map_err(|errors| {
            let msg = errors
                .iter()
                .map(|e| {
                    let span = e.span();
                    let expected: Vec<String> = e.expected()
                        .map(|c| match c {
                            Some(c) => format!("'{}'", c),
                            None => "end of input".to_string(),
                        })
                        .collect();
                    let found = e.found()
                        .map(|c| format!("'{}'", c))
                        .unwrap_or_else(|| "end of input".to_string());

                    let detail = if expected.is_empty() {
                        format!("Unexpected {}", found)
                    } else {
                        format!("Expected {}, found {}", expected.join(" or "), found)
                    };

                    sig_types::format_error_with_context(source, span, &detail)
                })
                .collect::<Vec<_>>()
                .join("\n");
            SigcError::Parse(msg)
        })
    }

    /// Compile source code to IR
    pub fn compile(&self, source: &str) -> Result<Ir> {
        tracing::info!("Parsing source");

        // Check cache first
        let source_hash = sig_cache::Cache::hash(source.as_bytes());
        if let Some(ref cache) = self.cache {
            if let Ok(Some(ir)) = cache.get_ir(&source_hash) {
                tracing::info!("Cache hit for source hash {}", source_hash);
                return Ok(ir);
            }
        }

        // Parse to AST
        let program = self.parse(source)?;
        tracing::info!(
            "Parsed {} data, {} params, {} signals, {} portfolios",
            program.data.len(),
            program.params.len(),
            program.signals.len(),
            program.portfolios.len()
        );

        // Type check and lower to IR
        let ir = self.lower_to_ir(&program, &source_hash)?;
        tracing::info!("Lowered to {} IR nodes", ir.nodes.len());

        // Cache the compiled IR
        if let Some(ref cache) = self.cache {
            if let Err(e) = cache.put_ir(&source_hash, &ir) {
                tracing::warn!("Failed to cache IR: {}", e);
            }
        }

        Ok(ir)
    }

    /// Lower AST to IR
    fn lower_to_ir(&self, program: &Program, source_hash: &str) -> Result<Ir> {
        let mut lowering = IrLowering::new();
        lowering.lower_program(program)?;

        // Extract parameters from AST
        let parameters: Vec<IrParamDecl> = program
            .params
            .iter()
            .filter_map(|p| {
                // Extract numeric value from expression
                let value = match &p.node.value {
                    ast::Expr::Number(n) => *n,
                    _ => return None, // Skip non-numeric params for now
                };
                Some(IrParamDecl {
                    name: p.node.name.clone(),
                    default_value: value,
                })
            })
            .collect();

        // Extract data sources from AST
        let data_sources: Vec<IrDataDecl> = program
            .data
            .iter()
            .map(|d| IrDataDecl {
                name: d.node.name.clone(),
                path: d.node.source.clone(),
            })
            .collect();

        Ok(Ir {
            nodes: lowering.nodes,
            outputs: lowering.outputs,
            metadata: IrMetadata {
                source_hash: source_hash.to_string(),
                compiled_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                compiler_version: env!("CARGO_PKG_VERSION").to_string(),
                parameters,
                data_sources,
            },
        })
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// IR lowering state
struct IrLowering {
    nodes: Vec<IrNode>,
    outputs: Vec<u64>,
    next_id: u64,
    symbols: HashMap<String, u64>,
}

impl IrLowering {
    fn new() -> Self {
        IrLowering {
            nodes: Vec::new(),
            outputs: Vec::new(),
            next_id: 0,
            symbols: HashMap::new(),
        }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn lower_program(&mut self, program: &Program) -> Result<()> {
        // Register parameters
        for param in &program.params {
            let id = self.alloc_id();
            self.symbols.insert(param.node.name.clone(), id);
        }

        // Register data sources
        for data in &program.data {
            let id = self.alloc_id();
            self.symbols.insert(data.node.name.clone(), id);
        }

        // Lower signal blocks
        for signal in &program.signals {
            self.lower_signal_block(&signal.node)?;
        }

        Ok(())
    }

    fn lower_signal_block(&mut self, signal: &ast::SignalBlock) -> Result<()> {
        let mut last_emit = None;

        for stmt in &signal.statements {
            match &stmt.node {
                ast::Statement::Assignment { name, value } => {
                    let id = self.lower_expr(value)?;
                    self.symbols.insert(name.clone(), id);
                }
                ast::Statement::Emit(expr) => {
                    let id = self.lower_expr(expr)?;
                    last_emit = Some(id);
                }
            }
        }

        if let Some(id) = last_emit {
            self.symbols.insert(signal.name.clone(), id);
            self.outputs.push(id);
        }

        Ok(())
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> Result<u64> {
        match expr {
            ast::Expr::Number(_) => {
                // Constants don't create IR nodes in this simple model
                Ok(self.alloc_id())
            }

            ast::Expr::Ident(name) => self.symbols.get(name).copied().ok_or_else(|| {
                // Find similar names for suggestion
                let similar: Vec<&String> = self.symbols.keys()
                    .filter(|k| {
                        // Simple similarity: same prefix or contains
                        k.starts_with(&name.chars().take(2).collect::<String>())
                            || name.starts_with(&k.chars().take(2).collect::<String>())
                            || k.contains(name) || name.contains(k.as_str())
                    })
                    .take(3)
                    .collect();

                let msg = if similar.is_empty() {
                    format!("Undefined identifier: '{}'", name)
                } else {
                    format!("Undefined identifier: '{}'. Did you mean: {}?",
                        name,
                        similar.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))
                };
                SigcError::Type(msg)
            }),

            ast::Expr::BinOp { op, left, right } => {
                let left_id = self.lower_expr(&left.node)?;
                let right_id = self.lower_expr(&right.node)?;
                let id = self.alloc_id();

                let operator = match op {
                    ast::BinOp::Add => Operator::Add,
                    ast::BinOp::Sub => Operator::Sub,
                    ast::BinOp::Mul => Operator::Mul,
                    ast::BinOp::Div => Operator::Div,
                };

                self.nodes.push(IrNode {
                    id,
                    operator,
                    inputs: vec![left_id, right_id],
                    type_info: TypeAnnotation {
                        dtype: DType::Float64,
                        shape: Shape::scalar(),
                    },
                });

                Ok(id)
            }

            ast::Expr::UnaryOp { op: _, expr } => {
                let expr_id = self.lower_expr(&expr.node)?;
                let id = self.alloc_id();

                self.nodes.push(IrNode {
                    id,
                    operator: Operator::Mul, // -x = -1 * x
                    inputs: vec![expr_id],
                    type_info: TypeAnnotation {
                        dtype: DType::Float64,
                        shape: Shape::scalar(),
                    },
                });

                Ok(id)
            }

            ast::Expr::Call { func, args, kwargs } => {
                let arg_ids: Vec<u64> = args
                    .iter()
                    .map(|a| self.lower_expr(&a.node))
                    .collect::<Result<_>>()?;

                let id = self.alloc_id();
                let operator = self.func_to_operator(func, &kwargs)?;

                self.nodes.push(IrNode {
                    id,
                    operator,
                    inputs: arg_ids,
                    type_info: TypeAnnotation {
                        dtype: DType::Float64,
                        shape: Shape::scalar(),
                    },
                });

                Ok(id)
            }

            ast::Expr::MethodCall {
                object,
                method,
                args,
                kwargs,
            } => {
                let obj_id = self.lower_expr(&object.node)?;
                let mut arg_ids = vec![obj_id];
                for a in args {
                    arg_ids.push(self.lower_expr(&a.node)?);
                }

                let id = self.alloc_id();
                let operator = self.method_to_operator(method, &kwargs)?;

                self.nodes.push(IrNode {
                    id,
                    operator,
                    inputs: arg_ids,
                    type_info: TypeAnnotation {
                        dtype: DType::Float64,
                        shape: Shape::scalar(),
                    },
                });

                Ok(id)
            }

            ast::Expr::String(_) => Ok(self.alloc_id()),
        }
    }

    fn func_to_operator(
        &self,
        func: &str,
        kwargs: &[(String, ast::Spanned<ast::Expr>)],
    ) -> Result<Operator> {
        match func {
            // Arithmetic
            "abs" => Ok(Operator::Abs),
            "sign" => Ok(Operator::Sign),
            "log" => Ok(Operator::Log),
            "exp" => Ok(Operator::Exp),
            "pow" => {
                let exp = self.get_float_kwarg(kwargs, "exp").unwrap_or(2.0);
                Ok(Operator::Pow { exponent: exp })
            }
            "clip" => {
                let min = self.get_float_kwarg(kwargs, "min").unwrap_or(f64::NEG_INFINITY);
                let max = self.get_float_kwarg(kwargs, "max").unwrap_or(f64::INFINITY);
                Ok(Operator::Clip { min, max })
            }
            "sqrt" => Ok(Operator::Sqrt),
            "floor" => Ok(Operator::Floor),
            "ceil" => Ok(Operator::Ceil),
            "round" => {
                let decimals = self.get_int_kwarg(kwargs, "decimals").unwrap_or(0);
                Ok(Operator::Round { decimals })
            }
            "min" => Ok(Operator::Min),
            "max" => Ok(Operator::Max),

            // Comparison
            "gt" => Ok(Operator::Gt),
            "lt" => Ok(Operator::Lt),
            "ge" => Ok(Operator::Ge),
            "le" => Ok(Operator::Le),
            "eq" => Ok(Operator::Eq),
            "ne" => Ok(Operator::Ne),

            // Logical
            "and" => Ok(Operator::And),
            "or" => Ok(Operator::Or),
            "not" => Ok(Operator::Not),
            "where" | "if_else" => Ok(Operator::Where),

            // Data handling
            "is_nan" | "isnan" => Ok(Operator::IsNan),
            "fill_nan" | "fillna" => {
                let value = self.get_float_kwarg(kwargs, "value").unwrap_or(0.0);
                Ok(Operator::FillNan { value })
            }
            "coalesce" => Ok(Operator::Coalesce),
            "cumsum" => Ok(Operator::Cumsum),
            "cumprod" => Ok(Operator::Cumprod),
            "cummax" => Ok(Operator::Cummax),
            "cummin" => Ok(Operator::Cummin),

            // Time series
            "lag" => {
                let periods = self.get_int_kwarg(kwargs, "periods").unwrap_or(1);
                Ok(Operator::Lag { periods })
            }
            "ret" => {
                let periods = self.get_int_kwarg(kwargs, "periods").unwrap_or(1);
                Ok(Operator::Ret { periods })
            }
            "delta" => {
                let periods = self.get_int_kwarg(kwargs, "periods").unwrap_or(1);
                Ok(Operator::Delta { periods })
            }
            "rolling_mean" | "ts_mean" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::RollingMean { window })
            }
            "rolling_std" | "ts_std" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::RollingStd { window })
            }
            "rolling_sum" | "ts_sum" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::RollingSum { window })
            }
            "rolling_min" | "ts_min" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::RollingMin { window })
            }
            "rolling_max" | "ts_max" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::RollingMax { window })
            }
            "decay_linear" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::DecayLinear { window })
            }
            "ema" => {
                let span = self.get_int_kwarg(kwargs, "span").unwrap_or(20) as usize;
                Ok(Operator::Ema { span })
            }
            "ts_argmax" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsArgmax { window })
            }
            "ts_argmin" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsArgmin { window })
            }
            "ts_rank" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsRank { window })
            }
            "ts_skew" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsSkew { window })
            }
            "ts_kurt" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsKurt { window })
            }
            "ts_product" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsProduct { window })
            }
            "ts_zscore" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(20) as usize;
                Ok(Operator::TsZscore { window })
            }

            // Cross-sectional
            "zscore" => Ok(Operator::Zscore),
            "rank" => Ok(Operator::Rank),
            "rank_pct" => Ok(Operator::RankPct),
            "scale" => Ok(Operator::Scale),
            "demean" => Ok(Operator::Demean),
            "winsor" => {
                let p = self.get_float_kwarg(kwargs, "p").unwrap_or(0.01);
                Ok(Operator::Winsor {
                    lower: p,
                    upper: 1.0 - p,
                })
            }
            "neutralize" => {
                let groups = vec![];
                Ok(Operator::Neutralize { groups })
            }
            "quantile" => {
                let q = self.get_float_kwarg(kwargs, "q").unwrap_or(0.5);
                Ok(Operator::Quantile { q })
            }
            "bucket" | "ntile" => {
                let n = self.get_int_kwarg(kwargs, "n").unwrap_or(5) as usize;
                Ok(Operator::Bucket { n })
            }
            "median" => Ok(Operator::Median),
            "mad" => Ok(Operator::Mad),

            // Aliases
            "as_xs" => Ok(Operator::Demean),

            // Technical indicators
            "rsi" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(14) as usize;
                Ok(Operator::Rsi { window })
            }
            "macd" => {
                let fast = self.get_int_kwarg(kwargs, "fast").unwrap_or(12) as usize;
                let slow = self.get_int_kwarg(kwargs, "slow").unwrap_or(26) as usize;
                let signal = self.get_int_kwarg(kwargs, "signal").unwrap_or(9) as usize;
                Ok(Operator::Macd { fast, slow, signal })
            }
            "atr" => {
                let window = self.get_int_kwarg(kwargs, "window").unwrap_or(14) as usize;
                Ok(Operator::Atr { window })
            }
            "vwap" => Ok(Operator::Vwap),

            _ => {
                // Suggest similar function names
                let known = vec![
                    "abs", "sign", "log", "exp", "pow", "clip", "sqrt", "floor", "ceil", "round",
                    "min", "max", "gt", "lt", "ge", "le", "eq", "ne", "and", "or", "not", "where",
                    "is_nan", "fill_nan", "coalesce", "cumsum", "cumprod", "cummax", "cummin",
                    "lag", "ret", "delta", "rolling_mean", "rolling_std", "rolling_sum",
                    "rolling_min", "rolling_max", "decay_linear", "ema", "ts_argmax", "ts_argmin",
                    "ts_rank", "ts_skew", "ts_kurt", "ts_product", "ts_zscore",
                    "zscore", "rank", "rank_pct", "scale", "demean", "winsor", "neutralize",
                    "quantile", "bucket", "median", "mad", "rsi", "macd", "atr", "vwap",
                ];
                let similar: Vec<&str> = known.iter()
                    .filter(|k| k.contains(func) || func.contains(*k) ||
                        k.starts_with(&func.chars().take(3).collect::<String>()))
                    .take(3)
                    .cloned()
                    .collect();

                let msg = if similar.is_empty() {
                    format!("Unknown function: '{}'", func)
                } else {
                    format!("Unknown function: '{}'. Did you mean: {}?",
                        func,
                        similar.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))
                };
                Err(SigcError::Type(msg))
            }
        }
    }

    fn method_to_operator(
        &self,
        method: &str,
        kwargs: &[(String, ast::Spanned<ast::Expr>)],
    ) -> Result<Operator> {
        match method {
            "long_short" => {
                let top = self.get_float_kwarg(kwargs, "top").unwrap_or(0.2);
                let bottom = self.get_float_kwarg(kwargs, "bottom").unwrap_or(0.2);
                Ok(Operator::LongShort {
                    long_pct: top,
                    short_pct: bottom,
                })
            }
            _ => Err(SigcError::Type(format!("Unknown method: {}", method))),
        }
    }

    fn get_int_kwarg(&self, kwargs: &[(String, ast::Spanned<ast::Expr>)], name: &str) -> Option<i32> {
        kwargs.iter().find_map(|(k, v)| {
            if k == name {
                if let ast::Expr::Number(n) = &v.node {
                    return Some(*n as i32);
                }
            }
            None
        })
    }

    fn get_float_kwarg(
        &self,
        kwargs: &[(String, ast::Spanned<ast::Expr>)],
        name: &str,
    ) -> Option<f64> {
        kwargs.iter().find_map(|(k, v)| {
            if k == name {
                if let ast::Expr::Number(n) = &v.node {
                    return Some(*n);
                }
            }
            None
        })
    }
}
