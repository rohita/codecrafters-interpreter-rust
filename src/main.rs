mod environment;
mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod object;
mod function;
mod resolver;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use crate::resolver::Resolver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {filename}");
        exit(65);
    });

    match command.as_str() {
        "tokenize" => tokenize(file_contents),
        "parse" => parse(file_contents),
        "evaluate" => evaluate(file_contents),
        "run" => run(file_contents),
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }

    if error::had_error() {
        exit(65);
    }
    if error::had_runtime_error() {
        exit(70);
    }
}

fn tokenize(file_contents: String) {
    let mut scanner = Scanner::new(file_contents);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{}", token);
    }
}

fn parse(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    if let Ok(expr) = parser.expression() {
        println!("{expr}");
    }
}

fn evaluate(file_contents: String) {
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

fn run(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let mut resolver = Resolver::new();
    let locals = resolver.resolve(&stmts);
    let mut interpreter = Interpreter::new_with_resolver(locals);
    interpreter.interpret(&stmts);
}
