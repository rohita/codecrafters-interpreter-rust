use std::cell::RefCell;
use crate::error::Error;
use crate::object::Object;
use crate::token::Token;
use std::collections::HashMap;
use std::rc::Rc;

/// Functions and variables occupy the same namespace.
#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
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

    pub fn define(&mut self, name: String, value: Object) {
        // A new variable is always declared in the current innermost scope.
        // No need to define in outer scope.
        self.values.insert(name, value);
    }

    /// The key difference between assign and define is that assign is not allowed
    /// to create a new variable. It’s a runtime error if the key doesn’t
    /// already exist.
    pub fn assign(&mut self, name: Token, value: Object) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            self.values.insert(variable, value.clone());
            return Ok(value);
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
        if self.values.contains_key(&variable) {
            return Ok(self.values[&variable].clone());
        };

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
