use std::fmt::Display;
use crate::expr::Expr;
use crate::token::TokenType;

pub struct Evaluator;

pub enum Value {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Value::Nil => f.write_str("nil"),
            Value::Number(n) => f.write_fmt(format_args!("{n}")),
            Value::String(s) => f.write_fmt(format_args!("{s}")),
        }
    }
}

impl Evaluator {
    pub fn evaluate(expression: Expr) -> Value {
        match expression {
            Expr::Literal(value) => value,
            Expr::Grouping(e) => Evaluator::evaluate(*e),
            Expr::Unary{operator, right} => {
                let value = Evaluator::evaluate(*right);
                match operator.token_type {
                    TokenType::MINUS => match value {
                        Value::Number(n) => Value::Number(-n),
                        _ => unreachable!(),
                    },
                    TokenType::BANG => match value {
                        Value::Boolean(b) => Value::Boolean(!b),
                        Value::Nil => Value::Boolean(true),
                        Value::Number(n) => Value::Boolean(n == 0.0),
                        Value::String(s) => Value::Boolean(s.is_empty()),
                    },
                    _ => unreachable!(),
                }
            },
            Expr::Binary {operator, left, right} => {
                let left = Evaluator::evaluate(*left);
                let right = Evaluator::evaluate(*right);
                match operator.token_type {
                    TokenType::STAR => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Value::Number(l * r),
                        _ => unreachable!(),
                    },
                    TokenType::SLASH => match (left, right) {
                        (Value::Number(l), Value::Number(r)) => Value::Number(l / r),
                        _ => unreachable!(),
                    },
                    _ => todo!(),
                }
            }
        }
    }
}