use std::{collections::HashMap, mem};

use crate::{
    ast::{Expr, Stmt},
    interpreter::Interpreter,
    lox,
    token::Token,
};

#[derive(Clone)]
enum FunctionType {
    Function,
    Initializer,
    Method,
    None,
}

enum ClassType {
    Class,
    None,
    SubClass,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.resolve_statement(stmt);
        }
    }

    fn resolve_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();

                self.resolve(stmts);

                self.end_scope();
            }
            Stmt::Class {
                name,
                methods,
                opt_superclass,
            } => {
                let enclosing_class = mem::replace(&mut self.current_class, ClassType::Class);

                self.declare(name);
                self.define(name);

                if let Some(Expr::Variable(superclass_name)) = opt_superclass {
                    if name.lexeme == superclass_name.lexeme {
                        lox::parse_error(superclass_name, "A class can't inherit from itself.");
                    }

                    self.current_class = ClassType::SubClass;

                    self.resolve_local(superclass_name);

                    self.begin_scope();

                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert("super".to_string(), true);
                    }
                }

                self.begin_scope();

                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("this".to_string(), true);
                }

                for method in methods {
                    if let Stmt::Function {
                        body, params, name, ..
                    } = method
                    {
                        let mut declaration = FunctionType::Method;

                        if name.lexeme == "init" {
                            declaration = FunctionType::Initializer;
                        }

                        self.resolve_function(params, body, declaration);
                    }
                }

                self.end_scope();

                if opt_superclass.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
            }
            Stmt::Expression(expr) => {
                self.resolve_expression(expr);
            }
            Stmt::Function { body, name, params } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(params, body, FunctionType::Function);
            }
            Stmt::If {
                condition,
                then_branch,
                opt_else_branch,
            } => {
                self.resolve_expression(condition);

                self.resolve_statement(then_branch);

                if let Some(else_branch) = opt_else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Stmt::Print(expr) => {
                self.resolve_expression(expr);
            }
            Stmt::Return { value, keyword } => {
                if let FunctionType::None = self.current_function {
                    lox::parse_error(keyword, "Can't return from top-level code.")
                }

                if !value.is_nil() {
                    if let FunctionType::Initializer = self.current_function {
                        lox::parse_error(keyword, "Can't return a value from an initializer.");
                    }

                    self.resolve_expression(value);
                }
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);

                if !initializer.is_nil() {
                    self.resolve_expression(initializer);
                }

                self.define(name);
            }
            Stmt::While { body, condition } => {
                self.resolve_expression(condition);

                self.resolve_statement(body);
            }
        }
    }

    fn resolve_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expression(value);

                self.resolve_local(name);
            }
            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expression(callee);

                for arg in arguments {
                    self.resolve_expression(arg);
                }
            }
            Expr::Get { object, .. } => {
                self.resolve_expression(object);
            }
            Expr::Grouping(group) => {
                self.resolve_expression(group);
            }
            Expr::Literal(_) => (),
            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Set { object, value, .. } => {
                self.resolve_expression(value);
                self.resolve_expression(object);
            }
            Expr::Super { keyword, .. } => {
                match self.current_class {
                    ClassType::None => {
                        lox::parse_error(keyword, "Can't use 'super' outside of a class.");
                    }
                    ClassType::Class => {
                        lox::parse_error(
                            keyword,
                            "Can't use 'super' in a class with no superclass.",
                        );
                    }
                    ClassType::SubClass => (),
                };

                self.resolve_local(keyword);
            }
            Expr::This(keyword) => {
                if let ClassType::None = self.current_class {
                    lox::parse_error(keyword, "Can't use 'this' outside of a class.");
                } else {
                    self.resolve_local(keyword);
                }
            }
            Expr::Unary { right, .. } => {
                self.resolve_expression(right);
            }
            Expr::Variable(name) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(val) = scope.get(&name.lexeme) {
                        if !val {
                            lox::parse_error(
                                name,
                                "Can't read local variable in its own initializer.",
                            );
                        }
                    }
                }

                self.resolve_local(name);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                lox::parse_error(name, "Already a variable with this name in this scope.")
            }

            scope.insert(name.lexeme.to_string(), false);
        };
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_string(), true);
        }
    }

    fn resolve_local(&mut self, name: &Token) {
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name, index);

                return;
            }
        }
    }

    fn resolve_function(&mut self, params: &[Token], body: &[Stmt], function_type: FunctionType) {
        let enclosing_function = mem::replace(&mut self.current_function, function_type);

        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        self.resolve(body);

        self.end_scope();

        self.current_function = enclosing_function;
    }
}
