use std::cell::RefCell;
use crate::environment::{Environment, MutableEnvironment};
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::stmt::FunctionDeclaration;
use crate::token::Token;
use crate::value::object::Object;
use crate::value::object::Object::Nil;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::value::instance::Instance;

/// The runtime representation of a function statement 
#[derive(Clone, Debug)]
pub enum Function {
    Clock,
    UserDefined {
        /// Stmt::Function
        declaration: Rc<FunctionDeclaration>, 
        
        /// This holds surrounding variables where the function is declared.
        /// This is the environment that is active when the function is declared 
        /// not when it’s called. It represents the lexical scope surrounding the 
        /// function declaration.
        closure: MutableEnvironment, 
    },
}

impl Function {
    pub fn new(declaration: Rc<FunctionDeclaration>, closure: MutableEnvironment) -> Self {
        Function::UserDefined {declaration, closure }
    }
    
    pub fn name(&self) -> String {
        match self {
            Function::Clock => "clock".to_string(),
            Function::UserDefined { declaration, ..} => declaration.name.lexeme.clone()
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Clock => 0,
            Function::UserDefined { declaration, ..} => declaration.params.len()
        }
    }
    
    pub fn bind(&self, instance: &Instance) -> Function {
        match self {
            Function::Clock => { self.clone() }
            Function::UserDefined {declaration, closure } => {
                // We declare “this” as a variable in that environment and bind it to the 
                // given instance, the instance that the method is being accessed from. 
                // The returned Function now carries around its own little persistent world 
                // where “this” is bound to the object.
                let scope = Environment::new(closure.clone(), "bind env");
                let value = Object::Instance(Rc::new(RefCell::new(instance.clone())));
                scope.borrow_mut().define("this".into(), value); 
                Function::new(declaration.clone(), scope)
            }
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Object>, paren: Token) -> Result<Object, Error> {
        if args.len() != self.arity() {
            return Err(Error::RuntimeError(
                paren,
                format!("Expected {} arguments but got {}.", self.arity(), args.len()),
            ));
        }
        
        match self {
            Function::Clock => {
                let timestamp_f64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Object::Number(timestamp_f64))
            }
            Function::UserDefined {declaration, closure } => {
                // We create a new environment at each call. We will execute the body of the function
                // in this new function-local environment. Up until now, the current environment
                // was the environment where the function was being called. Now, we teleport from
                // there inside the new parameter space we’ve created for the function.
                let scope = Environment::new(closure.clone(), &self.name());
                for (i, param) in declaration.params.iter().enumerate() {
                    scope.borrow_mut().define(param.lexeme.clone(), args[i].clone());
                }
                
                match interpreter.execute_block(&declaration.body, scope) {
                    Err(Error::Return(value)) => Ok(value),
                    Err(r) => Err(r),
                    // Every Lox function must return something, even if it contains 
                    // no return statements at all. We use nil for this.
                    _ => Ok(Nil)
                }
            }
        }
    }
}