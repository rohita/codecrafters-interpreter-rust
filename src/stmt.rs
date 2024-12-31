use crate::expr::Expr;
use crate::token::Token;

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
    /*
    Class(Class stmt);
    Function(Function stmt);
    Return(Return stmt);
    While(While stmt);
     */
}
