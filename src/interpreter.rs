use crate::environment::Environment;
use crate::error;
use crate::error::Error;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::TokenType;
use std::fmt::Display;

pub struct Interpreter {
    environment: Environment,
}

#[derive(Clone, Debug)]
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

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => continue,
                Err(error) => {
                    error::runtime_error(error);
                    break;
                }
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<Object, Error> {
        match stmt {
            Stmt::Expression { expression } => self.evaluate(expression),
            Stmt::Print { expression } => {
                let evaluated = self.evaluate(expression)?;
                println!("{evaluated}");
                Ok(evaluated)
            }
            Stmt::Var { name, initializer } => {
                let mut value = Object::Nil;
                if let Some(intz) = initializer {
                    value = self.evaluate(intz)?;
                }
                self.environment.define(name.lexeme, value.clone());
                Ok(value)
            }
            Stmt::Block { statements } => {
                self.environment = Environment::new_enclosing(self.environment.clone());

                let results: Result<Vec<_>, _> =
                    statements.into_iter().map(|s| self.execute(s)).collect();

                self.environment = self.environment.get_enclosing();
                match results {
                    Ok(_) => Ok(Object::Nil),
                    Err(error) => Err(error),
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let if_value = self.evaluate(condition)?;
                if self.is_truthy(if_value) {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
                Ok(Object::Nil)
            }
        }
    }

    pub fn evaluate(&mut self, expression: Expr) -> Result<Object, Error> {
        let return_val = match expression {
            Expr::Literal { value } => value,
            Expr::Grouping { expression } => self.evaluate(*expression)?,
            Expr::Unary { operator, right } => {
                let value = self.evaluate(*right)?;
                match operator.token_type {
                    TokenType::MINUS => match value {
                        Object::Number(n) => Object::Number(-n),
                        _ => {
                            return Err(Error::RuntimeError(
                                operator,
                                "Operand must be a number.".to_string(),
                            ))
                        }
                    },
                    TokenType::BANG => Object::Boolean(!self.is_truthy(value)),
                    _ => unreachable!(),
                }
            }
            Expr::Binary { operator, left, right} => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

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
                        _ => {
                            return Err(Error::RuntimeError(
                                operator,
                                "Operands must be numbers.".to_string(),
                            ))
                        }
                    },
                    (Object::Boolean(left), Object::Boolean(right)) => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(left != right),
                        TokenType::EQUAL_EQUAL => Object::Boolean(left == right),
                        _ => {
                            return Err(Error::RuntimeError(
                                operator,
                                "Operands must be numbers.".to_string(),
                            ))
                        }
                    },
                    (Object::Nil, Object::Nil) => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(false),
                        TokenType::EQUAL_EQUAL => Object::Boolean(true),
                        _ => {
                            return Err(Error::RuntimeError(
                                operator,
                                "Operands must be numbers.".to_string(),
                            ))
                        }
                    },
                    _ => match operator.token_type {
                        TokenType::BANG_EQUAL => Object::Boolean(true),
                        TokenType::EQUAL_EQUAL => Object::Boolean(false),
                        _ => {
                            return Err(Error::RuntimeError(
                                operator,
                                "Operands must be numbers.".to_string(),
                            ))
                        }
                    },
                }
            }
            Expr::Variable { name } => self.environment.get(name)?,
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.environment.assign(name, value.clone())?;
                value
            }
        };
        Ok(return_val)
    }

    fn is_truthy(&self, value: Object) -> bool {
        match value {
            Object::Boolean(b) => b,
            Object::Nil => false,
            _ => true,
        }
    }
}
