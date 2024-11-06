use crate::token_type::TokenType;
use std::fmt::Display;

///Token结构体
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    ///token的类型
    pub token_type: TokenType,
    ///token的原始字符，比如 [`TokenType::LESS`] 的原始字符是`<`
    pub lexeme: String,
    ///token包含的值，内部是 [`Literal`]
    pub literal: Option<Literal>,
    ///token所在的行数
    pub line: i32,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: i32,
    ) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

///用于记录token内部的值
///
///比如[`TokenType::TRUE`] 这个token内部的值为`bool`值`true`
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
