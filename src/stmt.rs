use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If {condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>},
    /*
    Class(Class stmt);
    Function(Function stmt);
    Return(Return stmt);
    While(While stmt);
     */
}