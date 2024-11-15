//! scanner.rs是用于词法分析的文件，它将文本分析成`token`流，并将`token`流传递给语法分析器[`crate::parser`]

use crate::token::Literal;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::token_type::TokenType::*;
use crate::Lox;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
///使用lazy定义了lox语言的关键字
    ///内部是一个HASHMAP,可以点击左上角的*source*查看源码
    pub static ref KEYWORDS: HashMap<String, TokenType> = {
        [
            ("and", AND),
            ("class", CLASS),
            ("else", ELSE),
            ("false", FALSE),
            ("for", FOR),
            ("fun", FUN),
            ("if", IF),
            ("nil", NIL),
            ("or", OR),
            ("print", PRINT),
            ("return", RETURN),
            ("super", SUPER),
            ("this", THIS),
            ("true", TRUE),
            ("var", VAR),
            ("while", WHILE),
        ]
        .iter()
        .map(|&(k, v)| (String::from(k), v))
        .collect()
    };
}

///`Scanner`结构体
pub struct Scanner {
    ///源代码
    source: String,
    ///保存分析得出的token流
    tokens: Vec<Token>,
    ///记录了一个词开头在`source`中的位置
    start: i32,
    ///记录分析到了位置
    current: i32,
    ///记录分析到了文件的哪一行，每次遇到一个`\n`，`line = line + 1`
    line: i32,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    ///启动scanner进行词法分析
    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(EOF, String::from(""), None, self.line));
        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as i32
    }

    ///对每一个字符进行扫描，如果是符号，E.g. （, ), }, !, < 就在本函数进行处理，将符号化成token
    ///如果不是符号，有如下逻辑：
    ///
    ///  
    ///  if [`Scanner::is_digit`] => [`Scanner::number`]
    ///
    ///  if [`Scanner::is_alpha`] => [`Scanner::identifier`]
    ///  
    ///   对于异常，我们有：
    ///   _ => [`Lox::error_with_line`]
    ///
    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            ',' => self.add_token(COMMA),
            '.' => self.add_token(DOT),
            '-' => self.add_token(MINUS),
            '+' => self.add_token(PLUS),
            ';' => self.add_token(SEMICOLON),
            '*' => self.add_token(STAR),
            '!' => {
                let token_type = if self.match_char('=') {
                    BANG_EQUAL
                } else {
                    BANG
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    EQUAL_EQUAL
                } else {
                    EQUAL
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    LESS_EQUAL
                } else {
                    LESS
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    GREATER_EQUAL
                } else {
                    GREATER
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(SLASH);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string(),
            c if Scanner::is_digit(c) => self.number(),
            c if Scanner::is_alpha(c) => self.identifier(),
            _ => Lox::error_with_line(self.line, "Unexpected character."),
        }
    }

    ///识别TRUE，FALSE，NIL
    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start as usize..self.current as usize];
        let token_type = *KEYWORDS.get(text).unwrap_or(&IDENTIFIER);
        match token_type {
            TRUE => self.add_token_with_literal(TRUE, Some(Literal::Bool(true))),
            FALSE => self.add_token_with_literal(FALSE, Some(Literal::Bool(false))),
            NIL => self.add_token_with_literal(NIL, Some(Literal::Nil)),
            _ => self.add_token(token_type),
        }
    }
    ///识别字符串
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Lox::error_with_line(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let value = &self.source[self.start as usize + 1..self.current as usize - 1];
        self.add_token_with_literal(STRING, Some(Literal::String(value.to_string())));
    }

    ///识别数字
    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = &self.source[self.start as usize..self.current as usize];
        self.add_token_with_literal(NUMBER, Some(Literal::Number(value.parse().unwrap())));
    }

    /// 判断当前字符是否为expected，如果是，current指针后移一位
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current as usize).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    /// 查看当前字符，但不移动current指针
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current as usize).unwrap()
    }

    /// 预览下一个字符
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() as i32 {
            return '\0';
        }
        self.source
            .chars()
            .nth((self.current + 1) as usize)
            .unwrap()
    }

    /// 判断是否是字母
    fn is_alpha(c: char) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
    }

    /// 判断是否是字母或数字
    fn is_alphanumeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    /// 判断是否是数字
    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    /// 查看当前字符并将current指针后移一位
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source
            .chars()
            .nth((self.current - 1) as usize)
            .unwrap()
    }

    /// 添加token
    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start as usize..self.current as usize];
        self.tokens
            .push(Token::new(token_type, text.to_string(), None, self.line));
    }

    /// 添加带有字面量的token
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start as usize..self.current as usize];
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line));
    }
}
