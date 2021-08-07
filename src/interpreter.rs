use std::convert::TryFrom;

use crate::{ast::Expr, lox_type::LoxType, token_type::TokenType};

pub struct InterpreterError;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<LoxType, InterpreterError> {
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
                        let n = f64::try_from(left_value)?;
                        let m = f64::try_from(right_value)?;

                        let result = n - m;

                        Ok(LoxType::Number(result))
                    }
                    TokenType::Plus => match (left_value, right_value) {
                        (LoxType::Number(n), LoxType::Number(m)) => {
                            let result = n + m;

                            Ok(LoxType::Number(result))
                        }
                        (LoxType::String(mut n), LoxType::String(m)) => {
                            n.push_str(&m);

                            Ok(LoxType::String(n))
                        }
                        _ => Err(InterpreterError),
                    },
                    TokenType::Slash => {
                        let n = f64::try_from(left_value)?;
                        let m = f64::try_from(right_value)?;

                        let result = n / m;

                        Ok(LoxType::Number(result))
                    }
                    TokenType::Star => {
                        let n = f64::try_from(left_value)?;
                        let m = f64::try_from(right_value)?;

                        let result = n * m;

                        Ok(LoxType::Number(result))
                    }
                    TokenType::Greater => Ok(LoxType::Boolean(left_value > right_value)),
                    TokenType::GreaterEqual => Ok(LoxType::Boolean(left_value >= right_value)),
                    TokenType::Less => Ok(LoxType::Boolean(left_value < right_value)),
                    TokenType::LessEqual => Ok(LoxType::Boolean(left_value <= right_value)),
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
                        let n = f64::try_from(right_value)?;

                        Ok(LoxType::Number(-n))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
