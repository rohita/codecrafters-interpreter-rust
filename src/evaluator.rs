use std::fmt::Display;
use crate::expr::Expr;
use crate::token::TokenType;

pub struct Evaluator;

pub enum Object {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Object::Nil => f.write_str("nil"),
            Object::Number(n) => f.write_fmt(format_args!("{n}")),
            Object::String(s) => f.write_fmt(format_args!("{s}")),
        }
    }
}

impl Evaluator {
    pub fn evaluate(expression: Expr) -> Object {
        match expression {
            Expr::Literal(value) => value,
            Expr::Grouping(e) => Evaluator::evaluate(*e),
            Expr::Unary{operator, right} => {
                let value = Evaluator::evaluate(*right);
                match operator.token_type {
                    TokenType::MINUS => match value {
                        Object::Number(n) => Object::Number(-n),
                        _ => unreachable!(),
                    },
                    TokenType::BANG => match value {
                        Object::Boolean(b) => Object::Boolean(!b),
                        Object::Nil => Object::Boolean(true),
                        Object::Number(n) => Object::Boolean(n == 0.0),
                        Object::String(s) => Object::Boolean(s.is_empty()),
                    },
                    _ => unreachable!(),
                }
            },
            Expr::Binary {operator, left, right} => {
                let left = Evaluator::evaluate(*left);
                let right = Evaluator::evaluate(*right);

                match (left, right) {
                    (Object::Number(left), Object::Number(right)) => match operator.token_type {
                        TokenType::STAR => Object::Number(left * right),
                        TokenType::SLASH => Object::Number(left / right),
                        TokenType::PLUS => Object::Number(left + right),
                        TokenType::MINUS => Object::Number(left - right),
                        TokenType::GREATER => Object::Boolean(left > right),
                        TokenType::GREATER_EQUAL => Object::Boolean(left >= right),
                        TokenType::LESS => Object::Boolean(left < right),
                        TokenType::LESS_EQUAL => Object::Boolean(left <= right),
                        TokenType::BANG_EQUAL => Object::Boolean(left != right),
                        TokenType::EQUAL_EQUAL => Object::Boolean(left == right),
                        _ => unreachable!(),
                    },
                    (Object::String(left), Object::String(right)) => match operator.token_type {
                        TokenType::PLUS => Object::String(left + right.as_str()),
                        TokenType::BANG_EQUAL => Object::Boolean(left != right),
                        TokenType::EQUAL_EQUAL => Object::Boolean(left == right),
                        _ => unreachable!(),
                    },
                    (Object::Boolean(left), Object::Boolean(right)) => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(left != right),
                        TokenType::EQUAL_EQUAL => Object::Boolean(left == right),
                        _ => unreachable!(),
                    },
                    (Object::Nil, Object::Nil) => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(false),
                        TokenType::EQUAL_EQUAL => Object::Boolean(true),
                        _ => unreachable!(),
                    }
                    _ => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(true),
                        TokenType::EQUAL_EQUAL => Object::Boolean(false),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}