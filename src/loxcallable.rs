use crate::{interpreter::Interpreter, loxresult::LoxResult, value::Value};

///定义了可以被调用的结构体的共同特征
///
///目前只有[`crate::loxfunction`]
pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, LoxResult>;
    fn arity(&self) -> usize;
}
