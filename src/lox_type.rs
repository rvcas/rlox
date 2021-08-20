use std::fmt;

use crate::function::Function;

#[derive(Debug, Clone)]
pub enum LoxType {
    Boolean(bool),
    Callable(Function),
    Nil,
    Number(f64),
    String(String),
}

impl From<LoxType> for bool {
    fn from(value: LoxType) -> Self {
        use LoxType::*;

        match value {
            Boolean(b) => b,
            Nil => false,
            _ => true,
        }
    }
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        use LoxType::*;

        match (self, other) {
            (Boolean(n), Boolean(m)) => n == m,
            (Nil, Nil) => true,
            (Number(n), Number(m)) => n == m,
            (String(n), String(m)) => n == m,
            _ => false,
        }
    }
}

impl fmt::Display for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoxType::*;

        match self {
            Boolean(ref b) => write!(f, "{}", b),
            Callable(function) => write!(f, "{}", function),
            Number(ref n) => write!(f, "{}", n),
            String(ref s) => write!(f, "{}", s),
            Nil => write!(f, "nil"),
        }
    }
}
