use crate::environment::Environment;
use crate::error;
use crate::error::Error;
use crate::expr::Expr;
use crate::object::Object;
use crate::stmt::Stmt;
use crate::token::TokenType;

pub struct Interpreter {
    environment: Environment,
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
                if if_value.is_truthy() {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
                Ok(Object::Nil)
            },
            Stmt::While { condition, body } => {
                while self.evaluate(condition.clone())?.is_truthy() {
                    self.execute(*body.clone())?;
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
                    TokenType::BANG => Object::Boolean(!value.is_truthy()),
                    _ => unreachable!(),
                }
            }
            Expr::Binary { left, operator, right} => {
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
            },
            Expr::Logical { left, operator, right } => {
                let left_eval = self.evaluate(*left)?;
                
                // We look at left value to see if we can short-circuit. 
                // If not, and only then, do we evaluate the right operand.
                if operator.token_type == TokenType::OR {
                    if left_eval.is_truthy() {
                        return Ok(left_eval);
                    }
                } else {
                    if !left_eval.is_truthy() {
                        return Ok(left_eval);
                    }
                }
                
                // Instead of returning true or false, a logic operator merely 
                // guarantees it will return a value with appropriate truthiness.
                // For example:
                // print "hi" or 2; // "hi".
                // print nil or "yes"; // "yes".
                // On the first example, "hi" is truthy, so the 'or' short-circuits and returns that. 
                // On the second example, 'nil is falsey, so it evaluates and returns the second operand, "yes".
                self.evaluate(*right)?
            },
        };
        Ok(return_val)
    }
}
