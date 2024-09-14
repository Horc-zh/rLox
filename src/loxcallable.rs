use crate::{interpreter::Interpreter, runtime_error::RuntimeError, value::Value};

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
    fn arity(&self) -> usize;
}
