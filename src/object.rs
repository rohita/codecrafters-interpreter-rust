use std::string::String;
use crate::error::Error;
use crate::function::Function;
use crate::token::Token;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Object {
    Boolean(bool),
    String(String),
    Number(f64),   // Lox uses double-precision numbers even for integer values.
    Nil,
    Callable(Box<Function>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => f.write_fmt(format_args!("{b}")),
            Object::Nil => f.write_str("nil"),
            Object::Number(n) => f.write_fmt(format_args!("{n}")), // print integer without decimal point
            Object::String(s) => f.write_fmt(format_args!("{s}")),
            Object::Callable(func) => f.write_fmt(format_args!("<fn {}>", func.name())),
        }
    }
}

impl Object {
    /// All types are partitioned into two sets, one of which are defined to be true ("truthy"),
    /// and the rest which are false (“falsey”). This partitioning is somewhat arbitrary.
    /// Lox follows Ruby’s simple rule: false and nil are falsey, and everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(b) => *b,
            Object::Nil => false,
            _ => true,
        }
    }

    pub fn is_equal(&self, right: Object) -> bool {
        match (self, right) {
            (Object::Nil, Object::Nil) => true,
            (Object::Nil, _) => false,
            (Object::Number(l), Object::Number(r)) => *l == r,
            (Object::Boolean(l), Object::Boolean(r)) => *l == r,
            (Object::String(l), Object::String(r)) => *l == r,
            _ => false,
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