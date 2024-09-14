use core::fmt;

use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::value::Value;
use crate::{loxcallable::LoxCallable, stmt::Stmt};

#[derive(PartialEq, Clone, Debug)]
struct Declaration {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct LoxFunction {
    declaration: Declaration,
}

impl LoxFunction {
    //TODO: cannot ensure the argument's kind is Stmt::Function
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> LoxFunction {
        LoxFunction {
            declaration: Declaration { name, params, body },
        }
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut crate::interpreter::Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        //WARNING: maybe error occur here
        let mut env = interpreter.globals.clone();

        for (index, token) in self.declaration.params.iter().enumerate() {
            env.define(token.lexeme.clone(), arguments[index].clone());
        }

        interpreter.execute_block(self.declaration.body.clone(), env)?;

        //TODO: change this the control the return value of function
        Ok(Value::Nil)
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
    // add code here
}
