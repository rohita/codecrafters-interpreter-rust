use crate::error::Error;
use crate::interpreter::Object;
use crate::token::Token;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Box<Environment>>,
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
    pub fn new_enclosing(enclosing: Environment) -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        // A new variable is always declared in the current innermost scope.
        // No need to define in outer scope.
        self.values.insert(name, value);
    }

    /// The key difference between assign and define is that assign is not allowed
    /// to create a new variable. That means it’s a runtime error if the key doesn’t
    /// already exist in the environment’s variable map.
    pub fn assign(&mut self, name: Token, value: Object) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            self.values.insert(variable, value.clone());
            return Ok(value);
        }

        // Walk the chain to find if the key exists
        if self.enclosing.is_some() {
            return self.enclosing.as_mut().unwrap().assign(name.clone(), value);
        }

        Err(Error::RuntimeError(
            name,
            format!("Undefined variable: '{}'", variable),
        ))
    }

    pub fn get(&self, name: Token) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            return Ok(self.values[&variable].clone());
        };

        // Walk the chain to find if the key exists
        match &self.enclosing {
            Some(outer) => outer.get(name),
            None => Err(Error::RuntimeError(
                name,
                format!("Undefined variable: '{}'", variable),
            )),
        }
    }

    pub fn get_enclosing(&self) -> Environment {
        match &self.enclosing {
            None => panic!("No enclosing environment"),
            Some(outer) => *outer.clone(),
        }
    }
}
