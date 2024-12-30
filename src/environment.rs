use std::collections::HashMap;
use crate::error::Error;
use crate::interpreter::Object;
use crate::token::Token;

pub struct Environment {
    values: HashMap<String, Object>
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new()
        }
    }
    
    pub fn define(&mut self, name: String, value: Object) {
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
        Err(Error::RuntimeError(name, format!("Undefined variable: '{}'", variable)))
    }
    
    pub fn get(&self, name: Token) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            return Ok(self.values[&variable].clone());
        }; 
        Err(Error::RuntimeError(name, format!("Undefined variable: '{}'", variable)))
    }
}