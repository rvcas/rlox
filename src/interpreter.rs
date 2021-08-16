use crate::{ast::Expr, lox, lox_type::LoxType, token::Token, token_type::TokenType};

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

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: &Expr) {
        match self.evaluate(expr) {
            Ok(value) => println!("{}", value),
            Err(err) => lox::runtime_error(err),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<LoxType, RuntimeError> {
        match expr {
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
