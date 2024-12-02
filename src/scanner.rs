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
            self.scan_token()
        }
        self.tokens.push(Token::new(EOF, String::new(), None, self.line));
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
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
            _ => {
                eprintln!("[line {}] Error: Unexpected character: {}", ln, c);
                self.had_error = true;
            }
        }
    }

    fn advance(&mut self) -> Option<&char> {
        let res = self.source.get(self.current);
        self.current += 1;
        res
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(token_type, text, None, self.line));
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
}