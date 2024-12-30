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
    
    pub fn get(&self, name: Token) -> Result<Object, Error> {
        let variable = name.lexeme.clone();
        if self.values.contains_key(&variable) {
            return Ok(self.values[&variable].clone());
        }; 
        Err(Error::RuntimeError(name, format!("Undefined variable: '{}'", variable)))
    }
}