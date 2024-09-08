mod ast_printer;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod token_type;

use expr::Expr;
use parser::Parser;
use scanner::Scanner;
use token::Token;
use token_type::TokenType;

struct Lox {
    had_error: bool,
}

static mut LOX: Lox = Lox { had_error: false };

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
    pub fn run_file(path: String) -> Result<(), std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        Self::run(source);
        if unsafe { LOX.had_error } {
            std::process::exit(65);
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
            }
        }
    }

    pub fn run(source: String) {
        let mut scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.scan_tokens();
        match Parser::new(tokens).parse() {
            Err(parseerror) => {
                parseerror.error();

                if unsafe { LOX.had_error } {
                    return;
                }
            }
            Ok(expr) => {
                println!("{}", ast_printer::ExprVisitor.print(&expr));
            }
        }

        if unsafe { LOX.had_error } {
            return;
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