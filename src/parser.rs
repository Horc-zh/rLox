use std::vec;

use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::*;
use crate::Lox;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Some(stmt) => statements.push(stmt),
                None => {}
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        fn parse_with_recovery<F>(parser: &mut Parser, parse_fn: F) -> Option<Stmt>
        where
            F: FnOnce(&mut Parser) -> Result<Stmt, ParseError>,
        {
            match parse_fn(parser) {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    parser.synchronize();
                    None
                }
            }
        }
        if self.match_token(&[FUN]) {
            return parse_with_recovery(self, |p| p.function("function".to_string()));
        }

        if self.match_token(&[VAR]) {
            return parse_with_recovery(self, |p| p.var_declaration());
        }

        parse_with_recovery(self, |p| p.statement())
    }

    fn function(&mut self, kind: String) -> Result<Stmt, ParseError> {
        let name = self.consume(IDENTIFIER, format!("Expect '(' after {} name.", kind))?;

        self.consume(LEFT_PAREN, format!("Expect '(' after {} name.", kind))?;
        let mut params = Vec::new();
        if !self.check(&RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek(),
                        message: "Can't have more than 255 parameters.".to_string(),
                    }
                    .error());
                }
                params.push(self.consume(IDENTIFIER, "Expect parameter name.".to_string())?);

                if !self.match_token(&[COMMA]) {
                    break;
                }
            }
        }
        self.consume(RIGHT_PAREN, "Expect ')' after parameters.".to_string())?;
        self.consume(LEFT_BRACE, format!("Expect '{{' before {} body", kind))?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(IDENTIFIER, "Expect variable name.".to_string())?;
        let mut initializer = None;
        if self.match_token(&[EQUAL]) {
            initializer = Some(Box::new(self.expression()?));
        }
        self.consume(
            SEMICOLON,
            "Expect ';' after variable declaration.".to_string(),
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[FOR]) {
            return self.for_statement();
        }
        if self.match_token(&[IF]) {
            return self.if_statement();
        }
        if self.match_token(&[PRINT]) {
            return self.print_statement();
        }
        if self.match_token(&[WHILE]) {
            return self.while_statement();
        }
        if self.match_token(&[LEFT_BRACE]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LEFT_PAREN, "Expect '(' after 'for'.".to_string())?;
        let initializer = if self.match_token(&[SEMICOLON]) {
            None
        } else if self.match_token(&[VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = None;
        if !self.check(&SEMICOLON) {
            condition = Some(self.expression()?);
        }
        self.consume(SEMICOLON, "Expect ';' after loop condition.".to_string())?;

        let mut increment = None;
        if !self.check(&RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }
        self.consume(RIGHT_PAREN, "Expect ')' after for clause.".to_string())?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: Box::new(increment),
                    },
                ],
            }
        }

        let condition = condition.unwrap_or(Expr::Literal {
            value: Literal::Bool(true),
        });

        body = Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        };
        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }
        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LEFT_PAREN, "Expect '(' after 'while'.".to_string())?;
        let condition = Box::new(self.expression()?);
        self.consume(RIGHT_PAREN, "Expect ')' after condition.".to_string())?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LEFT_PAREN, "Expect '(' after 'if'.".to_string())?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after if condition.".to_string())?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_token(&[ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&RIGHT_BRACE) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
        }

        self.consume(RIGHT_BRACE, "Expect '}' after a block".to_string())?;
        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value".to_string())?;
        Ok(Stmt::Print {
            expression: Box::new(expr),
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value".to_string())?;
        Ok(Stmt::Expression {
            expression: Box::new(expr),
        })
    }

    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_token(&[EQUAL]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(ParseError {
                token: equals,
                message: "Invaild assignment target.".to_string(),
            });
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_token(&[OR]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[AND]) {
            let operator = self.previous();
            let right = self.equality()?;

            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
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
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[LEFT_PAREN]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&RIGHT_PAREN) {
            arguments.push(self.expression()?);
            while self.match_token(&[COMMA]) {
                if arguments.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek(),
                        message: "Can't have more than 255 parameters.".to_string(),
                    }
                    .error());
                }

                arguments.push(self.expression()?);
            }
        }
        let paren = self.consume(RIGHT_PAREN, "Expect ')' after arguments.".to_string())?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
        if self.match_token(&[IDENTIFIER]) {
            return Ok(Expr::Variable {
                name: self.previous(),
            });
        }
        if self.match_token(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expect ')' after expression.".to_string())?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }
        Err(ParseError {
            token: self.peek(),
            message: "Expect expression".to_string(),
        }
        .error())
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

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                token: self.peek(),
                message,
            }
            .error())
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
    message: String,
}

impl ParseError {
    pub fn error(self) -> Self {
        {
            Lox::error_with_token(self.token.clone(), self.message.as_str());
            self
        }
    }
}

#[cfg(test)]
mod test {

    use crate::ast_printer;
    use crate::scanner::Scanner;

    use super::*;

    // #[test]
    // fn test_vec() {
    //     let mut body = 1;
    //     let mut i = 0;
    //     while i < 5 {
    //         i = i + 1;
    //         body = vec![body, i];
    //     }
    // }

    #[test]
    fn test_parse_val() {
        let mut scanner = Scanner::new("var a = 1;\nprint a;".to_string());
        let tokens = scanner.scan_tokens();
        let mut parse = Parser::new(tokens.to_vec());
        let stmts = parse.parse();
        assert!(false)
    }

    #[test]
    fn test_parse_into_stmt() {
        let mut scanner = Scanner::new("print true; \"hello\";".to_string());
        let tokens = scanner.scan_tokens();
        let mut parse = Parser::new(tokens.to_vec());
        let stmts = parse.parse();
        assert!(false)
    }

    #[test]
    fn test_parse_true_false_nil() {
        let mut scanner = Scanner::new("(1 + 1) - 1".to_string());
        let tokens = scanner.scan_tokens();
        let mut parse = Parser::new(tokens.to_vec());
        let a = parse.parse();
        assert!(false)
    }
}
