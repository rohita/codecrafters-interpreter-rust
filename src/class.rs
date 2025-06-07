use std::cell::RefCell;
use std::rc::Rc;
use crate::error::Error;
use crate::instance::Instance;
use crate::object::Object;

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }

    pub fn call(&self) -> Result<Object, Error> {
        // When we “call” a class, it instantiates a new Instance 
        // for the called class and returns it.
        let instance = Instance::new(self.clone());
        Ok(Object::Instance(Rc::new(RefCell::new(instance))))
    }
}