use crate::error::Error;
use crate::value::function::Function;
use crate::value::instance::Instance;
use crate::value::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The syntactic representation of Class — the runtime representation of the 
/// class declaration stmt (the AST node).
#[derive(Clone, Debug)]
pub struct Class {
    /// Class name
    pub name: String,
    
    /// Even though methods are owned by the class, they are still accessed 
    /// through instance of that class.
    pub methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Self {
        Self {
            name,
            methods,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<Function> {
        self.methods.get(name).cloned()
    }

    pub fn call(&self) -> Result<Object, Error> {
        // When we “call” a class, it instantiates a new Instance 
        // for the called class and returns it.
        let instance = Instance::new(self.clone());
        Ok(Object::Instance(Rc::new(RefCell::new(instance))))
    }
}