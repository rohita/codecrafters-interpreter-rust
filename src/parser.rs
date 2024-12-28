use crate::expr::Expr;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
    
    pub fn parse(&self) -> Option<Vec<Expr>> {
        if self.tokens.is_empty() {
            return None;
        }
        let mut exprs = Vec::new();
        let tokens = self.tokens.iter().peekable();
        for token in tokens {
            let expr = match token.token_type {
                TokenType::TRUE => Some(Expr::Bool(true)),
                TokenType::FALSE => Some(Expr::Bool(false)),
                TokenType::NIL => Some(Expr::Nil),
                TokenType::NUMBER => Some(Expr::Number(token.literal.clone().unwrap().parse().unwrap())),
                TokenType::EOF => return Some(exprs),
                _ => None,
            };
            exprs.push(expr?);
        }
        Some(exprs)
    }
}