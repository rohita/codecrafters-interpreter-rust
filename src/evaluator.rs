use std::fmt::Display;
use crate::expr::Expr;

pub struct Evaluator;

pub enum LoxType {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil(),
}

impl Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Boolean(b) => f.write_fmt(format_args!("{b}")),
            LoxType::Nil() => f.write_str("nil"),
            LoxType::Number(n) => f.write_fmt(format_args!("{n}")),
            LoxType::String(s) => f.write_fmt(format_args!("{s}")),
        }
    }
}

impl Evaluator {
    pub fn evaluate(expression: Expr) -> LoxType {
        match expression {
            Expr::Literal(value) => value,
            Expr::Grouping(e) => Evaluator::evaluate(*e),
            _ => LoxType::Nil(),
        }
    }
}