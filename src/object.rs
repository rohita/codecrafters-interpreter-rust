use crate::error::Error;
use crate::function::Function;
use crate::token::Token;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Object {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil,
    Callable(Box<Function>),
}

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
                        format!("Expected {} arguments but got {}.", func.arity(), args.len()),
                    ));
                }
                func.call(args)
            }
            _ => Err(Error::RuntimeError(paren, "Can only call functions and classes.".to_string())),
        }
    }
}