use crate::error;
use crate::error::Error;
use crate::error::Error::ParseError;
use crate::expr::Expr;
use crate::interpreter::Object;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use TokenType::*;

#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
        }
        stmts
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let try_value = {
            if self.match_types(vec![VAR]) {
                self.var_declaration()
            } else {
                self.statement()
            }
        };

        match try_value {
            Ok(value) => Some(value),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(IDENTIFIER, "Expect variable name")?;
        let mut initializer: Option<Expr> = None;
        if self.match_types(vec![EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(SEMICOLON, "Expect ';' after variable declaration")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_types(vec![IF]) {
            return self.if_statement();
        }
        if self.match_types(vec![PRINT]) {
            return self.print_statement();
        }

        if self.match_types(vec![LEFT_BRACE]) {
            return self.block();
        }

        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);

        // We solve the 'dangling else' problem by choosing the rule:
        // the 'else' is bound to the nearest 'if' that precedes it.
        // Since we eagerly looks for an else before returning, the
        // innermost call to a nested series will claim the else clause
        // for itself before returning to the outer if statements.
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_types(vec![ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression })
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression })
    }

    fn block(&mut self) -> Result<Stmt, Error> {
        let mut statements = Vec::new();

        while !self.check(RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration().unwrap());
        }

        self.consume(RIGHT_BRACE, "Expect '}' after block.")?;
        Ok(Stmt::Block { statements })
    }

    pub fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.equality()?;

        if self.match_types(vec![EQUAL]) {
            let equals = self.previous();
            let value = Box::from(self.assignment()?);
            match expr {
                Expr::Variable{name} => {
                    return Ok(Expr::Assign { name, value });
                }
                _ => return Err(self.error(equals, "Invalid assignment target.")),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_types(vec![BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_types(vec![GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_types(vec![MINUS, PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_types(vec![SLASH, STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                operator,
                left: Box::from(expr),
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_types(vec![BANG, MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::from(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_types(vec![FALSE]) {
            return Ok(Expr::Literal { value: Object::Boolean(false) });
        }
        if self.match_types(vec![TRUE]) {
            return Ok(Expr::Literal { value: Object::Boolean(true) });
        }
        if self.match_types(vec![NIL]) {
            return Ok(Expr::Literal { value: Object::Nil });
        }
        if self.match_types(vec![NUMBER]) {
            let num = self.previous().literal.clone().unwrap().parse().unwrap();
            return Ok(Expr::Literal { value: Object::Number(num) });
        }
        if self.match_types(vec![STRING]) {
            let string = self.previous().literal.clone().unwrap();
            return Ok(Expr::Literal { value: Object::String(string) });
        }
        if self.match_types(vec![IDENTIFIER]) {
            return Ok(Expr::Variable { name: self.previous().clone() });
        }

        if self.match_types(vec![LEFT_PAREN]) {
            let expr = self.expression()?;
            return match self.consume(RIGHT_PAREN, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping { expression: Box::from(expr) }),
                Err(err) => Err(err),
            };
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
        self.peek().token_type == EOF
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

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }

            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => return,
                _ => {}
            }

            self.advance();
        }
    }
}
