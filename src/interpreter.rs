use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{Expr, Stmt},
    class::{LoxClass, LoxInstance},
    environment::Environment,
    function::Function,
    lox,
    lox_type::LoxType,
    token::Token,
    token_type::TokenType,
};

pub enum InterpreterError {
    RuntimeError(RuntimeError),
    Return(LoxType),
}

impl InterpreterError {
    pub fn runtime_error(token: Option<Token>, message: &str) -> Self {
        Self::RuntimeError(RuntimeError::new(token, message))
    }
}

pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Option<Token>, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    env: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));

        env.borrow_mut().define(
            "clock",
            LoxType::Callable(Function::Native {
                arity: 0,
                body: |_| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|duration| LoxType::Number(duration.as_millis() as f64))
                        .map_err(|_| {
                            InterpreterError::runtime_error(None, "could not retrieve time.")
                        })
                },
            }),
        );

        Self {
            globals: Rc::clone(&env),
            env: Rc::clone(&env),
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(err) = self.execute(statement) {
                lox::runtime_error(err);

                break;
            }
        }
    }

    pub fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.clone(), depth);
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.execute_block(
                    stmts,
                    Rc::new(RefCell::new(Environment::with_enclosing(&self.env))),
                )?;
            }
            Stmt::Class { name, methods } => {
                self.env.borrow_mut().define(&name.lexeme, LoxType::Nil);

                let mut class_methods = HashMap::new();

                for method in methods {
                    if let Stmt::Function {
                        name: function_name,
                        params,
                        body,
                    } = method
                    {
                        let function = Function::User {
                            name: Box::new(function_name.clone()),
                            params: params.clone(),
                            body: body.clone(),
                            closure: Rc::clone(&self.env),
                            is_initializer: name.lexeme == "init",
                        };

                        class_methods.insert(function_name.lexeme.to_string(), function);
                    } else {
                        unreachable!()
                    }
                }

                let class = Rc::new(RefCell::new(LoxClass::new(&name.lexeme, class_methods)));

                self.env
                    .borrow_mut()
                    .assign(&name.lexeme, LoxType::Class(class));
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Function { name, body, params } => {
                let function = LoxType::Callable(Function::User {
                    name: Box::new(name.clone()),
                    body: body.to_vec(),
                    params: params.to_vec(),
                    closure: Rc::clone(&self.env),
                    is_initializer: false,
                });

                self.env.borrow_mut().define(&name.lexeme, function);
            }
            Stmt::If {
                condition,
                then_branch,
                opt_else_branch,
            } => {
                if bool::from(self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = opt_else_branch {
                    self.execute(else_branch)?
                }
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;

                println!("{}", value);
            }
            Stmt::Return { value, .. } => {
                let value = match *value {
                    Expr::Literal(LoxType::Nil) => LoxType::Nil,
                    _ => self.evaluate(value)?,
                };

                return Err(InterpreterError::Return(value));
            }
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;

                self.env.borrow_mut().define(&name.lexeme, value);
            }
            Stmt::While { condition, body } => {
                while bool::from(self.evaluate(condition)?) {
                    self.execute(body)?;
                }
            }
        }

        Ok(())
    }

    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), InterpreterError> {
        let previous = self.env.clone();

        let exec_stmts = || -> Result<(), InterpreterError> {
            self.env = env;

            for stmt in stmts {
                self.execute(stmt)?
            }

            Ok(())
        };

        let res = exec_stmts();

        self.env = previous;

        res
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxType, InterpreterError> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                let success = if let Some(distance) = self.locals.get(name) {
                    self.env
                        .borrow_mut()
                        .assign_at(*distance, &name.lexeme, value.clone())
                } else {
                    self.env.borrow_mut().assign(&name.lexeme, value.clone())
                };

                if success {
                    Ok(value)
                } else {
                    Err(InterpreterError::runtime_error(
                        Some(name.clone()),
                        &format!("Undefined variable '{}'.", name.lexeme),
                    ))
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let right_value = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Number(n - m))
                    }
                    TokenType::Plus => match (left_value, right_value) {
                        (LoxType::Number(n), LoxType::Number(m)) => Ok(LoxType::Number(n + m)),
                        (LoxType::String(mut n), LoxType::String(m)) => {
                            n.push_str(&m);

                            Ok(LoxType::String(n))
                        }
                        _ => Err(InterpreterError::runtime_error(
                            Some(operator.clone()),
                            "Operands must be two numbers or two strings.",
                        )),
                    },
                    TokenType::Slash => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Number(n / m))
                    }
                    TokenType::Star => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Number(n * m))
                    }
                    TokenType::Greater => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Boolean(n > m))
                    }
                    TokenType::GreaterEqual => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Boolean(n >= m))
                    }
                    TokenType::Less => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Boolean(n < m))
                    }
                    TokenType::LessEqual => {
                        let (n, m) =
                            Self::check_number_operands(operator.clone(), left_value, right_value)?;

                        Ok(LoxType::Boolean(n <= m))
                    }
                    TokenType::BangEqual => Ok(LoxType::Boolean(left_value != right_value)),
                    TokenType::EqualEqual => Ok(LoxType::Boolean(left_value == right_value)),
                    _ => unreachable!(),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee_value = self.evaluate(callee)?;

                let mut arguments_values = Vec::new();

                for argument in arguments {
                    arguments_values.push(self.evaluate(argument)?);
                }

                match callee_value {
                    LoxType::Callable(function) => {
                        if arguments_values.len() == function.arity() {
                            function.call(self, &arguments_values)
                        } else {
                            Err(InterpreterError::runtime_error(
                                Some(paren.clone()),
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity(),
                                    arguments_values.len()
                                ),
                            ))
                        }
                    }
                    LoxType::Class(class) => {
                        let instance = LoxInstance::new(&class);
                        let instance_type = LoxType::Instance(Rc::new(RefCell::new(instance)));

                        if let Some(initializer) = class.borrow().find_method("init") {
                            if arguments_values.len() == initializer.arity() {
                                initializer
                                    .bind(instance_type.clone())
                                    .call(self, &arguments_values)?;
                            } else {
                                return Err(InterpreterError::runtime_error(
                                    Some(paren.clone()),
                                    &format!(
                                        "Expected {} arguments but got {}.",
                                        initializer.arity(),
                                        arguments_values.len()
                                    ),
                                ));
                            }
                        }

                        Ok(instance_type)
                    }
                    _ => Err(InterpreterError::runtime_error(
                        Some(paren.clone()),
                        "Can only call functions and classes.",
                    )),
                }
            }
            Expr::Get { name, object } => {
                let object_value = self.evaluate(object)?;

                if let LoxType::Instance(ref instance) = object_value {
                    Ok(instance.borrow().get(name, &object_value)?)
                } else {
                    Err(InterpreterError::runtime_error(
                        Some(name.clone()),
                        "Only instances have properties.",
                    ))
                }
            }
            Expr::Grouping(grouped_expr) => self.evaluate(grouped_expr),
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;

                let is_left_truthy = bool::from(left_value.clone());

                if operator.token_type == TokenType::Or {
                    if is_left_truthy {
                        return Ok(left_value);
                    }
                } else {
                    if !is_left_truthy {
                        return Ok(left_value);
                    }
                }

                self.evaluate(right)
            }
            Expr::Set {
                name,
                object,
                value,
            } => {
                let object_value = self.evaluate(object)?;

                if let LoxType::Instance(instance) = object_value {
                    let value = self.evaluate(value)?;

                    instance.borrow_mut().set(name, value.clone());

                    Ok(value)
                } else {
                    Err(InterpreterError::runtime_error(
                        Some(name.clone()),
                        "Only instances have fields.",
                    ))
                }
            }
            Expr::This(keyword) => self.lookup_variable(keyword),
            Expr::Unary { operator, right } => {
                let right_value = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Bang => {
                        let b = bool::from(right_value);

                        Ok(LoxType::Boolean(!b))
                    }
                    TokenType::Minus => {
                        let n = Self::check_number_operand(operator.clone(), right_value)?;

                        Ok(LoxType::Number(-n))
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => self.lookup_variable(name),
        }
    }

    fn lookup_variable(&self, name: &Token) -> Result<LoxType, InterpreterError> {
        let opt_value = if let Some(distance) = self.locals.get(name) {
            self.env.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(&name.lexeme)
        };

        match opt_value {
            Some(value) => Ok(value),
            None => Err(InterpreterError::runtime_error(
                Some(name.clone()),
                &format!("Undefined variable '{}'.", name.lexeme),
            )),
        }
    }

    fn check_number_operand(token: Token, operand: LoxType) -> Result<f64, InterpreterError> {
        if let LoxType::Number(n) = operand {
            Ok(n)
        } else {
            Err(InterpreterError::runtime_error(
                Some(token),
                "Operand must be a number.",
            ))
        }
    }

    fn check_number_operands(
        token: Token,
        left: LoxType,
        right: LoxType,
    ) -> Result<(f64, f64), InterpreterError> {
        if let (LoxType::Number(n), LoxType::Number(m)) = (left, right) {
            Ok((n, m))
        } else {
            Err(InterpreterError::runtime_error(
                Some(token),
                "Operands must be numbers.",
            ))
        }
    }
}
