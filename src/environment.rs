use std::cell::RefCell;
use crate::error::Error;
use crate::object::Object;
use crate::token::Token;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use crate::function::Function;

/// Mutable type to easily modify values in memory
pub type MutableEnvironment = Rc<RefCell<Environment>>;

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
    /// The name of the scope which owns this environment. Helps with debugging. 
    name: String,
    
    /// Map to store the bindings. It uses bare strings for the keys, not tokens. 
    /// A token represents a unit of code at a specific place in the source text, 
    /// but when it comes to looking up variables, all identifier tokens with the 
    /// same name should refer to the same variable. Using the raw string ensures 
    /// all of those tokens refer to the same map key.
    values: HashMap<String, Object>,
    
    /// This is the parent environment (the outer scope).
    enclosing: Option<MutableEnvironment>,
}

impl Environment {
    /// The globals
    pub fn global_env() -> MutableEnvironment {
        let mut global = Self {
            name: "global".to_string(),
            values: HashMap::new(),
            enclosing: None,
        };
        global.define("clock".to_string(), Object::Callable(Box::from(Function::Clock)));
        Rc::new(RefCell::new(global))
    }

    /// This constructor creates a new local scope nested inside the given outer one.
    pub fn new(enclosing: MutableEnvironment, name: &str) -> MutableEnvironment {
        Rc::new(RefCell::new(Self {
            name: name.to_string(),
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    /// A variable definition binds a new name to a value.
    pub fn define(&mut self, name: String, value: Object) {
        // A new variable is always declared in the current innermost scope.
        // No need to define in outer scope.
        // eprintln!("env:{} var: {name}: value: {value:#?}", self.name);
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
            None => Err(Error::RuntimeError(name, format!("Undefined variable: '{variable}'")))
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: Token, value: Object) -> Result<(), Error> {
        if distance == 0 {
            return self.assign(name, value);
        }
        self.ancestor(distance).borrow_mut().assign(name, value)
    }

    pub fn get(&self, name: &Token) -> Result<Object, Error> {
        let variable = &name.lexeme;
        if let Some(value) = self.values.get(variable) {
            return Ok(value.clone());
        }

        // Walk the chain to find if the key exists
        match &self.enclosing {
            Some(outer) => outer.borrow().get(name),
            None => Err(Error::RuntimeError(
                name.clone(),
                format!("Undefined variable: '{}'", variable),
            )),
        }
    }

    /// The previous get() method dynamically walks the chain of enclosing environments,
    /// scouring each one to see if the variable might be hiding in there somewhere.
    /// With this, we know exactly which environment in the chain will have the variable.
    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object, Error> {
        if distance == 0 {
            return self.get(name);
        }
        self.ancestor(distance).borrow().get(name)
    }

    fn ancestor(&self, distance: usize) -> MutableEnvironment {
        let mut environment = self.enclosing.clone().expect("No enclosing environment");

        for _ in 1..distance {
            let next = environment
                .borrow()
                .enclosing
                .clone()
                .expect("No enclosing environment at required distance");
            environment = next;
        }
        environment
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if let Some(enclosing) = &self.enclosing {
            write!(
                f,
                "({},{:?},[{}])",
                self.name,
                self.values.keys(),
                enclosing.borrow()
            )
        } else {
            write!(f, "({},{:?})", self.name, self.values.keys())
        }
    }
}
