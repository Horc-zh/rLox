use crate::{token::Token, Lox};

#[derive(Debug)]
pub(crate) struct RuntimeError {
    pub(crate) token: Token,
    pub(crate) message: String,
}

impl RuntimeError {
    pub(crate) fn new(token: Token, message: String) -> RuntimeError {
        RuntimeError { token, message }
    }

    pub fn error(self) {
        Lox::error_with_token(self.token, &self.message);
    }
}
