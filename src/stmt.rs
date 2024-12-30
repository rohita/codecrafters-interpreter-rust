use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    /*
    Class(Class stmt);
    Function(Function stmt);
    If(If stmt);
    Return(Return stmt);
    While(While stmt);
     */
}