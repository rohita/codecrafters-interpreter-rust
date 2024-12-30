use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Var(Token, Option<Expr>),
    /*
    Class(Class stmt);
    Block(Block stmt);
    Function(Function stmt);
    If(If stmt);
    Return(Return stmt);
    While(While stmt);
     */
}