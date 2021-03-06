use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    class::{LoxClass, LoxInstance},
    function::Function,
};

#[derive(Debug, Clone)]
pub enum LoxType {
    Boolean(bool),
    Callable(Function),
    Class(Rc<RefCell<LoxClass>>),
    Instance(Rc<RefCell<LoxInstance>>),
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
            Class(class) => write!(f, "{}", class.borrow_mut()),
            Callable(function) => write!(f, "{}", function),
            Instance(instance) => write!(f, "{}", instance.borrow_mut()),
            Nil => write!(f, "nil"),
            Number(ref n) => write!(f, "{}", n),
            String(ref s) => write!(f, "{}", s),
        }
    }
}
