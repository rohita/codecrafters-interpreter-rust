use std::fmt::Display;
use crate::expr::Expr;
use crate::token::Token;

/// Our Abstract Syntax Tree (AST) consists of two types of 
/// nodes: Expr and Stmt. We split expression and statement 
/// syntax trees into two separate hierarchies because there’s 
/// no single place in the grammar that allows both an expression 
/// and a statement. Also, it’s nice to have separate enums 
/// for expressions and statements. E.g. In the field declarations 
/// of 'While' it is clear that the condition is an expression 
/// and the body is a statement.
#[derive(Clone, Debug)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        /// We store the body as the list of statements 
        /// contained inside the curly braces.
        body: Vec<Stmt>,
    },
    Return {
        /// Use token location for error reporting
        keyword: Token,
        value: Option<Expr>,
    },
    /*
    Class(Class stmt);
     */
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expression { .. } => { write!(f, "<Expression>") }, 
            Stmt::Print { .. } => { write!(f, "<Print>") },
            Stmt::Var { .. } => { write!(f, "<Var>") },
            Stmt::Block { .. } => { write!(f, "<Block>") },
            Stmt::If { .. } => { write!(f, "<If>") },
            Stmt::While { .. } => { write!(f, "<While>") },
            Stmt::Function { .. } => { write!(f, "<Function>") },
            Stmt::Return { .. } => { write!(f, "<Return>") },
        }
    }
}
