use crate::environment::{Environment, MutableEnvironment};
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::stmt::FunctionDeclaration;
use crate::value::callable::Callable;
use crate::value::object::Object;
use crate::value::object::Object::Nil;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The runtime representation of a function statement 
#[derive(Clone, Debug)]
pub enum Function {
    Clock,
    UserDefined {
        /// Is this function an init. We can’t simply see if the name of the function 
        /// is “init” because the user could have defined a function with that name.
        is_initializer: bool,
        
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
    pub fn new(
        declaration: Rc<FunctionDeclaration>, 
        closure: MutableEnvironment, 
        is_initializer: bool) -> Self {
        Function::UserDefined {declaration, closure, is_initializer }
    }
    
    pub fn name(&self) -> String {
        match self {
            Function::Clock => "clock".to_string(),
            Function::UserDefined { declaration, ..} => declaration.name.lexeme.clone()
        }
    }
    
    pub fn bind(&self, instance_object: &Object) -> Function {
        match self {
            Function::UserDefined {declaration, closure, is_initializer } => {
                // We declare “this” as a variable in that environment and bind it to the 
                // given instance, the instance that the method is being accessed from. 
                // The returned Function now carries around its own little persistent world 
                // where “this” is bound to the object.
                let scope = Environment::new(closure.clone(), "bind env");
                scope.borrow_mut().define("this".into(), instance_object.clone()); 
                Function::new(declaration.clone(), scope, *is_initializer)
            }
            _ => self.clone()
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        match self {
            Function::Clock => 0,
            Function::UserDefined { declaration, ..} => declaration.params.len()
        }
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Object>) -> Result<Object, Error> {
        match self {
            Function::Clock => {
                let timestamp_f64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Object::Number(timestamp_f64))
            }
            Function::UserDefined {declaration, closure, is_initializer } => {
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
                    _ => {
                        match is_initializer {
                            // If the function is an initializer, we override the actual 
                            // return value and forcibly return this. 
                            true => closure.borrow().get_at(0, "this"),
                            
                            // Every Lox function must return something, even if it contains 
                            // no return statements at all. We use nil for this.
                            false => Ok(Nil)
                        }
                    }
                }
            }
        }
    }
}