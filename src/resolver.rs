use crate::error::token_error;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;
use std::collections::HashMap;
use crate::function::FunctionDeclaration;

/// This is kind of step 2.5. After the parser produces the syntax tree, but 
/// before the interpreter starts executing it, we’ll do a single walk over 
/// the tree to "resolve" all the variables it contains. This variable resolution 
/// pass works like a sort of mini-interpreter. It walks the tree, visiting each 
/// node once, so its performance is O(n). This is also called 'static analysis'. 
pub struct Resolver {
    /// This field keeps track of the stack of scopes currently in scope.
    /// Each element in the stack is a Map representing a single block scope.
    /// Keys, as in Environment, are variable names. The values are Booleans, and
    /// represents whether we have finished resolving that variable’s initializer.
    /// 
    /// The scope stack is only used for local block scopes. Variables declared
    /// at the top level in the global scope are not tracked by the resolver
    /// since they are more dynamic in Lox. When resolving a variable, if we
    /// can’t find it in the stack of local scopes, we assume it must be global.
    /// 
    /// Rust doesn't have a Stack data structure. So we are using Vec, and its kinda 
    /// like reversed stack, where the 'top' is the at the end. The innermost scope
    /// is at the 'top' of this stack. 
    scopes: Vec<HashMap<String, bool>>,

    /// Keeps track of all the resolved variables 
    resolved: HashMap<*const Expr, usize>
}

impl Resolver {
    
    pub fn new() -> Resolver {
        Self {
            scopes: Vec::new(),
            resolved: HashMap::new()
        }
    }
    
    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> HashMap<*const Expr, usize> {
        self.resolve_block(statements);
        self.resolved.clone()
    }

    fn resolve_block(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }
    
    /// This method is similar to the execute() method in Interpreter — it 
    /// applies the Visitor pattern to the given syntax tree node. We handle every 
    /// place where a variable is declared, read, or written, and every place where 
    /// a scope is created or destroyed. 
    fn resolve_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                // Introduces a new scope for the statements it contains.
                self.begin_scope();
                self.resolve_block(statements);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                // Resolving a variable declaration adds a new entry to the current 
                // innermost scope’s map. We split binding into two steps, declaring 
                // then defining. This is to handle if the initializer for a local variable 
                // refers to a variable with the same name as the variable being declared. 
                self.declare(name);
                if let Some(expr) = initializer {
                    self.resolve_expression(expr);
                }
                self.define(name);
            }
            Stmt::Function { decl } => {
                // A function declaration introduces a new scope for its body and 
                // binds its parameters in that scope.
                self.declare(&decl.name);
                self.define(&decl.name); // This lets function recursively refer to itself inside its body.
                self.resolve_function(decl);
            }
            Stmt::Expression { expression } => {
                self.resolve_expression(expression);
            }
            Stmt::If { condition, then_branch, else_branch } => {
                // An if statement has an expression for its condition and one or two statements 
                // for the branches. Here, we see how resolution is different from interpretation. 
                // When we resolve an if statement, there is no control flow. We resolve the 
                // condition and both branches. Whereas a dynamic execution steps only into the 
                // branch that *is* run, a static analysis is conservative — it analyzes any branch 
                // that *could* be run. Since either one could be reached at runtime, we resolve both.
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Stmt::Print { expression } => {
                self.resolve_expression(expression);
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.resolve_expression(expr);
                }
            }
            Stmt::While { condition, body } => {
                // Same as `if` statements, we resolve condition and body exactly once.
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
        }
    }

    /// This method is similar to the evaluate() method in Interpreter — it 
    /// applies the Visitor pattern to the given syntax tree node.
    fn resolve_expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Variable { name } => {
                // It's a compile error if an initializer mentions the variable being initialized.
                // e.g. var a = a; 
                if self.scopes.last().and_then(|scope| scope.get(&name.lexeme)) == Some(&false) {
                    token_error(name.clone(), "Can't read local variable in its own initializer.".into());
                }
                self.resolve_local(expression, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expression(value);
                self.resolve_local(expression, name);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call { callee, arguments, .. } => {
                self.resolve_expression(callee);
                for argument in arguments {
                    self.resolve_expression(argument);
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expression(expression);
            }
            Expr::Literal { .. } => {
                // A literal expression doesn’t mention any variables and 
                // doesn’t contain any subexpressions so there is no work to do.
            }
            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Unary { right, .. } => {
                self.resolve_expression(right);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declaration adds the variable to the innermost scope so that it shadows any outer 
    /// one and so that we know the variable exists. We mark it as “not ready yet” by 
    /// binding its name to false in the scope map.
    fn declare(&mut self, name: &Token) {
        if let Some(innermost_scope) = self.scopes.last_mut() {
            innermost_scope.insert(name.lexeme.clone(), false);
        }
    }
    
    /// Sets the variable’s value in the scope map to true to mark it as fully 
    /// initialized and available for use. 
    fn define(&mut self, name: &Token) {
        if let Some(innermost_scope) = self.scopes.last_mut() {
            innermost_scope.insert(name.lexeme.clone(), true);
        }
    }

    /// We start at the innermost scope and work outwards, looking in each map for 
    /// a matching name. If we find the variable, we resolve it, passing in the number 
    /// of scopes between the current innermost scope and the scope where the variable was found. 
    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) { 
                //let ptr = expr as *const Expr;
                //eprintln!("Put Distance: ptr: {:?} name: {} lexeme: {} distance: {distance}", ptr, expr.to_string(), name.lexeme);
                self.resolved.insert(expr, distance);
                return;
            }
        }
    }

    /// Creates a new scope for the body and then binds variables for each of the function’s 
    /// parameters. This is different from how the interpreter handles function declarations. 
    /// At runtime, declaring a function doesn’t do anything with the function’s body. The 
    /// body doesn’t get touched until later when the function is called. In a static analysis, 
    /// we immediately traverse into the body right then and there.
    fn resolve_function(&mut self, function: &FunctionDeclaration) {
        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param)
        }
        self.resolve_block(&function.body);
        self.end_scope();
    }
}