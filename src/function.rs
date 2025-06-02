use std::rc::Rc;
use crate::environment::{Environment, MutableEnvironment};
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::object::Object::Nil;
use crate::stmt::Stmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub enum Function {
    Clock,
    UserDefined {
        /// Stmt::Function
        declaration: Stmt, 
        
        /// This holds surrounding variables where the function is declared.
        /// This is the environment that is active when the function is declared 
        /// not when it’s called. It represents the lexical scope surrounding the 
        /// function declaration.
        closure: MutableEnvironment, 
    },
}

impl Function {
    pub fn name(&self) -> String {
        match self {
            Function::Clock => "clock".to_string(),
            Function::UserDefined { declaration, ..} => {
                match declaration {
                    Stmt::Function { name, .. } => name.lexeme.clone(),
                    _ => "unknown".to_string(),
                }
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Clock => 0,
            Function::UserDefined { declaration, ..} => {
                match declaration {
                    Stmt::Function { params, .. } => params.len(),
                    _ => 0
                }
            }
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Object>) -> Result<Object, Error> {
        match self {
            Function::Clock => {
                let timestamp_f64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Object::Number(timestamp_f64))
            }
            Function::UserDefined {declaration, closure } => {
                if let Stmt::Function { params, body, .. } = declaration {
                    // We create a new environment at each call. We will execute the body of the function
                    // in this new function-local environment. Up until now, the current environment
                    // was the environment where the function was being called. Now, we teleport from
                    // there inside the new parameter space we’ve created for the function.
                    let scope = Environment::new(closure.clone(), &self.name());
                    for (i, param) in params.iter().enumerate() {
                        scope.borrow_mut().define(param.lexeme.clone(), args[i].clone());
                    }
                    
                    return match interpreter.execute_block(body, scope) {
                        Err(Error::Return(value)) => Ok(value),
                        Err(r) => Err(r),
                        // Every Lox function must return something, even if it contains 
                        // no return statements at all. We use nil for this.
                        _ => Ok(Nil)
                    }
                }
                unreachable!()
            }
        }
    }
}