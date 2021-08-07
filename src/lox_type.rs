use std::{cmp::Ordering, convert::TryFrom, fmt};

use crate::interpreter::InterpreterError;

#[derive(Debug, Clone)]
pub enum LoxType {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl TryFrom<LoxType> for f64 {
    type Error = InterpreterError;

    fn try_from(value: LoxType) -> Result<Self, Self::Error> {
        use LoxType::*;

        if let Number(n) = value {
            Ok(n)
        } else {
            Err(InterpreterError)
        }
    }
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

impl PartialOrd for LoxType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use LoxType::*;

        match (self, other) {
            (Boolean(n), Boolean(m)) => n.partial_cmp(m),
            (Nil, Nil) => Some(Ordering::Equal),
            (Number(n), Number(m)) => n.partial_cmp(m),
            (String(n), String(m)) => n.partial_cmp(m),
            _ => None,
        }
    }
}

impl fmt::Display for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoxType::*;

        match self {
            Boolean(ref b) => write!(f, "{}", b),
            Number(ref n) => write!(f, "{}", n),
            String(ref s) => write!(f, "{}", s),
            Nil => write!(f, "nil"),
        }
    }
}
