use std::time::{SystemTime, UNIX_EPOCH};
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::stmt::Stmt;

#[derive(Clone)]
pub enum Function {
    Clock,
    UserDefined(Stmt),
} 

impl Function {
    pub fn name(&self) -> String {
        match self {
            Function::Clock => "clock".to_string(),
            Function::UserDefined(declaration) => {
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
            Function::UserDefined(declaration) => {
                match declaration {
                    Stmt::Function { params, .. } => params.len(),
                    _ => 0
                }
            }
        }
    }
    
    pub fn call(&self, args: Vec<Object>) -> Object {
        match self {
            Function::Clock => {
                let timestamp_f64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Object::Number(timestamp_f64)
            }
            Function::UserDefined(declaration) => {
                if let Stmt::Function { params, body, .. } = declaration {
                    // We create a new environment at each call. We will execute the body of the function 
                    // in this new function-local environment. Up until now, the current environment 
                    // was the environment where the function was being called. Now, we teleport from 
                    // there inside the new parameter space we’ve created for the function.
                    let mut function_scope = Environment::new_enclosing(globals());
                    for (i, param) in params.iter().enumerate() {
                        function_scope.define(param.clone().lexeme, args[i].clone());
                    }
                    let mut function_interpreter = Interpreter::new_with_env(function_scope);
                    return match function_interpreter.execute_block(body.clone()) {
                        Ok(result) => result,
                        _ => Object::Nil,
                    }
                }
                Object::Nil
            }
        }
    }
}

pub fn globals() -> Environment {
    let mut env = Environment::new();
    env.define("clock".to_string(), Object::Callable(Box::from(Function::Clock)));
    env
}