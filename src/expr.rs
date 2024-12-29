use crate::interpreter::Object;
use crate::token::Token;
use std::fmt::Display;

pub enum Expr {
    Literal(Object),
    Unary { operator: Token, right: Box<Expr> },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    /*
    https://craftinginterpreters.com/appendix-ii.html#expression-statement
    AssignExpr(Assign expr);
    CallExpr(Call expr);
    GetExpr(Get expr);
    LogicalExpr(Logical expr);
    SetExpr(Set expr);
    SuperExpr(Super expr);
    ThisExpr(This expr);
    VariableExpr(Variable expr);
     */
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(value) => match value {
                Object::Number(n) => f.write_fmt(format_args!("{n:?}")),
                _ => f.write_fmt(format_args!("{value}")),
            },
            Expr::Unary { operator, right } => {
                f.write_fmt(format_args!("({} {right})", operator.lexeme))
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => f.write_fmt(format_args!("({} {left} {right})", operator.lexeme)),
            Expr::Grouping(expression) => f.write_fmt(format_args!("(group {})", expression)),
        }
    }
}