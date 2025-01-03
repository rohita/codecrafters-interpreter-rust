use crate::object::Object;
use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn error(line: usize, message: String) {
    report(line, "".to_string(), message);
}

pub fn runtime_error(error: Error) {
    match error {
        Error::RuntimeError(token, message) => {
            eprintln!("{}\n[line {}]", message, token.line);
            unsafe {
                HAD_RUNTIME_ERROR = true;
            }
        }
        _ => unreachable!(),
    }
}

pub fn error_token(token: Token, message: String) {
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

pub fn had_error() -> bool {
    unsafe { HAD_ERROR }
}

pub fn had_runtime_error() -> bool {
    unsafe { HAD_RUNTIME_ERROR }
}

pub enum Error {
    ParseError,
    RuntimeError(Token, String),
    Return(Object),
}
