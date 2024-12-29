use crate::evaluator::LoxType;
use crate::token::Token;
use std::fmt::Display;

pub enum Expr {
    Literal(LoxType),
    Unary { operator: Token, right: Box<Expr> },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(t) => match t {
                LoxType::Number(n) => f.write_fmt(format_args!("{n:?}")),
                _ => f.write_fmt(format_args!("{t}")),
            },
            Expr::Unary { operator, right } => {
                f.write_fmt(format_args!("({} {right})", operator.lexeme))
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => f.write_fmt(format_args!("({} {left} {right})", operator.lexeme)),
            Expr::Grouping(expression) => f.write_fmt(format_args!("(group {})", expression)),
        }
    }
}