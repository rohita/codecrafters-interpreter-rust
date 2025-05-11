use crate::object::Object;
use crate::token::Token;
use std::fmt::Display;

/// Expr is the base class that all expression types inherit from.
/// It's a one of the two node types in the Abstract Syntax Tree (AST). 
#[derive(Clone, Debug)]
pub enum Expr {
    Literal {
        value: Object,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        /// closing parenthesis location is used to
        /// report a runtime error caused by a function call
        paren: Token,
    },
    /*
    To be implemented:
    GetExpr(Get expr);
    LogicalExpr(Logical expr);
    SetExpr(Set expr);
    SuperExpr(Super expr);
    ThisExpr(This expr);
     */
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal { value } => match value {
                Object::Number(n) => f.write_fmt(format_args!("{n:?}")),
                _ => f.write_fmt(format_args!("{value}")),
            },
            Expr::Unary { operator, right } => {
                f.write_fmt(format_args!("({} {right})", operator.lexeme))
            }
            Expr::Binary { left, operator, right } => {
                f.write_fmt(format_args!("({} {left} {right})", operator.lexeme))
            },
            Expr::Grouping { expression } => f.write_fmt(format_args!("(group {})", expression)),
            Expr::Variable { name } => f.write_fmt(format_args!("(var {})", name.lexeme)),
            Expr::Assign { name, value } => {
                f.write_fmt(format_args!("(= {} {})", name.lexeme, value))
            },
            Expr::Logical { left, operator, right } => {
                f.write_fmt(format_args!("({} {left} {right})", operator.lexeme))
            },
            Expr::Call { callee, arguments, paren: _ } => {
                let string_vec = arguments.into_iter().map(Expr::to_string).collect::<Vec<String>>();
                f.write_fmt(format_args!("({callee} {})", string_vec.join(" ")))
            }
        }
    }
}
