use crate::error;
use crate::token::TokenType::*;
use crate::token::{Token, TokenType};
use std::collections::HashMap;

/// The first step in any compiler or interpreter is scanning. The scanner
/// takes in raw source code as a series of characters and groups it into
/// a series of chunks we call tokens. These are the meaningful “words” and
/// “punctuation” that make up the language’s grammar. 
pub struct Scanner {
    /// The raw source code
    source: Vec<char>,

    /// A list to fill with tokens the scanner is going to generate
    tokens: Vec<Token>,

    /// These fields are used to keep track of where the scanner is in the source code.
    /// 'start' points to the first character in the lexeme being scanned.
    /// 'current' points at the character currently being considered.
    /// 'line' field tracks what source line current is on.
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: vec![],
            current: 0,
            start: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(EOF, String::new(), None, self.line));
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Scans a single token. This is the real heart of the scanner.
    /// We could define a regex for each kind of lexeme and using
    /// those to match characters. But our goal is to understand how
    /// a scanner works, so we won’t be delegating that task.
    fn scan_token(&mut self) {
        let ln = self.line;
        let c = self.advance().unwrap();
        match c {
            // --------Single-character lexemes ----------------------
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            ',' => self.add_token(COMMA),
            '.' => self.add_token(DOT),
            '-' => self.add_token(MINUS),
            '+' => self.add_token(PLUS),
            ';' => self.add_token(SEMICOLON),
            '*' => self.add_token(STAR),

            // --------Two-character Operators ----------------------
            // We recognize these lexemes in two stages. e.g. we know
            // the lexeme starts with !. We look at the next
            // character to determine if we’re on a != or merely a !.
            '!' => match self.match_next('=') {
                true => self.add_token(BANG_EQUAL),
                false => self.add_token(BANG),
            },
            '=' => match self.match_next('=') {
                true => self.add_token(EQUAL_EQUAL),
                false => self.add_token(EQUAL),
            },
            '<' => match self.match_next('=') {
                true => self.add_token(LESS_EQUAL),
                false => self.add_token(LESS),
            },
            '>' => match self.match_next('=') {
                true => self.add_token(GREATER_EQUAL),
                false => self.add_token(GREATER),
            },

            // --------Newline and Whitespaces ----------------------
            // We simply ignore whitespace character. For newlines, we
            // do the same thing, but we also increment the line counter.
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,

            // --------Longer Lexemes ----------------------------------
            // This is our general strategy for handling longer lexemes.
            // After we detect the beginning of one, we shunt over to
            // some lexeme-specific code that keeps eating characters
            // until it sees the end.
            '/' => self.comment(),
            '"' => self.string(),
            d if is_digit(*d) => self.number(),
            a if is_alpha(*a) => self.identifier(),

            // --------Invalid characters -------------------------------------
            // We log error and keep scanning. There may be other errors later
            // in the program. We detect as many of those as possible in one go.
            // Otherwise, users will see one tiny error and fix it, only to have
            // the next error appear, and so on.
            _ => {
                error::error(ln, format!("Unexpected character: {}", c));
            }
        }
    }

    fn comment(&mut self) {
        // Comment goes until the end of the line. Comments
        // are lexemes, but they aren’t meaningful. When we
        // reach the end of the comment, we don’t call addToken().
        if self.match_next('/') {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else {
            self.add_token(SLASH)
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            // Lox supports multi-line strings
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string.".to_string());
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes to produce the actual string
        // value that will be used later by the interpreter.
        let value: String = self.source[self.start + 1..self.current - 1].iter().collect();
        self.add_token_with_literal(STRING, Option::from(value));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let mut value: String = self.source[self.start..self.current].iter().collect();
        let my_int: f64 = value.parse().unwrap();
        value = format!("{:?}", my_int);
        self.add_token_with_literal(NUMBER, Option::from(value));
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type: TokenType = keywords().get(&*text).unwrap_or(&IDENTIFIER).clone();
        self.add_token(token_type);
    }

    /// Consumes the next character in the source file and returns it
    fn advance(&mut self) -> Option<&char> {
        let res = self.source.get(self.current);
        self.current += 1;
        res
    }

    /// Grabs the text of the current lexeme and creates a new token for it
    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    /// Grabs the text of the current lexeme and creates a new token, along with its literal value
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(token_type, text, literal, self.line));
    }

    /// It’s like a conditional advance(). We only consume the
    /// current character if it’s what we’re looking for.
    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    /// Like advance(), but doesn’t consume the character. This is also called lookahead.
    /// Since it only looks at the current unconsumed character, we have one character of lookahead.
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn keywords() -> HashMap<&'static str, TokenType> {
    HashMap::from([
        ("and", AND),
        ("class", CLASS),
        ("else", ELSE),
        ("false", FALSE),
        ("for", FOR),
        ("fun", FUN),
        ("if", IF),
        ("nil", NIL),
        ("or", OR),
        ("print", PRINT),
        ("return", RETURN),
        ("super", SUPER),
        ("this", THIS),
        ("true", TRUE),
        ("var", VAR),
        ("while", WHILE),
    ])
}
