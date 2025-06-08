use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::value::object::Object;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Object>) -> Result<Object, Error>;
}