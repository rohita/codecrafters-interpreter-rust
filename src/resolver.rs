use crate::error::token_error;
use crate::expr::Expr;
use crate::stmt::{Stmt, FunctionDeclaration};
use crate::token::Token;
use std::collections::HashMap;


#[derive(Clone, Copy, Debug)]
enum FunctionType {
    None, Function, Method, Initializer,
}

#[derive(Clone, Copy, Debug)]
enum ClassType {
    None, Class
}

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
    resolved: HashMap<*const Expr, usize>,

    /// Much like we track scopes as we walk the tree, this is used to track whether the 
    /// code we are currently visiting is inside a function declaration.
    current_function: FunctionType,
    
    /// This is used to track whether we are inside a class declaration
    /// while traversing the syntax tree. 
    current_class: ClassType,
}

impl Resolver {
    
    pub fn new() -> Resolver {
        Self {
            scopes: Vec::new(),
            resolved: HashMap::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
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
            Stmt::Class { name, superclass, methods } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;
                
                self.declare(name);
                self.define(name);
                
                // Resolve superclass if it exists
                if let Some(superclass) = superclass {
                    if let Expr::Variable {name: superclass_name} = superclass {
                        if name.lexeme == superclass_name.lexeme {
                            token_error(superclass_name.clone(), "A class can't inherit from itself.".into());
                        }
                    }
                    
                    self.resolve_expression(superclass);
                    
                    // If the class declaration has a superclass, then we create a new scope 
                    // surrounding all of its methods. In that scope, we define the name “super”.
                    self.begin_scope();
                    if let Some(innermost_scope) = self.scopes.last_mut() {
                        innermost_scope.insert("super".into(), true);
                    }
                }
                
                // Before we step in and start resolving the method bodies, we push a 
                // new scope and define “this” in it as if it were a variable. Then, 
                // whenever "this" expression is encountered inside a method, it will resolve 
                // to a “local variable” defined in an implicit scope just outside the block 
                // for the method body.
                self.begin_scope();
                if let Some(innermost_scope) = self.scopes.last_mut() {
                    innermost_scope.insert("this".into(), true);
                }
                
                for method in methods {
                    let mut declaration = FunctionType::Method;
                    if method.name.lexeme == "init" {
                        declaration = FunctionType::Initializer;
                    }
                    self.resolve_function(method, declaration);
                }
                
                self.end_scope();
                
                // Once we’re done resolving the class’s methods, we discard 'super' scope.
                if let Some(_) = superclass {
                    self.end_scope();
                }
                
                self.current_class = enclosing_class;
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
                self.resolve_function(decl, FunctionType::Function);
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
            Stmt::Return { keyword, value } => {
                if let FunctionType::None = self.current_function {
                    token_error(keyword.clone(), "Can't return from top-level code.".into());
                }
                
                if let Some(expr) = value {
                    if let FunctionType::Initializer = self.current_function {
                        token_error(keyword.clone(), "Can't return a value from an initializer.".into());
                    }
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
            Expr::Get { object, .. } => {
                // Since properties are looked up dynamically, they don’t get resolved. 
                // During resolution, we recurse only into the expression to the left 
                // of the dot. The actual property access happens in the interpreter.
                self.resolve_expression(object);
            }
            Expr::Set { object, value, .. } => {
                // Like Get, the property itself is dynamically evaluated, so there’s 
                // nothing to resolve there. All we need to do is recurse into the two 
                // subexpressions of Set, the object whose property is being set, 
                // and the value it’s being set to.
                self.resolve_expression(value);
                self.resolve_expression(object);
            }
            Expr::Super { keyword, .. } => {
                // The resolution stores the number of hops along the environment chain 
                // that the interpreter needs to walk to find the environment where the 
                // superclass is stored.
                self.resolve_local(expression, keyword);
            }
            Expr::This { keyword } => {
                if let ClassType::None = self.current_class {
                    token_error(keyword.clone(), "Can't use 'this' outside of a class.".into());
                    return;
                }
                
                // this works like a variable
                self.resolve_local(expression, keyword);    
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
        let lexeme = name.lexeme.clone();
        if let Some(innermost_scope) = self.scopes.last_mut() {
            if innermost_scope.contains_key(&lexeme) {
                token_error(name.clone(), "Already a variable with this name in this scope.".into());
            }
            
            innermost_scope.insert(lexeme, false);
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
    fn resolve_function(&mut self, function: &FunctionDeclaration, function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        
        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param)
        }
        self.resolve_block(&function.body);
        self.end_scope();
        
        self.current_function = enclosing_function;
    }
}