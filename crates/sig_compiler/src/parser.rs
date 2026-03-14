//! Parser for the sigc DSL using chumsky

use chumsky::prelude::*;
use crate::ast::*;

/// Either type for distinguishing positional vs keyword args
enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Parse a complete sigc program
pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let program = blank_lines()
        .ignore_then(data_section())
        .then(params_section().or_not())
        .then(macro_def().repeated())
        .then(function_def().repeated())
        .then(signal_block().repeated())
        .then(portfolio_block().repeated())
        .then_ignore(blank_lines())
        .then_ignore(end())
        .map(|(((((data, params), macros), functions), signals), portfolios)| Program {
            data,
            params: params.unwrap_or_default(),
            macros,
            functions,
            signals,
            portfolios,
        });

    program
}

fn ws() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    filter(|c: &char| *c == ' ' || *c == '\t')
        .repeated()
        .ignored()
}

fn comment() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    just("//")
        .then(filter(|c: &char| *c != '\n').repeated())
        .ignored()
}

fn nl() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    just('\n').ignored()
}

fn blank_lines() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    ws()
        .then(comment().or_not())
        .then(nl())
        .repeated()
        .ignored()
}

fn indent() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    just(' ').then(just(' ')).ignored()
}

fn ident() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
        .then(filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_').repeated())
        .map(|(first, rest)| {
            let mut s = String::new();
            s.push(first);
            s.extend(rest);
            s
        })
}

fn number() -> impl Parser<char, f64, Error = Simple<char>> + Clone {
    let digits = filter(|c: &char| c.is_ascii_digit()).repeated().at_least(1);

    just('-')
        .or_not()
        .then(digits.clone())
        .then(just('.').then(digits).or_not())
        .map(|((neg, int), frac)| {
            let mut s = String::new();
            if neg.is_some() {
                s.push('-');
            }
            s.extend(int);
            if let Some((_, f)) = frac {
                s.push('.');
                s.extend(f);
            }
            s.parse().unwrap_or(0.0)
        })
}

fn string_literal() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
}

#[allow(dead_code)]
fn date_str() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    filter(|c: &char| c.is_ascii_digit() || *c == '-')
        .repeated()
        .at_least(10)
        .at_most(10)
        .collect::<String>()
}

fn expr() -> impl Parser<char, Spanned<Expr>, Error = Simple<char>> + Clone {
    recursive(|expr| {
        let literal = number()
            .map(Expr::Number)
            .or(string_literal().map(Expr::String))
            .map_with_span(Spanned::new);

        let ident_expr = ident()
            .map(Expr::Ident)
            .map_with_span(Spanned::new);

        // Keyword argument: name=expr
        let kwarg = ident()
            .then_ignore(just('='))
            .then(expr.clone())
            .map(|(k, v)| (k, v));

        // Positional or keyword argument
        let arg = kwarg.clone().map(Either::Right)
            .or(expr.clone().map(Either::Left));

        // Argument list (positional and keyword)
        let args_list = arg
            .separated_by(just(',').padded())
            .allow_trailing()
            .map(|args| {
                let mut positional = vec![];
                let mut keyword = vec![];
                for a in args {
                    match a {
                        Either::Left(e) => positional.push(e),
                        Either::Right((k, v)) => keyword.push((k, v)),
                    }
                }
                (positional, keyword)
            });

        // Function call
        let call = ident()
            .then(
                args_list.clone()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(func, (args, kwargs))| Expr::Call {
                func,
                args,
                kwargs,
            })
            .map_with_span(Spanned::new);

        // Parenthesized expression
        let paren = expr
            .clone()
            .delimited_by(just('('), just(')'));

        // Atom: call, literal, ident, or parenthesized
        let atom = call
            .or(literal)
            .or(ident_expr)
            .or(paren);

        // Method call: atom.method(args)
        let with_methods = atom.then(
            just('.')
                .ignore_then(ident())
                .then(args_list.delimited_by(just('('), just(')')))
                .repeated()
        ).map_with_span(|(base, methods), span| {
            let mut result = base;
            for (method, (args, kwargs)) in methods {
                result = Spanned::new(
                    Expr::MethodCall {
                        object: Box::new(result),
                        method,
                        args,
                        kwargs,
                    },
                    span.clone(),
                );
            }
            result
        });

        // Unary minus
        let unary = just('-')
            .repeated()
            .then(with_methods)
            .map_with_span(|(negs, expr), span| {
                let mut result = expr;
                for _ in negs {
                    result = Spanned::new(
                        Expr::UnaryOp {
                            op: UnaryOp::Neg,
                            expr: Box::new(result),
                        },
                        span.clone(),
                    );
                }
                result
            });

        // Binary operations with proper precedence
        let product = unary.clone().then(
            choice((just('*').to(BinOp::Mul), just('/').to(BinOp::Div)))
                .padded()
                .then(unary)
                .repeated()
        ).map_with_span(|(first, rest), span| {
            let mut result = first;
            for (op, right) in rest {
                result = Spanned::new(
                    Expr::BinOp {
                        op,
                        left: Box::new(result),
                        right: Box::new(right),
                    },
                    span.clone(),
                );
            }
            result
        });

        product.clone().then(
            choice((just('+').to(BinOp::Add), just('-').to(BinOp::Sub)))
                .padded()
                .then(product)
                .repeated()
        ).map_with_span(|(first, rest), span| {
            let mut result = first;
            for (op, right) in rest {
                result = Spanned::new(
                    Expr::BinOp {
                        op,
                        left: Box::new(result),
                        right: Box::new(right),
                    },
                    span.clone(),
                );
            }
            result
        })
    })
}

fn data_section() -> impl Parser<char, Vec<Spanned<DataDecl>>, Error = Simple<char>> {
    let option = ident()
        .then_ignore(just('='))
        .then(
            ident().map(Expr::Ident)
                .or(string_literal().map(Expr::String))
                .or(number().map(Expr::Number)),
        );

    let data_line = indent()
        .ignore_then(ident())
        .then_ignore(just(':').then(ws()))
        .then_ignore(just("load").then(ws()))
        .then(ident())
        .then_ignore(ws().then(just("from")).then(ws()))
        .then(string_literal())
        .then(ws().ignore_then(option).repeated())
        .then_ignore(nl())
        .map(|(((name, kind), source), options)| DataDecl {
            name,
            kind,
            source,
            options,
        })
        .map_with_span(Spanned::new);

    just("data:")
        .then_ignore(nl())
        .ignore_then(data_line.repeated())
        .then_ignore(blank_lines())
}

fn params_section() -> impl Parser<char, Vec<Spanned<ParamDecl>>, Error = Simple<char>> {
    let param_line = indent()
        .ignore_then(ident())
        .then_ignore(ws().then(just('=')).then(ws()))
        .then(number())
        .then_ignore(nl())
        .map(|(name, value)| ParamDecl {
            name,
            value: Expr::Number(value),
        })
        .map_with_span(Spanned::new);

    just("params:")
        .then_ignore(nl())
        .ignore_then(param_line.repeated())
        .then_ignore(blank_lines())
}

fn statement() -> impl Parser<char, Spanned<Statement>, Error = Simple<char>> {
    let assignment = indent()
        .ignore_then(ident())
        .then_ignore(ws().then(just('=')).then(ws()))
        .then(expr())
        .then_ignore(nl())
        .map(|(name, value)| Statement::Assignment {
            name,
            value: value.node,
        })
        .map_with_span(Spanned::new);

    let emit = indent()
        .ignore_then(just("emit").then(ws()))
        .ignore_then(expr())
        .then_ignore(nl())
        .map(|e| Statement::Emit(e.node))
        .map_with_span(Spanned::new);

    emit.or(assignment)
}

/// Parse a macro definition
/// Syntax: macro name(param: type, ...):
///           let x = expr
///           emit expr
fn macro_def() -> impl Parser<char, Spanned<MacroDef>, Error = Simple<char>> {
    // Parameter type annotation
    let param_kind = just("expr").to(MacroParamKind::Expr)
        .or(just("number").to(MacroParamKind::Number))
        .or(just("string").to(MacroParamKind::String))
        .or(just("ident").to(MacroParamKind::Ident));

    // Macro parameter: name: type or name: type = default
    let macro_param = ident()
        .then_ignore(just(':').padded())
        .then(param_kind)
        .then(
            just('=')
                .padded()
                .ignore_then(
                    number().map(MacroValue::Number)
                        .or(string_literal().map(MacroValue::String))
                        .or(ident().map(MacroValue::Ident))
                )
                .or_not()
        )
        .map(|((name, kind), default)| MacroParam { name, kind, default });

    // Parameter list
    let param_list = macro_param
        .separated_by(just(',').padded())
        .allow_trailing()
        .delimited_by(just('('), just(')'));

    // Macro statement: let x = expr or emit expr
    let macro_let = indent()
        .ignore_then(just("let "))
        .ignore_then(ident())
        .then_ignore(ws().then(just('=')).then(ws()))
        .then(expr())
        .then_ignore(nl())
        .map(|(name, value)| MacroStatement::Let { name, value: value.node })
        .map_with_span(Spanned::new);

    let macro_emit = indent()
        .ignore_then(just("emit "))
        .ignore_then(expr())
        .then_ignore(nl())
        .map(|e| MacroStatement::Emit(e.node))
        .map_with_span(Spanned::new);

    let macro_stmt = macro_let.or(macro_emit);

    // macro name(params):
    //   body
    just("macro ")
        .ignore_then(ident())
        .then(param_list)
        .then_ignore(just(':').then(nl()))
        .then(macro_stmt.repeated().at_least(1))
        .then_ignore(blank_lines())
        .map(|((name, params), body)| MacroDef { name, params, body })
        .map_with_span(Spanned::new)
}

fn function_def() -> impl Parser<char, Spanned<FunctionDef>, Error = Simple<char>> {
    // Function parameter: name or name=default
    let func_param = ident()
        .then(
            just('=')
                .ignore_then(
                    number().map(Expr::Number)
                        .or(string_literal().map(Expr::String))
                        .or(ident().map(Expr::Ident))
                )
                .or_not()
        )
        .map(|(name, default)| FunctionParam { name, default });

    // Parameter list
    let param_list = func_param
        .separated_by(just(',').padded())
        .allow_trailing()
        .delimited_by(just('('), just(')'));

    // fn name(params): body
    just("fn ")
        .ignore_then(ident())
        .then(param_list)
        .then_ignore(just(':').then(nl()))
        .then(indent().ignore_then(expr()).then_ignore(nl()))
        .then_ignore(blank_lines())
        .map(|((name, params), body)| FunctionDef { name, params, body })
        .map_with_span(Spanned::new)
}

fn signal_block() -> impl Parser<char, Spanned<SignalBlock>, Error = Simple<char>> {
    just("signal ")
        .ignore_then(ident())
        .then_ignore(just(':').then(nl()))
        .then(statement().repeated().at_least(1))
        .then_ignore(blank_lines())
        .map(|(name, statements)| SignalBlock { name, statements })
        .map_with_span(Spanned::new)
}

fn portfolio_block() -> impl Parser<char, Spanned<PortfolioBlock>, Error = Simple<char>> {
    let weights_line = indent()
        .ignore_then(just("weights"))
        .ignore_then(ws().then(just('=')).then(ws()))
        .ignore_then(expr())
        .then_ignore(nl());

    let costs_line = indent()
        .ignore_then(just("costs"))
        .ignore_then(ws().then(just('=')).then(ws()))
        .ignore_then(expr())
        .then_ignore(nl());

    // Simplified backtest line
    let backtest_line = indent()
        .ignore_then(just("backtest"))
        .ignore_then(take_until(nl()))
        .map_with_span(|(chars, _), span| {
            let line: String = chars.into_iter().collect();
            // Parse the backtest line manually
            let mut rebal = String::new();
            let mut benchmark = None;
            let mut from_date = String::new();
            let mut to_date = String::new();

            let parts: Vec<&str> = line.split_whitespace().collect();
            for i in 0..parts.len() {
                if parts[i].starts_with("rebal=") {
                    rebal = parts[i][6..].to_string();
                } else if parts[i].starts_with("benchmark=") {
                    benchmark = Some(parts[i][10..].to_string());
                } else if parts[i] == "from" && i + 1 < parts.len() {
                    from_date = parts[i + 1].to_string();
                } else if parts[i] == "to" && i + 1 < parts.len() {
                    to_date = parts[i + 1].to_string();
                }
            }

            Spanned::new(
                BacktestConfig {
                    rebal,
                    benchmark,
                    from_date,
                    to_date,
                },
                span,
            )
        });

    just("portfolio ")
        .ignore_then(ident())
        .then_ignore(just(':').then(nl()))
        .then(weights_line)
        .then(costs_line.or_not())
        .then(backtest_line.or_not())
        .then_ignore(blank_lines())
        .map(|(((name, weights), costs), backtest)| PortfolioBlock {
            name,
            weights,
            costs,
            backtest,
        })
        .map_with_span(Spanned::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = number().parse("42").unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_parse_expr() {
        let result = expr().parse("x + y").unwrap();
        assert!(matches!(result.node, Expr::BinOp { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_parse_function_def() {
        let source = r#"data:
  prices: load csv from "data.csv"

fn momentum(x, window=20):
  x.ret(periods=1).rolling_mean(window=window)

signal main:
  result = momentum(prices)
  emit result

portfolio test:
  weights = main
"#;
        let result = parser().parse(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let program = result.unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].node.name, "momentum");
        assert_eq!(program.functions[0].node.params.len(), 2);
        assert_eq!(program.functions[0].node.params[0].name, "x");
        assert_eq!(program.functions[0].node.params[1].name, "window");
        assert!(program.functions[0].node.params[1].default.is_some());
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = r#"data:
  prices: load csv from "data.csv"

fn momentum(x, window=20):
  x.ret(periods=1).rolling_mean(window=window)

fn volatility(x, window=20):
  x.ret(periods=1).rolling_std(window=window)

signal main:
  emit momentum(prices) / volatility(prices)

portfolio test:
  weights = main
"#;
        let result = parser().parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.functions.len(), 2);
    }

    #[test]
    fn test_parse_macro_def() {
        let source = r#"data:
  prices: load csv from "data.csv"

macro momentum_signal(px: expr, lookback: number = 20):
  let r = ret(px, lookback)
  emit zscore(r)

signal main:
  emit prices

portfolio test:
  weights = main
"#;
        let result = parser().parse(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let program = result.unwrap();
        assert_eq!(program.macros.len(), 1);
        assert_eq!(program.macros[0].node.name, "momentum_signal");
        assert_eq!(program.macros[0].node.params.len(), 2);
        assert_eq!(program.macros[0].node.params[0].name, "px");
        assert_eq!(program.macros[0].node.params[0].kind, MacroParamKind::Expr);
        assert_eq!(program.macros[0].node.params[1].name, "lookback");
        assert_eq!(program.macros[0].node.params[1].kind, MacroParamKind::Number);
        assert!(program.macros[0].node.params[1].default.is_some());
        assert_eq!(program.macros[0].node.body.len(), 2);
    }

    #[test]
    fn test_parse_macro_with_multiple_params() {
        let source = r#"data:
  prices: load csv from "data.csv"

macro trend_signal(px: expr, fast: number = 10, slow: number = 50):
  let fast_ma = ema(px, fast)
  let slow_ma = ema(px, slow)
  emit zscore(fast_ma - slow_ma)

signal main:
  emit prices

portfolio test:
  weights = main
"#;
        let result = parser().parse(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let program = result.unwrap();
        assert_eq!(program.macros.len(), 1);
        assert_eq!(program.macros[0].node.params.len(), 3);
        assert_eq!(program.macros[0].node.body.len(), 3);
    }
}
