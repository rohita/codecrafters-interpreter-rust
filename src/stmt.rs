#![allow(dead_code)]
use std::fmt::Display;
use std::rc::Rc;
use crate::expr::Expr;
use crate::function::FunctionDeclaration;
use crate::token::Token;

/// Stmt is one of the two node types in the Abstract Syntax Tree (AST). 
/// These nodes are higher up than expression nodes in the tree. 
#[derive(Clone, Debug)]
pub enum Stmt {
    /// An expression statement is an expression followed by a semicolon ; 
    Expression { expression: Expr },

    /// A print statement evaluates an expression and displays the result to the user. 
    Print { expression: Expr },
    
    /// A variable declaration statement brings a new variable into the world.
    /// It stores the name token so we know what it’s declaring, along with the 
    /// initializer expression. (If there isn’t an initializer, that field is null.)
    Var { name: Token, initializer: Option<Expr> },
    
    /// Contains the list of statements that are inside the { } block. 
    Block { statements: Vec<Stmt> },
    
    /// An if statement has an expression for the condition, then a statement to 
    /// execute if the condition is truthy. Optionally, it may also have an else 
    /// keyword and a statement to execute if the condition is falsey. 
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    
    /// While has a parenthesized condition expression, then a statement for the body. 
    /// Here we can see why it’s nice to have separate base classes for expressions 
    /// and statements. The fields below make it clear that the condition is an 
    /// expression and the body is a statement.
    While { condition: Expr, body: Box<Stmt> },
    
    /// A function statement is declared with a name, a list of parameters, and its body.
    Function { decl: Rc<FunctionDeclaration> },
    
    /// We use the return keyword token for its location for error reporting, 
    /// and the value being returned, if any. 
    Return { keyword: Token, value: Option<Expr> },
    
    /// Stores the class’s name and the methods inside its body. Methods are represented 
    /// by the existing FunctionDeclaration struct that we use for function declaration. That 
    /// gives us all the bits of state that we need for a method: name, parameter list, and body.
    Class { name: Token, methods: Vec<Rc<FunctionDeclaration>> },
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
            Stmt::Class { .. } => { write!(f, "<Class>") }
        }
    }
}
