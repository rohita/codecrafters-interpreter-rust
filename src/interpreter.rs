use crate::environment::Environment;
use crate::error;
use crate::error::Error;
use crate::error::Error::RuntimeError;
use crate::expr::Expr;
use crate::function::globals;
use crate::function::Function;
use crate::object::Object;
use crate::object::Object::*;
use crate::stmt::Stmt;
use crate::token::TokenType::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Interpreter is the third step. It takes in the AST produced by the parser and
/// recursively traverse it, building up a value which it ultimately returned.
/// The interpreter does a **post-order traversal**, where each node evaluates
/// its children before doing its own work.
///
/// The two note types - Stmt and Expr - are handled in separate methods. Stmt are
/// executed in the `execute` method, and Expr are evaluated in the `evaluate` method.
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {
            // The 'environment' field in the interpreter changes as we enter 
            // and exit local scopes. It tracks the current environment. This 
            // 'globals' holds a fixed reference to the outermost global environment.
            environment: globals(),
        }
    }
    
    pub fn new_with_env(environment: Rc<RefCell<Environment>>) -> Interpreter {
        Self { environment }
    }

    /// Takes in a list of statements — in other words, a program.
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
    
    pub fn execute_block(&mut self, statements: Vec<Stmt>) -> Result<Object, Error> {
        let results: Result<Vec<_>, _> =
            statements.into_iter().map(|s| self.execute(s)).collect();
        
        match results {
            Ok(_) => Ok(Nil),
            Err(error) => Err(error),
        }
    }

    /// This is the statement analogue to the evaluate() method we have for expressions.
    /// Unlike expressions, statements produce no values, so the return type is Void, not Object.
    fn execute(&mut self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }
            Stmt::Print { expression } => {
                let evaluated = self.evaluate(expression)?;
                println!("{evaluated}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = Nil;
                if let Some(expr) = initializer {
                    value = self.evaluate(expr)?;
                }
                self.environment.borrow_mut().define(name.lexeme, value.clone());
                Ok(())
            }
            Stmt::Block { statements } => {
                let block_scope = Rc::new(RefCell::new(Environment::new_enclosing(&self.environment)));
                let mut block_interpreter = Interpreter::new_with_env(block_scope);
                block_interpreter.execute_block(statements)?;
                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let if_value = self.evaluate(condition)?;
                if if_value.is_truthy() {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
                Ok(())
            },
            Stmt::While { condition, body } => {
                while self.evaluate(condition.clone())?.is_truthy() {
                    self.execute(*body.clone())?;
                }
                Ok(())
            },
            Stmt::Function { .. } => {
                // This is similar to how we interpret other literal expressions. We take a function 
                // syntax node (Stmt::Function) — a compile-time representation of the function — and convert it to 
                // its runtime representation. Here, that’s a Function::UserDefined that wraps the syntax node.
                let func = Function::UserDefined {declaration: stmt, closure: Rc::clone(&self.environment)}; 
                let name = func.name();
                let value = Callable(Box::from(func));
                self.environment.borrow_mut().define(name, value);
                Ok(())
            },
            Stmt::Return { value, .. } => {
                let mut return_value = Object::Nil;
                if let Some(value) = value {
                    return_value = self.evaluate(value)?;
                }
                Err(Error::Return(return_value))
            },
        }
    }

    /// This evaluates an Expr tree node and produce a value. For each kind of Expr — literal,
    /// operator, etc. — we have a corresponding chunk of code that knows how to evaluate
    /// that tree and produce a result represented by the Object enum.
    pub fn evaluate(&mut self, expression: Expr) -> Result<Object, Error> {
        match expression {
            Expr::Literal { value } => Ok(value),
            Expr::Grouping { expression } => self.evaluate(*expression),
            Expr::Unary { operator, right } => {
                let value = self.evaluate(*right)?;
                match (&operator.token_type, value) {
                    (MINUS, Number(n)) => Ok(Number(-n)),
                    (BANG, value) => Ok(Boolean(!value.is_truthy())),
                    _ => Err(RuntimeError(operator, "Operand must be a number.".into()))
                }
            }
            Expr::Binary { left, operator, right } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
                match (&operator.token_type, left, right) {
                    (STAR,  Number(left), Number(right)) => Ok(Number(left * right)),
                    (SLASH, Number(left), Number(right)) => Ok(Number(left / right)),
                    (PLUS,  Number(left), Number(right)) => Ok(Number(left + right)),
                    (PLUS,  String(left), String(right)) => Ok(String(left + right.as_str())),
                    (MINUS, Number(left), Number(right)) => Ok(Number(left - right)),
                    (GREATER, Number(left), Number(right)) => Ok(Boolean(left > right)),
                    (GREATER_EQUAL, Number(left), Number(right)) => Ok(Boolean(left >= right)),
                    (LESS, Number(left), Number(right)) => Ok(Boolean(left < right)),
                    (LESS_EQUAL, Number(left), Number(right)) => Ok(Boolean(left <= right)),
                    (BANG_EQUAL,  left, right) => Ok(Boolean(!left.is_equal(right))),
                    (EQUAL_EQUAL, left, right) => Ok(Boolean(left.is_equal(right))),
                    _ => Err(RuntimeError(operator, "Operands must be numbers.".into()))
                }
            }
            Expr::Variable { name } => self.environment.borrow().get(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value) // Assignment can be nested inside other expressions. So needs a value.
            },
            Expr::Logical { left, operator, right } => {
                let left_eval = self.evaluate(*left)?;
                
                // We look at left value to see if we can short-circuit. 
                // If not, and only then, do we evaluate the right operand.
                if operator.token_type == OR {
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
                self.evaluate(*right)
            },
            Expr::Call { callee, arguments, paren } => {
                let callee_evaluated = self.evaluate(*callee)?;    
                let mut args_evaluated = Vec::new();
                for argument in arguments {
                    args_evaluated.push(self.evaluate(argument)?);
                }

                callee_evaluated.call(args_evaluated, paren)
            },
        }
    }
}

