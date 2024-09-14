mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod loxcallable;
mod loxfunction;
mod parser;
mod runtime_error;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod value;

use expr::Expr;
use interpreter::Interpreter;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use parser::Parser;
use scanner::Scanner;
use token::Token;
use token_type::TokenType;

struct Lox {
    interpreter: Interpreter,
    had_error: bool,
    had_runtime_error: bool,
}

static mut LOX: Lazy<Lox> = Lazy::new(Lox::new);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        Lox::run_file(args[1].clone());
    } else {
        Lox::run_prompt();
    }
}

impl Lox {
    pub(crate) fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(path: String) -> Result<(), std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        Self::run(source);
        if unsafe { LOX.had_error } {
            std::process::exit(65);
        }
        if unsafe { LOX.had_runtime_error } {
            std::process::exit(70);
        }
        Ok(())
    }

    pub fn run_prompt() -> Result<(), std::io::Error> {
        loop {
            print!("> ");
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            Self::run(line);
            unsafe {
                LOX.had_error = false;
                LOX.had_runtime_error = false;
            }
        }
    }

    pub fn run(source: String) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        let mut parser = parser::Parser::new(tokens);
        let statements = parser.parse();
        if unsafe { LOX.had_error } {
            return;
        }

        unsafe { LOX.interpreter.interpret(statements) }
    }

    pub(crate) fn runtime_error(error: runtime_error::RuntimeError) {
        eprintln!("{}\n[line {}]", error.message, error.token.line);
        unsafe {
            LOX.had_runtime_error = true;
        }
    }

    pub fn error_with_line(line: i32, message: &str) {
        Self::report(line, "", message);
    }

    pub fn error_with_token(token: Token, message: &str) {
        if token.token_type == TokenType::EOF {
            Self::report(token.line, " at end", message);
        } else {
            Self::report(token.line, &format!(" at ' {} '", token.lexeme), message);
        }
    }

    pub fn report(line: i32, location: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
        unsafe {
            LOX.had_error = true;
        }
    }
}
