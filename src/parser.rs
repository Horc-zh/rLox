//!parser.rs 是用于进行语法分析的文件，将token流转换为 [`Stmt`]，这将用在[`crate::interpreter`]中

use crate::LoxResult;
use std::vec;

use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::token_type::TokenType;
use crate::token_type::TokenType::*;

///定义parser结构体
pub struct Parser {
    ///记录由scanner传递来的token流
    tokens: Vec<Token>,
    ///记录现在分析到的token
    current: usize,
}

///使用递归下降分析:
///
///对于本`impl` 的调用逻辑，详见[`crate`]的语法规则
///
///-------
///
///异常处理:
///
///如果发生异常，参与分析的函数都将返回[`LoxResult`]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    ///开始语法分析，把token流转化为语句
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
            F: FnOnce(&mut Parser) -> Result<Stmt, LoxResult>,
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

    ///对函数调用的token进行分析
    fn function(&mut self, kind: String) -> Result<Stmt, LoxResult> {
        let name = self.consume(IDENTIFIER, format!("Expect '(' after {} name.", kind))?;

        self.consume(LEFT_PAREN, format!("Expect '(' after {} name.", kind))?;
        let mut params = Vec::new();
        if !self.check(&RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    return Err(LoxResult::ParseError {
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

    ///对变量定义的token进行分析
    fn var_declaration(&mut self) -> Result<Stmt, LoxResult> {
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

    ///分析statement的token，包括[`TokenType::FOR`], [`TokenType::IF`],
    ///[`TokenType::PRINT`],[`TokenType::RETURN`],[`TokenType::WHILE`],[`TokenType::LEFT_BRACE`]
    ///
    ///如果以上[`TokenType`] 都不匹配,那么就进入[`Parser::expression_statement`] 函数
    fn statement(&mut self) -> Result<Stmt, LoxResult> {
        if self.match_token(&[FOR]) {
            return self.for_statement();
        }
        if self.match_token(&[IF]) {
            return self.if_statement();
        }
        if self.match_token(&[PRINT]) {
            return self.print_statement();
        }
        if self.match_token(&[RETURN]) {
            return self.return_statement();
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

    ///处理return语句
    fn return_statement(&mut self) -> Result<Stmt, LoxResult> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(&SEMICOLON) {
            value = Some(self.expression()?);
        }
        self.consume(SEMICOLON, "Expect ';' after return value".to_string())?;
        Ok(Stmt::Return { keyword, value })
    }

    ///处理for语句
    fn for_statement(&mut self) -> Result<Stmt, LoxResult> {
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

    ///处理while
    fn while_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(LEFT_PAREN, "Expect '(' after 'while'.".to_string())?;
        let condition = Box::new(self.expression()?);
        self.consume(RIGHT_PAREN, "Expect ')' after condition.".to_string())?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    ///处理if
    fn if_statement(&mut self) -> Result<Stmt, LoxResult> {
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

    ///处理大括号块
    fn block(&mut self) -> Result<Vec<Stmt>, LoxResult> {
        let mut stmts = Vec::new();
        while !self.check(&RIGHT_BRACE) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
        }

        self.consume(RIGHT_BRACE, "Expect '}' after a block".to_string())?;
        Ok(stmts)
    }

    ///处理print语句
    fn print_statement(&mut self) -> Result<Stmt, LoxResult> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value".to_string())?;
        Ok(Stmt::Print {
            expression: Box::new(expr),
        })
    }

    ///分析token，将其转换成[`Stmt::Expression`],并返回
    fn expression_statement(&mut self) -> Result<Stmt, LoxResult> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value".to_string())?;
        Ok(Stmt::Expression {
            expression: Box::new(expr),
        })
    }

    ///调用了[`Parser::assignment`]
    pub fn expression(&mut self) -> Result<Expr, LoxResult> {
        self.assignment()
    }

    ///分析赋值语句，返回[`Expr::Assign`]
    fn assignment(&mut self) -> Result<Expr, LoxResult> {
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
            return Err(LoxResult::ParseError {
                token: equals,
                message: "Invaild assignment target.".to_string(),
            });
        }
        Ok(expr)
    }

    ///处理or运算符
    fn or(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理and运算符
    fn and(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理 ==
    fn equality(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理比较运算符
    fn comparison(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理加减
    fn term(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理乘除
    fn factor(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理单目运算符
    fn unary(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理函数调用
    fn call(&mut self) -> Result<Expr, LoxResult> {
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

    ///处理 ), ) 表示一段程序,参数的结束
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxResult> {
        let mut arguments = Vec::new();
        if !self.check(&RIGHT_PAREN) {
            arguments.push(self.expression()?);
            while self.match_token(&[COMMA]) {
                if arguments.len() >= 255 {
                    return Err(LoxResult::ParseError {
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

    fn primary(&mut self) -> Result<Expr, LoxResult> {
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
        Err(LoxResult::ParseError {
            token: self.peek(),
            message: "Expect expression".to_string(),
        }
        .error())
    }

    ///查看当前分析的token是否在types中，用来决定下一步的分析走向
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    ///让下一个token进入分析
    ///
    ///如果下一个token与 [`TokenType`] 不匹配，则抛出异常
    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, LoxResult> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(LoxResult::ParseError {
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

    ///当parse出现错误时，会跳出当前语句，直到遇到下一个语句，以防止连环报错
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

#[cfg(test)]
mod test {

    // use crate::ast_printer;
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn test_parse_val() {
        let mut scanner = Scanner::new("var a = 1;\nprint a;".to_string());
        let tokens = scanner.scan_tokens();
        let mut parse = Parser::new(tokens.to_vec());
        let stmts = parse.parse();
        dbg!(stmts);
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
