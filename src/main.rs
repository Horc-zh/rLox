/*!
这个项目使用rust编写，是一个`Tree-walk Interpret`, 处理了名为lox的语言，以下为lox的语法

```text
## Syntax Grammer
program        → declaration* EOF ;

## Declarations
declaration    →  funDecl
                | varDecl
                | statement ;

                  "{" function* "}" ;
funDecl        → "fun" function ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
statement      → exprStmt
               | forStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | block ;

exprStmt       → expression ";" ;
forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                           expression? ";"
                           expression? ")" statement ;
ifStmt         → "if" "(" expression ")" statement
                 ( "else" statement )? ;
printStmt      → "print" expression ";" ;
returnStmt     → "return" expression? ";" ;
whileStmt      → "while" "(" expression ")" statement ;
block          → "{" declaration* "}" ;

##Expressios
expression     → assignment ;

assignment     → ( call "." )? IDENTIFIER "=" assignment
               | logic_or ;

logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;

unary          → ( "!" | "-" ) unary | call ;
call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
primary        → "true" | "false" | "nil" | "this"
               | NUMBER | STRING | IDENTIFIER | "(" expression ")"
               | "super" "." IDENTIFIER ;

## Utility rules
function       → IDENTIFIER "(" parameters? ")" block ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
arguments      → expression ( "," expression )* ;

## Lexical Grammer
NUMBER         → DIGIT+ ( "." DIGIT+ )? ;
STRING         → "\"" <any char except "\"">* "\"" ;
IDENTIFIER     → ALPHA ( ALPHA | DIGIT )* ;
ALPHA          → "a" ... "z" | "A" ... "Z" | "_" ;
DIGIT          → "0" ... "9" ;```!*/
mod ast_printer;
pub mod environment;
pub mod expr;
pub mod interpreter;
pub mod loxcallable;
pub mod loxfunction;
pub mod loxresult;
pub mod parser;
pub mod scanner;
pub mod stmt;
pub mod token;
pub mod token_type;
pub mod value;

use interpreter::Interpreter;
use loxresult::LoxResult;
use once_cell::sync::Lazy;
use scanner::Scanner;
use token::Token;
use token_type::TokenType;

///定义lox结构体
struct Lox {
    ///整个解释器的环境
    interpreter: Interpreter,
    ///是否在编译期发生错误
    had_error: bool,
    ///是否在运行期发生错误
    had_runtime_error: bool,
}

///使用lazy初始化LOX变量，LOX变量是一个全局变量，在整个解释器运行期间存在
///它的类型是[`Lox`]
static mut LOX: Lazy<Lox> = Lazy::new(Lox::new);

///根据输入的参数个数进入不同的模式，如果参数个数小于二，那么进入本解释器的repl模式
pub fn main() {
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

///定义了Lox结构体的方法
impl Lox {
    pub(crate) fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    ///对文件进行解释
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

    ///执行解释器的repl模式
    pub fn run_prompt() -> Result<(), std::io::Error> {
        loop {
            // print!("> ");
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            Self::run(line);
            unsafe {
                LOX.had_error = false;
                LOX.had_runtime_error = false;
            }
        }
    }

    ///对lox语言进行编译与执行
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

    ///向stderr打印出发生执行期错误的行数
    pub(crate) fn runtime_error(error: LoxResult) {
        //123
        match error {
            LoxResult::RuntimeError { token, message }
            | LoxResult::ParseError { token, message } => {
                eprintln!("[line {}] {}  ", token.line, message)
            }
            _ => unreachable!(),
        }
        unsafe {
            LOX.had_runtime_error = true;
        }
    }

    ///打印错误信息，包含有行号
    pub fn error_with_line(line: i32, message: &str) {
        Self::report(line, "", message);
    }

    ///打印错误信息，包含有无法解析的字符token
    pub fn error_with_token(token: &Token, message: &str) {
        if token.token_type == TokenType::EOF {
            Self::report(token.line, " at end", message);
        } else {
            Self::report(token.line, &format!(" at ' {} '", token.lexeme), message);
        }
    }
    ///打印出发生编译器错误的行数
    pub fn report(line: i32, location: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
        unsafe {
            LOX.had_error = true;
        }
    }
}
