use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::object::Object::Nil;
use crate::stmt::Stmt;

#[derive(Clone, Debug)]
pub enum Function {
    Clock,
    UserDefined {
        declaration: Stmt,
        closure: Rc<RefCell<Environment>>,
    },
}

impl Function {
    pub fn name(&self) -> String {
        match self {
            Function::Clock => "clock".to_string(),
            Function::UserDefined {declaration, ..} => {
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

    pub fn call(&self, args: Vec<Object>) -> Result<Object, Error> {
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
                    let scope = Rc::new(RefCell::new(Environment::new_enclosing(closure)));
                    for (i, param) in params.iter().enumerate() {
                        scope.borrow_mut().define(param.clone().lexeme, args[i].clone());
                    }
                    let mut function_interpreter = Interpreter::new_with_env(scope);
                    return match function_interpreter.execute_block(body.clone()) {
                        Err(Error::Return(value)) => Ok(value),
                        Err(r) => Err(r),
                        _ => Ok(Nil)
                    }
                }
                unreachable!()
            }
        }
    }
}

pub fn globals() -> Rc<RefCell<Environment>> {
    let env = Rc::new(RefCell::new(Environment::new()));
    env.borrow_mut().define("clock".to_string(), Object::Callable(Box::from(Function::Clock)));
    env
}