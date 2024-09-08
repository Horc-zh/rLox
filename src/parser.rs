use core::panic;

use crate::expr::Expr;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::*;
use crate::{Lox, LOX};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // pub fn parse(&mut self) -> Vec<Expr> {
    //     let mut exprs = Vec::new();
    //     while !self.is_at_end() {
    //         if let Ok(expr) = self.expression() {
    //             exprs.push(expr);
    //         } else {
    //             panic!("parse error")
    //         }
    //     }
    //     exprs
    // }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.match_token(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_token(&[MINUS, PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_token(&[SLASH, STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[BANG, MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[FALSE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token(&[TRUE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token(&[NIL]) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }
        if self.match_token(&[NUMBER, STRING]) {
            return Ok(Expr::Literal {
                value: self.previous().literal.clone().unwrap(),
            });
        }
        if self.match_token(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }
        Err(ParseError {
            token: self.peek(),
            message: "Expect expression",
        })
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: &'static str,
    ) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                token: self.peek(),
                message,
            })
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }
            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => return,
                _ => (),
            }
            self.advance();
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    message: &'static str,
}

impl ParseError {
    pub fn error(&self) {
        {
            Lox::error_with_token(self.token.clone(), self.message)
        }
    }
}

#[cfg(test)]
mod test {

    use crate::ast_printer;
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn test_parse_true_false_nil() {
        let mut scanner = Scanner::new("(1 + 1) - 1".to_string());
        let tokens = scanner.scan_tokens();
        let mut parse = Parser::new(tokens.to_vec());
        let a = parse.parse();
        dbg!(a);
        assert!(false)
    }
}
