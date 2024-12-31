use std::fmt::Display;
use crate::error::Error;
use crate::token::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Object {
    Boolean(bool),
    String(String),
    Number(f64),
    Nil,
    Callable {
        arity: usize,
        func: fn(Vec<Object>) -> Object,
    }, 
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Object::Nil => f.write_str("nil"),
            Object::Number(n) => f.write_fmt(format_args!("{n}")),
            Object::String(s) => f.write_fmt(format_args!("{s}")),
            Object::Callable { .. } => f.write_fmt(format_args!("<native fn>")),
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
            Object::Callable { arity, func } => {
                if args.len() != *arity {
                    return Err(Error::RuntimeError(
                        paren,
                        "Expected {arity} arguments but got {args_evaluated.len()}.".to_string(),
                    ));
                }
                Ok(func(args))
            }
            _ => Err(Error::RuntimeError(paren, "Can only call functions and classes.".to_string())),
        }
    }
}