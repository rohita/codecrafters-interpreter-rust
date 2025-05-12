use crate::error;
use crate::error::Error;
use crate::error::Error::ParseError;
use crate::expr::Expr;
use crate::object::Object;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use TokenType::*;

/// Parsing is the second step in compiler. Like the scanner, the parser consumes a
/// flat input sequence, only now we’re reading tokens instead of characters, and returns
/// a corresponding *Abstract Syntax Tree (AST)* to be passed on to the interpreter.
/// This AST consists of two types of nodes: `Expr` and `Stmt`. We split expression and
/// statement syntax trees into two separate hierarchies because there’s no single place in
/// the grammar that allows both an expression and a statement. Also, it’s nice to have
/// separate classes for expressions and statements. E.g. In the field declarations of
/// 'While' it is clear that the condition is an expression and the body is a statement.
///
/// `while ( expression ) statement`
///
/// There is a whole pack of parsing techniques LL(k), LR(1), LALR, etc. For our interpreter,
/// we will use *Recursive Descent*.
///
/// Recursive descent is considered a *top-down parser* because it starts from the top
/// or outermost grammar rule and works its way down into the nested subexpressions
/// before finally reaching the leaves of the syntax tree. This is in contrast with
/// bottom-up parsers like LR that start with primary expressions and compose them
/// into larger and larger chunks of syntax. Recursive descent is the simplest
/// way to build a parser. It is fast, robust, and can support sophisticated error handling.
///
/// A recursive descent parser is a literal translation of the grammar’s rules straight
/// into imperative code. Each rule becomes a method inside this class. Each method for
/// parsing a grammar rule produces a syntax tree for that rule and returns it to the caller.
/// When the body of the rule contains a *nonterminal* — a reference to another rule — we call
/// that other rule’s method. When a grammar rule refers to itself — directly or indirectly —
/// that translates to a recursive function call (that's why it's called “recursive”).
#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// This is the starting point for the grammar and represents a complete Lox script. 
    /// It parses a series of statements, as many as it can find until it hits the end.
    /// program → statement* EOF ;
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
            if self.match_token([FUN]) {
                self.function("function")
            } else if self.match_token([VAR]) {
                self.var_declaration()
            } else {
                self.statement()
            }
        };

        match try_value {
            Ok(value) => Some(value),
            Err(_) => {
                // As soon as the parser detects an error, it enters panic mode. It knows at least
                // one token doesn’t make sense given its current state in the middle of some stack
                // of grammar productions. So before it can get back to parsing, it needs to get its
                // state and the sequence of forthcoming tokens aligned such that the next token does
                // match the rule being parsed. This process is called **synchronization**.
                //
                // To do that, we select some rule in the grammar that will mark the synchronization
                // point. The parser fixes its parsing state by jumping out of any nested productions
                // until it gets back to that rule. Then it "synchronizes" the token stream by discarding
                // tokens until it reaches one that can appear at that point in the rule.
                //
                // The traditional place in the grammar to synchronize is between statements. That's
                // where we’ll catch the `ParserError` exception. After the exception is caught, the
                // parser is in the right rule. All that’s left is to synchronize the tokens.
                self.synchronize();
                None
            }
        }
    }
    
    /// This parses functions and methods (inside classes). We’ll pass in "function" or “method” 
    /// for kind so that the error messages are specific to the kind of declaration being parsed.
    fn function(&mut self, kind: &str) -> Result<Stmt, Error> {
        let name = self.consume(IDENTIFIER, format!("Expect {kind} name").as_str())?;
        self.consume(LEFT_PAREN, format!("Expect '(' after {kind} name.").as_str())?;
        let mut parameters = Vec::new();
        if !self.check(RIGHT_PAREN) {
            loop {
                if parameters.len() > 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(IDENTIFIER, "Expect parameter name.")?);
                
                if !self.match_token([COMMA])  {
                    break;
                }
            }
        }
        self.consume(RIGHT_PAREN, "Expect ')' after parameters.")?;
        
        self.consume(LEFT_BRACE, format!("Expect '{{' before {kind} body.").as_str())?;
        let body = self.block()?;
        Ok(Stmt::Function {name, params: parameters, body})
    }

    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(IDENTIFIER, "Expect variable name")?;
        let mut initializer: Option<Expr> = None;
        if self.match_token([EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(SEMICOLON, "Expect ';' after variable declaration")?;
        Ok(Stmt::Var { name, initializer })
    }

    // ---------------------------------------------
    // Statements
    // ---------------------------------------------

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_token([FOR]) {
            return self.for_statement();
        }
        if self.match_token([IF]) {
            return self.if_statement();
        }
        if self.match_token([PRINT]) {
            return self.print_statement();
        }
        if self.match_token([RETURN]) {
            return self.return_statement();
        }
        if self.match_token([WHILE]) {
            return self.while_statement();
        }
        if self.match_token([LEFT_BRACE]) {
            let statements = self.block()?;
            return Ok(Stmt::Block { statements });
        }

        self.expression_statement()
    }
    
    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LEFT_PAREN, "Expect '(' after 'for'.")?;
        
        // If the token following the '(' is a semicolon then the initializer 
        // has been omitted. Otherwise, we check for a var keyword to see if 
        // it’s a variable declaration. If neither of those matched, it must 
        // be an expression. We parse that and wrap it in an expression statement 
        // so that the initializer is always of type Stmt.
        let initializer: Option<Stmt>;
        if self.match_token([SEMICOLON]) {
            initializer = None;
        } else if self.match_token([VAR]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }
        
        // Next up is the condition. Again, we look for a semicolon 
        // to see if the clause has been omitted. 
        let mut condition: Option<Expr> = None;
        if !self.check(SEMICOLON) {
            condition = Some(self.expression()?);
        }
        self.consume(SEMICOLON, "Expect ';' after loop condition.")?;
        
        // The last clause is the increment. It’s similar to the condition 
        // clause except this one is terminated by the closing parenthesis.
        let mut increment: Option<Expr> = None;
        if !self.check(RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }
        self.consume(RIGHT_PAREN, "Expect ')' after for clauses.")?;
        
        // All that remains is the body.
        let mut body = self.statement()?;
        
        // We’ve parsed all the various pieces of the for loop and the resulting 
        // AST nodes are sitting in a handful of local variables. This is where the 
        // desugaring comes in. We take those and use them to synthesize syntax tree 
        // nodes that express the semantics of the for loop into a while loop.
        
        // Working backwards, we start with the increment clause. The increment, 
        // if there is one, executes after the body in each iteration of the loop. 
        // We do that by replacing the body with a little block that contains the 
        // original body followed by an expression statement that evaluates the increment.
        if let Some(increment) = increment {
            let increment_stmt = Stmt::Expression { expression: increment };
            body = Stmt::Block { statements: vec![body, increment_stmt] }
        }
        
        // Next, we take the condition and the body and build the loop using a 
        // primitive while loop. If the condition is omitted, we jam in 'true' 
        // to make an infinite loop.
        if condition.is_none() {
            condition = Some(Expr::Literal { value: Object::Boolean(true) });
        }
        body = Stmt::While { condition: condition.unwrap(), body: Box::new(body) };
        
        // Finally, if there is an initializer, it runs once before the entire loop. 
        // We do that by, again, replacing the whole statement with a block that runs 
        // the initializer and then executes the loop.
        if let Some(initializer) = initializer {
            body = Stmt::Block { statements: vec![initializer, body] }
        }
        
        // That’s it. We now supports 'for loops' and we didn’t have to touch 
        // the Interpreter class at all. Since we converted 'for' to a 'while',
        // which the interpreter already knows how to visit, there is no more work to do.
        Ok(body)
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
        if self.match_token([ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// printStmt → "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression })
    }
    
    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(SEMICOLON) {
            value = Some(self.expression()?);
        }
        self.consume(SEMICOLON, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }
    
    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While {condition, body: Box::new(body)})
    }

    /// exprStmt → expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expression = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression { expression })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();

        while !self.check(RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration().unwrap());
        }

        self.consume(RIGHT_BRACE, "Expect '}' after block.")?;
        Ok(statements)
    }

    // ---------------------------------------------
    // Expressions
    // ---------------------------------------------

    pub fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;

        if self.match_token([EQUAL]) {
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
    
    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;
        
        while self.match_token([OR]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::from(expr),
                operator,
                right: Box::from(right)
            };
        }
        
        Ok(expr)
    }
    
    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;
        
        while self.match_token([AND]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::from(expr),
                operator,
                right: Box::from(right)
            };
        }
        
        Ok(expr)
    }

    // ----Binary operators-----------------------------

    /// equal or not-equal
    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_token([BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    /// less than and greater than
    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_token([GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    /// addition and subtraction
    /// term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_token([MINUS, PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary { 
                left: Box::from(expr), 
                operator, 
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    /// multiplication and division
    /// factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_token([SLASH, STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right: Box::from(right),
            };
        }

        Ok(expr)
    }

    // ----Unary operators-----------------------------

    /// unary → ( "!" | "-" ) unary | call ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_token([BANG, MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::from(right),
            });
        }

        self.call()
    }
    
    fn call(&mut self) -> Result<Expr, Error> {
        let mut callee = self.primary()?;
        loop {
            if self.match_token([LEFT_PAREN]) {
                callee = self.finish_call(callee)?;
            } else {
                break;
            }
        }
        Ok(callee)
    }
    
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = Vec::new();
        if !self.check(RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.match_token([COMMA]) {
                    break;
                }
            }
        }
        let paren = self.consume(RIGHT_PAREN, "Expect ')' after arguments.")?;
        Ok(Expr::Call { callee: Box::from(callee), paren, arguments })
    }

    /// These are the "terminals"
    /// primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_token([FALSE]) {
            return Ok(Expr::Literal { value: Object::Boolean(false) });
        }
        if self.match_token([TRUE]) {
            return Ok(Expr::Literal { value: Object::Boolean(true) });
        }
        if self.match_token([NIL]) {
            return Ok(Expr::Literal { value: Object::Nil });
        }
        if self.match_token([NUMBER]) {
            let num = self.previous().literal.clone().unwrap().parse().unwrap();
            return Ok(Expr::Literal { value: Object::Number(num) });
        }
        if self.match_token([STRING]) {
            let string = self.previous().literal.clone().unwrap();
            return Ok(Expr::Literal { value: Object::String(string) });
        }
        if self.match_token([IDENTIFIER]) {
            return Ok(Expr::Variable { name: self.previous() });
        }

        if self.match_token([LEFT_PAREN]) {
            let expr = self.expression()?;
            return match self.consume(RIGHT_PAREN, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping { expression: Box::from(expr) }),
                Err(err) => Err(err),
            };
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    // ---------------------------------------------
    // Parsing infrastructure
    // ---------------------------------------------

    /// This checks to see if the current token has any of the given types.
    /// If so, it consumes the token and returns true. Otherwise, it returns
    /// false and leaves the current token alone.
    fn match_token<const N: usize>(&mut self, types: [TokenType; N]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// It’s similar to match() in that it checks to see if the next token
    /// is of the expected type. If so, it consumes the token and everything
    /// is groovy. If some other token is there, then we’ve hit an error.
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), message))
    }

    /// This method returns true if the current token is of the given type.
    /// Unlike match(), it never consumes the token, it only looks at it.
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    /// The advance() method consumes the current token and returns it.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous()
    }

    /// Checks if we’ve run out of tokens to parse.
    fn is_at_end(&self) -> bool {
        self.peek().token_type == EOF
    }

    /// Returns the current token we have yet to consume
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    /// Returns the most recently consumed token.
    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    /// This reports the error and returns 'ParserError'. It does not throw because
    /// we want to let the calling method decide whether to unwind or not.
    fn error(&self, token: Token, message: &str) -> Error {
        error::token_error(token, message.to_string());
        ParseError
    }

    /// We want to discard tokens until we’re right at the beginning of the next statement.
    /// That boundary is after a semicolon. Most statements start with a keyword — for, if,
    /// return, var, etc. When the next token is any of those, we’re probably about to start
    /// a statement.
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
