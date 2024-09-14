use crate::{interpreter::Interpreter, loxresult::LoxResult, value::Value};

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxResult>;
    fn arity(&self) -> usize;
}
