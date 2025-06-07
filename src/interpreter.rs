use crate::environment::{Environment, MutableEnvironment};
use crate::error;
use crate::error::Error;
use crate::error::Error::RuntimeError;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::TokenType::*;
use crate::value::class;
use crate::value::function::Function;
use crate::value::object::Object;
use crate::value::object::Object::*;
use std::collections::HashMap;
use crate::token::Token;

/// Interpreter is the third step. It takes in the AST produced by the parser and
/// recursively traverse it, building up a value which it ultimately returned.
/// The interpreter does a **post-order traversal**, where each node evaluates
/// its children before doing its own work.
///
/// The two note types - Stmt and Expr - are handled in separate methods. Stmt are
/// executed in the `execute` method, and Expr are evaluated in the `evaluate` method.
pub struct Interpreter {
    /// This tracks the current environment.
    /// It changes as we enter and exit local scopes.
    environment: MutableEnvironment,

    /// Holds a fixed reference to the outermost global environment.
    globals: MutableEnvironment,

    /// "Side table" that associates each AST node with its "resolved location".
    /// That is, its distance to the outer environment where the interpreter can
    /// find the variable’s value.
    locals: Option<HashMap<*const Expr, usize>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let global = Environment::global_env();
        Self {
            environment: global.clone(),
            globals: global,
            locals: None,
        }
    }

    pub fn new_with_resolver(locals: HashMap<*const Expr, usize>) -> Interpreter {
        let global = Environment::global_env();
        Self {
            environment: global.clone(),
            globals: global,
            locals: Some(locals),
        }
    }

    /// Takes in a list of statements — in other words, a program.
    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
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
    
    pub fn execute_block(&mut self, statements: &Vec<Stmt>, block_scope: MutableEnvironment) -> Result<(), Error> {
        let previous = self.environment.clone();
        self.environment = block_scope;
        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));
        self.environment = previous;
        result
    }

    /// This is the statement analogue to the evaluate() method we have for expressions.
    /// Unlike expressions, statements produce no values, so the return type is Void, not Object.
    fn execute(&mut self, stmt: &Stmt) -> Result<(), Error> {
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
                self.environment.borrow_mut().define(name.lexeme.clone(), value.clone());
                Ok(())
            }
            Stmt::Block { statements } => {
                let block_scope = Environment::new(self.environment.clone(), "block");
                self.execute_block(statements, block_scope)?;
                Ok(())
            }
            Stmt::Class { name, methods } => {
                // The two-stage variable binding process allows references 
                // to the class inside its own methods.
                self.environment.borrow_mut().define(name.lexeme.clone(), Nil);
                
                // Each method declaration becomes a Function object.
                let mut class_methods = HashMap::new();
                for method in methods {
                    // When we first evaluate the class definition, the closure is the 
                    // environment surrounding the class, in this case the global one. 
                    let func = Function::new(method.clone(), self.environment.clone());
                    class_methods.insert(method.name.lexeme.clone(), func); 
                }
                let klass = Class(class::Class::new(name.lexeme.clone(), class_methods));
                self.environment.borrow_mut().assign(name.clone(), klass)?;
                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let if_value = self.evaluate(condition)?;
                if if_value.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            },
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(())
            },
            Stmt::Function { decl } => {
                // This is similar to how we interpret other literal expressions. We take a
                // function syntax node (Stmt::Function) — a compile-time representation of
                // the function — and convert it to its runtime representation. Here, that’s
                // a Function::UserDefined that wraps the syntax node.
                //
                // Also, this closure “closes over” and holds on to the surrounding variables
                // where the function is declared.
                let func = Function::new(decl.clone(), self.environment.clone());
                let name = func.name();
                let value = Function(func);
                self.environment.borrow_mut().define(name, value);
                Ok(())
            },
            Stmt::Return { value, .. } => {
                // If we have a return value, we evaluate it, otherwise, we use nil.
                let mut return_value = Nil;
                if let Some(value) = value {
                    return_value = self.evaluate(value)?;
                }

                // This can return from anywhere within the body of a function, even deeply
                // nested inside other statements. When the return is executed, we need to
                // jump all the way out of whatever function it’s currently in and cause the
                // function call to complete. We’ll use an exception to unwind the interpreter
                // past the visit methods of all the containing statements back to the code
                // that began executing the body.
                Err(Error::Return(return_value))
            },
        }
    }

    /// This evaluates an Expr tree node and produce a value. For each kind of Expr — literal,
    /// operator, etc. — we have a corresponding chunk of code that knows how to evaluate
    /// that tree and produce a result represented by the Object enum.
    pub fn evaluate(&mut self, expression: &Expr) -> Result<Object, Error> {
        match expression {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let value = self.evaluate(right)?;
                match (&operator.token_type, value) {
                    (MINUS, Number(n)) => Ok(Number(-n)),
                    (BANG, value) => Ok(Boolean(!value.is_truthy())),
                    _ => Err(RuntimeError(operator.clone(), "Operand must be a number.".into()))
                }
            }
            Expr::Binary { left, operator, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
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
                    _ => Err(RuntimeError(operator.clone(), "Operands must be numbers.".into()))
                }
            }
            Expr::Variable { name } => {
                self.lookup_variable(expression, name)
            }
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.assign_variable(expression, name.clone(), value.clone())?;
                Ok(value) // Assignment can be nested inside other expressions. So needs a value.
            },
            Expr::Logical { left, operator, right } => {
                let left_eval = self.evaluate(left)?;
                
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
                
                // Instead of returning `true` or `false`, a logic operator returns
                // a value with appropriate "truthiness".
                // For example:
                // print "hi" or 2; // "hi".
                // print nil or "yes"; // "yes".
                // On the first example, "hi" is truthy, so the 'or' short-circuits and returns "hi".
                // On the second example, nil is falsey, so it returns the second operand, "yes".
                self.evaluate(right)
            },
            Expr::Call { callee, arguments, paren } => {
                let callee_evaluated = self.evaluate(callee)?;
                let mut args_evaluated = Vec::new();
                for argument in arguments {
                    args_evaluated.push(self.evaluate(argument)?);
                }

                callee_evaluated.call(self, args_evaluated, paren.clone())
            },
            Expr::Get { object, name } => {
                let object_evaluated = self.evaluate(object)?;
                if let Instance(instance) = object_evaluated {
                    return instance.borrow().get(name)
                }
                Err(RuntimeError(name.clone(), "Only instances have properties.".into()))
            },
            Expr::Set { object, name, value } => {
                let object_evaluated = self.evaluate(object)?;
                if let Instance(instance) = object_evaluated {
                    let value_evaluated = self.evaluate(value)?;
                    instance.borrow_mut().set(name, value_evaluated.clone());
                    return Ok(value_evaluated);
                }
                Err(RuntimeError(name.clone(), "Only instances have fields.".into()))
            }
            Expr::This { keyword } => {
                self.lookup_variable(expression, keyword)
            }
        }
    }

    fn lookup_variable(&self, expression: &Expr, name: &Token) -> Result<Object, Error> {
        if self.locals.is_none() {
            return self.environment.borrow().get(name);
        }
        let distance = self.get_depth(expression);
        if let Some(distance) = distance {
            self.environment.borrow().get_at(distance, name)
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn assign_variable(&mut self, expr: &Expr, name: Token, value: Object) -> Result<(), Error> {
        if self.locals.is_none() {
            return self.environment.borrow_mut().assign(name, value);
        }
        let distance = self.get_depth(expr);
        if let Some(distance) = distance {
            self.environment.borrow_mut().assign_at(distance, name, value)
        } else {
            self.globals.borrow_mut().assign(name, value)
        }
    }

    fn get_depth(&self, expr: &Expr) -> Option<usize> {
        let ptr = expr as *const Expr;
        let depth = self.locals.as_ref()?.get(&ptr).copied();
        //eprintln!("Get Distance: ptr: {:?} name: {} distance: {:?}", ptr, expr.to_string(), depth);
        depth
    }
}

