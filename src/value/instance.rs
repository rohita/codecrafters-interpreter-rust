use crate::error::Error;
use crate::token::Token;
use crate::value::class::Class;
use crate::value::object::Object;
use std::collections::HashMap;
use std::fmt::Display;

/// The runtime representation of an instance of a Lox class.
#[derive(Clone, Debug)]
pub struct Instance {
    pub klass: Class,
    pub fields: HashMap<String, Object>,
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass.name)
    }
}

impl Instance {
    pub fn new(klass: Class) -> Self {
        Self { klass, fields: HashMap::new() }
    }
    
    pub fn get(&self, token: &Token) -> Result<Object, Error> {
        let name = &token.lexeme;
        if let Some(value) = self.fields.get(name) {
            return Ok(value.clone());
        }

        // We could silently return some dummy value like nil, but that behavior masks bugs 
        // more often than it does anything useful. Instead, we’ll make it a runtime error.
        Err(Error::RuntimeError(token.clone(), format!("Undefined property '{}''.", name)))
    }
    
    pub fn set(&mut self, token: &Token, value: Object) {
        self.fields.insert(token.lexeme.clone(), value);
    }
}