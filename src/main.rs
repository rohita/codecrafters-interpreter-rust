pub mod environment;
pub mod error;
pub mod expr;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod stmt;
pub mod token;
pub mod value;
pub mod resolver;
pub mod lox;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

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
    //eprintln!("{file_contents}");

    match command.as_str() {
        "tokenize" => lox::tokenize(file_contents),
        "parse" => lox::parse(file_contents),
        "evaluate" => lox::evaluate(file_contents),
        "run" => lox::run(file_contents),
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