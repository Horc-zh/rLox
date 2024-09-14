use crate::value::Value;
use crate::{token::Token, Lox};

// pub(crate) struct LoxResult {
//     pub(crate) token: Token,
//     pub(crate) message: String,
// }
#[derive(Debug, Clone)]
pub enum LoxResult {
    RuntimeError { token: Token, message: String },
    ParseError { token: Token, message: String },
    ReturnValue { value: Value },
    //TODO:
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
