use crate::error::Error;
use crate::token::Token;
use std::fmt::Display;
use crate::function::Function;

#[derive(Clone)]
pub enum Object {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil,
    Callable(Box<Function>),
}

/*
pub trait Callable: Clone {
    fn name(&self) -> String;
    fn arity(&self) -> usize;
    fn call(&self, args: Vec<Object>) -> Object;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.clone()
    }
}
*/

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Object::Nil => f.write_str("nil"),
            Object::Number(n) => f.write_fmt(format_args!("{n}")),
            Object::String(s) => f.write_fmt(format_args!("{s}")),
            Object::Callable(func) => f.write_fmt(format_args!("<fn {}>", func.name())),
        }
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(b) => *b,
            Object::Nil => false,
            _ => true,
        }
    }

    pub fn call(&self, args: Vec<Object>, paren: Token) -> Result<Object, Error> {
        match self {
            Object::Callable(func) => {
                if args.len() != func.arity() {
                    return Err(Error::RuntimeError(
                        paren,
                        "Expected {arity} arguments but got {args_evaluated.len()}.".to_string(),
                    ));
                }
                Ok(func.call(args))
            }
            _ => Err(Error::RuntimeError(paren, "Can only call functions and classes.".to_string())),
        }
    }
}