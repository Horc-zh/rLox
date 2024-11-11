use crate::value::Value;
use crate::{token::Token, Lox};

// pub(crate) struct LoxResult {
//     pub(crate) token: Token,
//     pub(crate) message: String,
// }
///定义了解释时语言发生的异常和一些函数的返回值
#[derive(Debug, Clone)]
pub enum LoxResult {
    RuntimeError {
        token: Token,
        message: String,
    },
    ParseError {
        token: Token,
        message: String,
    },
    ///当要在函数体中提前返回时，会触发这个异常
    ReturnValue {
        value: Value,
    },
    ///循环语句中返回
    Break,
}

impl LoxResult {
    pub fn error(&self) -> Self {
        match self {
            LoxResult::RuntimeError { token, message }
            | LoxResult::ParseError { token, message } => {
                Lox::error_with_token(&token, message);
                return self.clone();
            }
            _ => unreachable!(),
        }
    }
}
