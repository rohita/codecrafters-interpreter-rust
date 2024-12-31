use crate::expr::Expr;
use crate::token::Token;

/// We can split expression and statement syntax trees into two 
/// separate hierarchies because there’s no single place in the 
/// grammar that allows both an expression and a statement. 
/// Also, it’s nice to have separate enums for expressions and statements. 
/// E.g. In the field declarations if 'While' it is clear that the 
/// condition is an expression and the body is a statement.
#[derive(Clone)]
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
    /*
    Class(Class stmt);
    Function(Function stmt);
    Return(Return stmt);
     */
}
