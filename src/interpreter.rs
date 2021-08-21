use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    ast::{Expr, Stmt},
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
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();

        env.define(
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

        Self { env }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(err) = self.execute(statement) {
                lox::runtime_error(err);

                break;
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.execute_block(
                    stmts,
                    Environment::with_enclosing(Box::new(self.env.clone())),
                )?;
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Function { name, body, params } => {
                let function = LoxType::Callable(Function::User {
                    name: Box::new(name.clone()),
                    body: body.to_vec(),
                    params: params.to_vec(),
                });

                self.env.define(&name.lexeme, function);
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

                self.env.define(&name.lexeme, value);
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
        env: Environment,
    ) -> Result<(), InterpreterError> {
        self.env = env;

        for statement in stmts {
            self.execute(statement).map_err(|err| {
                if let Some(enclosing) = &self.env.enclosing {
                    self.env = *enclosing.clone();
                }

                err
            })?;
        }

        if let Some(enclosing) = &self.env.enclosing {
            self.env = *enclosing.clone();
        }

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxType, InterpreterError> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                if self.env.assign(&name.lexeme, value.clone()) {
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
                        if arguments_values.len() != function.arity() {
                            Err(InterpreterError::runtime_error(
                                Some(paren.clone()),
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity(),
                                    arguments_values.len()
                                ),
                            ))
                        } else {
                            function.call(self, &arguments_values)
                        }
                    }
                    _ => Err(InterpreterError::runtime_error(
                        Some(paren.clone()),
                        "Can only call functions and classes.",
                    )),
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
            Expr::Variable(name) => match self.env.get(&name.lexeme) {
                Some(value) => Ok(value),
                None => Err(InterpreterError::runtime_error(
                    Some(name.clone()),
                    &format!("Undefined variable '{}'.", name.lexeme),
                )),
            },
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
