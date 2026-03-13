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
        .then(signal_block().repeated())
        .then(portfolio_block().repeated())
        .then_ignore(blank_lines())
        .then_ignore(end())
        .map(|(((data, params), signals), portfolios)| Program {
            data,
            params: params.unwrap_or_default(),
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
}
