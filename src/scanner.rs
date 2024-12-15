use crate::token::{Token, TokenType};
use crate::token::TokenType::*;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    pub had_error: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: vec![],
            current: 0,
            start: 0,
            line: 1,
            had_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(); 
        }
        self.tokens.push(Token::new(EOF, String::new(), None, self.line));
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let ln = self.line;
        let c = self.advance().unwrap();
        match c {
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
            '/' => if self.match_next('/') {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            } else {
                self.add_token(SLASH)
            },
            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => {}
            '"' => self.string(),
            d if Self::is_digit(*d) => self.number(),
            a if Self::is_alpha(*a) => self.identifier(),
            _ => {
                Self::report(ln, "".to_string(), format!("Unexpected character: {}", c));
                self.had_error = true;
            }
        }
    }
    
    fn identifier(&mut self) {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        
        self.add_token(IDENTIFIER);
    }

    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance();

            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let mut value: String = self.source[self.start..self.current].iter().collect();
        let my_int: f64 = value.parse().unwrap();
        value = format!("{:?}", my_int);
        self.add_token_with_literal(NUMBER, Option::from(value));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Self::report(self.line, "".to_string(), "Unterminated string.".to_string());
            self.had_error = true;
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value: String = self.source[self.start+1..self.current-1].iter().collect();
        self.add_token_with_literal(STRING, Option::from(value));
    }

    fn advance(&mut self) -> Option<&char> {
        let res = self.source.get(self.current);
        self.current += 1;
        res
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(token_type, text, literal, self.line));
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let res = self.source.get(self.current).unwrap();

        if *res != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

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

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }
    
    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn report(line: usize, wh: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, wh, message);
    }
}