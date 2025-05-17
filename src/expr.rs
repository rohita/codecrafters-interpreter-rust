use crate::object::Object;
use crate::token::Token;
use std::fmt::Display;

/// Expr is the base class that all expression types inherit from.
/// It's a one of the two node types in the Abstract Syntax Tree (AST). 
#[derive(Clone, Debug)]
pub enum Expr {
    /// The leaves of an expression tree — the atomic bits of syntax 
    /// that all other expressions are composed of — are literals. 
    /// Literals are almost values already, but the distinction is 
    /// important. A literal is a bit of syntax that produces a "value". 
    /// A literal always appears somewhere in the user’s source code, 
    /// but values are produced by computation and don’t exist anywhere 
    /// in the code itself. Those computed values aren’t literals. A literal 
    /// comes from the parser’s domain. Values are an interpreter concept, 
    /// part of the runtime world.
    Literal { value: Object },
    
    /// Unary expressions have a single operator followed by a subexpression
    Unary { operator: Token, right: Box<Expr> },
    
    /// Binary expression have a left and right with a single operator in the middle
    Binary { left: Box<Expr>, operator: Token, right: Box<Expr> },
    
    /// A Grouping node represents parentheses in an expression. It has a 
    /// reference to an inner node for the expression contained inside the 
    /// parentheses.
    Grouping { expression: Box<Expr> },
    
    /// Simple wrapper around the token for the variable name. 
    Variable { name: Token },
    
    /// Token for the variable being assigned to, and an expression for the new value. 
    /// The classic terms for these two constructs are l-value and r-value. An l-value 
    /// “evaluates” to a storage location that we assign into. That’s why this has a 
    /// Token for the left-hand side, not an Expr. 
    Assign { name: Token, value: Box<Expr> },
    
    /// Represents OR and AND. We could reuse the existing Expr.Binary for these two 
    /// since they have the same fields. But then we would have to check to see if the 
    /// operator is one of the logical operators and use a different code path. It’s 
    /// cleaner to define a new class for these operators so that they get their evaluate 
    /// method.
    Logical { left: Box<Expr>, operator: Token, right: Box<Expr> },
    
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
