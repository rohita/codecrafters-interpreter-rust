use crate::expr::Expr;

pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    /*
    Class(Class stmt);
    Block(Block stmt);
    Function(Function stmt);
    If(If stmt);
    Return(Return stmt);
    Var(Var stmt);
    While(While stmt);
     */
}