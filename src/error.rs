use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;

pub fn error(line: usize, message: String) {
    report(line, "".to_string(), message);
}

pub fn error_token(token: Token, message: String ) {
    if token.token_type == TokenType::EOF {
        report(token.line, " at end".to_string(), message);
    } else {
        report(token.line, format!(" at '{}'", token.lexeme), message);
    }
}

fn report(line: usize, wh: String, message: String) {
    eprintln!("[line {}] Error{}: {}", line, wh, message);
    unsafe {
        HAD_ERROR = true;
    }
}

pub fn failed() -> bool {
    unsafe {
        HAD_ERROR
    }
}

pub enum Error {
    ParseError,
}