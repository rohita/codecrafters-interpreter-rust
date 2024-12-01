use crate::token::{Token, TokenType};
use crate::token::TokenType::*;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
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
        while self.current < self.source.len() {
            self.start = self.current;
            self.scan_token()
        }
        self.tokens.push(Token::new(EOF, String::new(), None, self.line));
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance().unwrap();
        match c {
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            _ => panic!("{}", format!("invalid token: {c} ({})", *c as u32)),
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
}