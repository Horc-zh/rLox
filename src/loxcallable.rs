use crate::{interpreter::Interpreter, value::Value};

pub trait LoxCallable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Value;
    fn arity(&self) -> usize;
}
