use crate::error;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;

pub fn tokenize(file_contents: String) {
    let mut scanner = Scanner::new(file_contents);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{}", token);
    }
}

pub fn parse(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    if let Ok(expr) = parser.expression() {
        println!("{expr}");
    }
}

pub fn evaluate(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    if let Ok(expr) = parser.expression() {
        let mut interpreter = Interpreter::new();
        match interpreter.evaluate(&expr) {
            Ok(evaluated) => println!("{evaluated}"),
            Err(error) => error::runtime_error(error),
        }
    }
}

pub fn run(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let mut resolver = Resolver::new();
    let locals = resolver.resolve(&stmts);

    // Stop if there was a resolution error.
    if error::had_error() {
        return;
    }

    let mut interpreter = Interpreter::new_with_resolver(locals);
    interpreter.interpret(&stmts);
}


