mod scanner;
mod token;
mod expr;
mod parser;
mod error;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use crate::parser::Parser;
use crate::scanner::Scanner;

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
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }

    if error::failed() {
        exit(65);
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
    if let Some(expr) = parser.parse() {
        println!("{expr}");
    }
}

fn evaluate(file_contents: String) {
    let mut lexer = Scanner::new(file_contents);
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    if let Some(expr) = parser.parse() {
        let evaluated = expr.evaluate();
        println!("{evaluated}");
    }
}
