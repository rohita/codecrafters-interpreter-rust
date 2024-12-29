use crate::error;
use crate::error::Error;
use crate::error::Error::ParseError;
use crate::evaluator::Object;
use crate::expr::Expr;
use crate::token::{Token, TokenType};

#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => None
        }
    }
    
    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }
    
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_types(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right)
            };
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_types(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right)
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_types(vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right)
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_types(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right)
            }; 
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_types(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {operator, right: Box::from(right)});
        }
        
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_types(vec![TokenType::FALSE]) {
            return Ok(Expr::Literal(Object::Boolean(false)));
        }
        if self.match_types(vec![TokenType::TRUE]) {
            return Ok(Expr::Literal(Object::Boolean(true)));
        }
        if self.match_types(vec![TokenType::NIL]) {
            return Ok(Expr::Literal(Object::Nil));
        }
        
        if self.match_types(vec![TokenType::NUMBER]) {
            let num = self.previous().literal.clone().unwrap().parse().unwrap();
            return Ok(Expr::Literal(Object::Number(num)));
        }
        if self.match_types(vec![TokenType::STRING]) {
            let string = self.previous().literal.clone().unwrap();
            return Ok(Expr::Literal(Object::String(string)));
        }
        
        if self.match_types(vec![TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            return match self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping(Box::from(expr))),
                Err(err) => Err(err),
            }
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), message))
    }

    fn match_types(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }
    
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    
    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    fn error(&self, token: Token, message: &str) -> Error {
        error::error_token(token, message.to_string()); 
        ParseError
    }
}