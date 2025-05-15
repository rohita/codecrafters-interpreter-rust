use std::cell::RefCell;
use crate::error::Error;
use crate::object::Object;
use crate::token::Token;
use std::collections::HashMap;
use std::rc::Rc;

/// The bindings that associate variables to values need to be stored somewhere. 
/// This storage is called an 'environment'. This is a map where the keys are variable 
/// names and the values are their values. We could have stuff this map and the code to 
/// manage it right into Interpreter, but since it forms a nicely delineated concept, 
/// we’ll pull it out into its own class.
/// 
/// A scope defines a region where a name maps to a certain entity. Scope and environments 
/// are close cousins. The former is the theoretical concept, and the latter is the machinery 
/// that implements it. Scope is controlled by curly-braced blocks ("block scope").
/// 
/// Functions and variables occupy the same namespace.
#[derive(Debug)]
pub struct Environment {
    /// Map to store the bindings. It uses bare strings for the keys, not tokens. 
    /// A token represents a unit of code at a specific place in the source text, 
    /// but when it comes to looking up variables, all identifier tokens with the 
    /// same name should refer to the same variable. Using the raw string ensures 
    /// all of those tokens refer to the same map key.
    values: HashMap<String, Object>,
    
    /// This is the parent environment (the outer scope).
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// Constructor for the global scope’s environment
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// This constructor creates a new local scope nested inside the given outer one.
    pub fn new_enclosing(enclosing: &Rc<RefCell<Environment>>) -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
        }
    }

    /// A variable definition binds a new name to a value.
    pub fn define(&mut self, name: String, value: Object) {
        // A new variable is always declared in the current innermost scope.
        // No need to define in outer scope.
        self.values.insert(name, value);
    }

    /// The key difference between assign and define is that assign is not allowed
    /// to create a new variable. It’s a runtime error if the key doesn’t
    /// already exist.
    pub fn assign(&mut self, name: Token, value: Object) -> Result<(), Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            self.values.insert(variable, value);
            return Ok(());
        }

        // Walk the chain to find if the key exists
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            None => Err(Error::RuntimeError(
                name,
                format!("Undefined variable: '{}'", variable),
            ))
        }
    }

    pub fn get(&self, name: Token) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if let Some(value) = self.values.get(&variable) {
            return Ok(value.clone());
        }

        // Walk the chain to find if the key exists
        match &self.enclosing {
            Some(outer) => outer.borrow().get(name),
            None => Err(Error::RuntimeError(
                name,
                format!("Undefined variable: '{}'", variable),
            )),
        }
    }

    // pub fn get_enclosing(&self) -> Environment {
    //     match &self.enclosing {
    //         None => panic!("No enclosing environment"),
    //         Some(outer) => **outer,
    //     }
    // }
}
