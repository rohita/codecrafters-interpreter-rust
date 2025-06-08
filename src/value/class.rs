use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::value::callable::Callable;
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
    
    /// The parent class 
    pub superclass: Option<Rc<Class>>,
    
    /// Even though methods are owned by the class, they are still accessed 
    /// through instance of that class.
    pub methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(name: String, superclass: Option<Rc<Class>>, methods: HashMap<String, Function>) -> Self {
        Self { name, superclass, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone())
        }
        
        // If there is a method in a superclass, we should be able to call that method 
        // when given an instance of the subclass. In other words, methods are inherited 
        // from the superclass.
        if let Some(superclass) = &self.superclass {
            return superclass.find_method(&name)
        }
        
        None
    }
}

impl Callable for Class {
    /// If there is an initializer, that method’s arity determines how many arguments 
    /// you must pass when you call the class itself. If you don’t have an initializer, 
    /// the arity is zero.
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Object>) -> Result<Object, Error> {
        // When we “call” a class, it instantiates a new Instance 
        // for the called class and returns it.
        let instance = Instance::new(self.clone());
        let instance_object = Object::Instance(Rc::new(RefCell::new(instance)));
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(&instance_object).call(interpreter, args)?;
        }
        Ok(instance_object)
    }
}