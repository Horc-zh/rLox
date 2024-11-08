use core::fmt;

use crate::environment::Environment;
use crate::loxresult::LoxResult;
use crate::token::Token;
use crate::value::Value;
use crate::{loxcallable::LoxCallable, stmt::Stmt};

///定义了函数结构
#[derive(PartialEq, Clone, Debug)]
struct Declaration {
    ///函数名称
    name: Token,
    ///参数列表
    params: Vec<Token>,
    ///函数体
    body: Vec<Stmt>,
}

///定义了函数
#[derive(PartialEq, Clone, Debug)]
pub struct LoxFunction {
    declaration: Declaration,
    // closure: Environment,
}

impl LoxFunction {
    //TODO: cannot ensure the argument's kind is Stmt::Function
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        // environment: Environment,
    ) -> LoxFunction {
        LoxFunction {
            declaration: Declaration { name, params, body },
        }
    }
}

///为[`LoxFunction`] 实现 [`fmt::Display`] ,这样可以使用[`print`]打印出函数的类型
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
    ) -> Result<Value, LoxResult> {
        // let mut env = self.closure.clone();
        let mut env = interpreter.globals.clone();

        for (index, token) in self.declaration.params.iter().enumerate() {
            env.define(token.lexeme.clone(), arguments[index].clone());
        }

        match interpreter.execute_block(self.declaration.body.clone(), env) {
            Err(LoxResult::ReturnValue { value }) => return Ok(value),
            Err(e) => Err(e),
            Ok(value) => Ok(value),
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}
