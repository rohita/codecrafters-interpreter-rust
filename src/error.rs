use crate::object::Object;
use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn error(line: usize, message: String) {
    report(line, "".to_string(), message);
}

/// If a runtime error is thrown while evaluating the expression, interpret()
/// catches it. This lets us report the error to the user and then gracefully continue.
/// We use the token associated with the RuntimeError to tell the user what
/// line of code was executing when the error occurred.
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

/// This reports an error at a given token. It shows the token’s location and the
/// token itself. This comes in handy since we use tokens throughout the interpreter
/// to track locations in code.
pub fn token_error(token: Token, message: String) {
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
    /// These are syntax errors, used by parser for unwinding and synchronizing.
    /// These are detected and reported before any code is executed.
    ParseError,

    /// Runtime errors are failures that the language semantics demand we detect
    /// and report while the program is running. These are used by the interpreter.
    /// This tracks the token that identifies where in the user’s code the runtime
    /// error came from. As with parsing errors, this helps the user know where to fix their code.
    RuntimeError(Token, String),

    ///
    Return(Object),
}
