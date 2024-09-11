use crate::{
    environment::Environment, expr::Expr, loxcallable::LoxCallable, parser::ParseError,
    runtime_error::RuntimeError, stmt::Stmt, token::Token, token_type::TokenType, value::Value,
    Lox,
};

pub struct Interpreter {
    globals: Environment,
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        //TODO: implement native function like clock
        Interpreter {
            globals,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        statements.into_iter().for_each(|stmt| {
            if let Err(e) = self.execute(stmt) {
                Lox::runtime_error(e);
                return;
            }
        })
    }
    //TODO: change the function signature otherwise there are bugs in whlie loop
    fn execute(&mut self, stmt: Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Print { expression } => {
                let value = self.evaluate(*expression)?;
                println!("{}", value);
                Ok(Value::Nil)
            }
            Stmt::Expression { expression } => Ok(self.evaluate(*expression)?),
            Stmt::Var { name, initializer } => {
                let mut value = Value::Nil;
                if let Some(initializer) = initializer {
                    value = self.evaluate(*initializer)?;
                }
                self.globals.define(name.lexeme, value);
                Ok(Value::Nil)
            }
            Stmt::Block { statements } => {
                //WARNING: the return value of new_enclosing is not correct in function execute_block
                // dbg!(&self.environment);
                // dbg!(&Environment::new_enclosing(self.environment.clone()));
                self.execute_block(statements, Environment::new_enclosing(self.globals.clone()))
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let value = self.evaluate(*condition)?;
                match value {
                    Value::Boolean(bool) => {
                        if bool {
                            self.execute(*then_branch)?;
                        } else {
                            if let Some(else_stmt) = else_branch {
                                self.execute(*else_stmt)?;
                            }
                        }
                    }
                    _ => unreachable!(), //TODO: add the exception handler
                }
                Ok(Value::Nil)
            }
            Stmt::While { condition, body } => {
                while self.evaluate(*condition.clone())?.is_true() {
                    self.execute(*body.clone())?;
                }
                Ok(Value::Nil)
            }
            _ => todo!(),
        }
    }

    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: Environment,
    ) -> Result<Value, RuntimeError> {
        let previous = std::mem::replace(&mut self.globals, environment); //useless
        for stmt in statements {
            if let Err(e) = self.execute(stmt) {
                self.globals = previous;
                return Err(e);
            }
        }
        if let Some(previous) = self.globals.get_enclosing_env() {
            self.globals = *previous;
        }
        Ok(Value::Nil)
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

    pub fn evaluate(&mut self, expr: Expr) -> Result<Value, RuntimeError> {
        Ok(match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match operator.token_type {
                    TokenType::PLUS => match (&left, &right) {
                        (Value::Number(_), Value::Number(_))
                        | (Value::String(_), Value::String(_)) => left + right,
                        _ => {
                            return Err(RuntimeError {
                                token: operator,
                                message: format!("Operands must be two numbers or two strings."),
                            })
                        }
                    },
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
            Expr::Grouping { expression } => self.evaluate(*expression)?,
            Expr::Literal { value } => match value {
                crate::token::Literal::String(s) => Value::String(s),
                crate::token::Literal::Number(n) => Value::Number(n),
                crate::token::Literal::Bool(b) => Value::Boolean(b),
                crate::token::Literal::Nil => Value::Nil,
            },
            Expr::Unary { operator, right } => {
                let right_value = self.evaluate(*right)?;
                match operator.token_type {
                    TokenType::MINUS => -right_value,
                    TokenType::BANG => !right_value,
                    _ => unreachable!(),
                }
            }
            Expr::Variable { name } => self.globals.get(name)?,
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.globals.assign(name, value.clone())?;
                value
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;
                if operator.token_type == TokenType::OR {
                    if left.is_true() {
                        return Ok(left);
                    } else {
                        if !left.is_true() {
                            return Ok(left);
                        }
                    }
                }
                self.evaluate(*right)?
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(*callee)?;
                let mut parameters = Vec::new();
                for argument in arguments {
                    parameters.push(self.evaluate(argument)?);
                }
                //TODO: implement the type checking : whether callee implement the trait,
                //loxcallable

                let function: Box<dyn LoxCallable>;
                function = Box::new(callee);

                if parameters.len() != function.arity() {
                    return Err(RuntimeError {
                        token: paren,
                        message: format!(
                            "Expect {} arguments but got {}.",
                            function.arity(),
                            parameters.len()
                        ),
                    });
                }

                let value = function.call(self, parameters);
                return Ok(value);
            }

            _ => todo!(),
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::parser::Parser;
    use crate::Scanner;

    fn get_value(s: &str) -> Value {
        let mut interpreter = Interpreter::new();
        interpreter
            .evaluate(
                Parser::new(Scanner::new(s.to_string()).scan_tokens())
                    .expression()
                    .unwrap(),
            )
            .unwrap()
    }

    #[test]
    fn test_eval_variable() {
        assert_eq!(get_value("var a = 1;\nprint a;"), Value::Number(1.0));
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
