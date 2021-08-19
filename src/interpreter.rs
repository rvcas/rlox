use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    lox,
    lox_type::LoxType,
    token::Token,
    token_type::TokenType,
};

pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
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

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(stmts) => {
                let previous = self.env.clone();
                self.env = Environment::with_enclosing(Box::new(previous));

                for statement in stmts {
                    self.execute(statement)?;
                }

                if let Some(enclosing) = &self.env.enclosing {
                    self.env = *enclosing.clone();
                }
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
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

    fn evaluate(&mut self, expr: &Expr) -> Result<LoxType, RuntimeError> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                if self.env.assign(&name.lexeme, value.clone()) {
                    Ok(value)
                } else {
                    Err(RuntimeError::new(
                        name.clone(),
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
                        _ => Err(RuntimeError::new(
                            operator.clone(),
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
            Expr::Grouping(grouped_expr) => self.evaluate(grouped_expr),
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
                None => Err(RuntimeError::new(
                    name.clone(),
                    &format!("Undefined variable '{}'.", name.lexeme),
                )),
            },
        }
    }

    fn check_number_operand(token: Token, operand: LoxType) -> Result<f64, RuntimeError> {
        if let LoxType::Number(n) = operand {
            Ok(n)
        } else {
            Err(RuntimeError::new(token, "Operand must be a number."))
        }
    }

    fn check_number_operands(
        token: Token,
        left: LoxType,
        right: LoxType,
    ) -> Result<(f64, f64), RuntimeError> {
        if let (LoxType::Number(n), LoxType::Number(m)) = (left, right) {
            Ok((n, m))
        } else {
            Err(RuntimeError::new(token, "Operands must be numbers."))
        }
    }
}
