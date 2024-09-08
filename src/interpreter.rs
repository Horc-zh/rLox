use crate::{
    expr::Expr, runtime_error::RuntimeError, token::Token, token_type::TokenType, value::Value,
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        //TODO: should I put env into this struct?
        Interpreter {}
    }

    fn check_number_operands(
        operator: &Token,
        left: &Value,
        right: &Value,
    ) -> Result<(), RuntimeError> {
        if let (Value::Number(_), Value::Number(_)) = (left, right) {
            return Ok(());
        }
        Err(RuntimeError {
            token: operator.clone(),
            message: format!("Operand must be a number"),
        })
    }

    pub fn evaluate(expr: Expr) -> Result<Value, RuntimeError> {
        Ok(match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = Interpreter::evaluate(*left)?;
                let right = Interpreter::evaluate(*right)?;

                match operator.token_type {
                    TokenType::PLUS => {
                        if left != right {
                            return Err(RuntimeError {
                                token: operator,
                                message: format!("Operands must be two numbers or two strings."),
                            });
                        }
                        left + right
                    }
                    TokenType::MINUS => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        left - right
                    }
                    TokenType::STAR => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        left * right
                    }
                    TokenType::SLASH => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        left / right
                    }
                    TokenType::EQUAL_EQUAL => Value::Boolean(left == right),
                    TokenType::BANG_EQUAL => Value::Boolean(left != right),
                    TokenType::GREATER => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        Value::Boolean(left > right)
                    }
                    TokenType::GREATER_EQUAL => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        Value::Boolean(left >= right)
                    }
                    TokenType::LESS => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        Value::Boolean(left < right)
                    }
                    TokenType::LESS_EQUAL => {
                        Interpreter::check_number_operands(&operator, &left, &right)?;
                        Value::Boolean(left <= right)
                    }

                    _ => unreachable!(),
                }
            }
            Expr::Grouping { expression } => Interpreter::evaluate(*expression)?,
            Expr::Literal { value } => match value {
                crate::token::Literal::String(s) => Value::String(s),
                crate::token::Literal::Number(n) => Value::Number(n),
                crate::token::Literal::Bool(b) => Value::Boolean(b),
                crate::token::Literal::Nil => Value::Nil,
            },
            Expr::Unary { operator, right } => {
                let right_value = Interpreter::evaluate(*right)?;
                match operator.token_type {
                    TokenType::MINUS => -right_value,
                    TokenType::BANG => !right_value,
                    _ => unreachable!(),
                }
            }
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::parser::Parser;
    use crate::Scanner;

    fn get_value(s: &str) -> Value {
        Interpreter::evaluate(
            Parser::new(Scanner::new(s.to_string()).scan_tokens())
                .parse()
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn test_eval_equal_not_equal() {
        assert_eq!(get_value("\"hello\" == \"hello\""), Value::Boolean(true));
        assert_eq!(get_value("\"hello\" == \"hello!\""), Value::Boolean(false));
    }

    #[test]
    fn test_eval_complex_expression() {
        assert_eq!(get_value("1+2 * (8/4)"), Value::Number(5.0));
    }

    #[test]
    fn test_eval_binary() {
        assert_eq!(get_value("1+2"), Value::Number(3.0));
        assert_eq!(get_value("1-1"), Value::Number(0.0));
        assert_eq!(get_value("10*10"), Value::Number(100.0));
        assert_eq!(get_value("5/2"), Value::Number(2.5));
    }

    #[test]
    fn test_eval_unary() {
        assert_eq!(get_value("-1"), Value::Number(-1.0));
        assert_eq!(get_value("!(true)"), Value::Boolean(false));
    }

    #[test]
    fn test_eval_literal() {
        let value = Value::Boolean(true);
        assert_eq!(value, get_value("(true)"))
    }
}
