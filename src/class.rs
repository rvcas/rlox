use std::{collections::HashMap, fmt};

use crate::{interpreter::InterpreterError, lox_type::LoxType, token::Token};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, LoxType>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxType, InterpreterError> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Ok(field.clone())
        } else {
            Err(InterpreterError::runtime_error(
                Some(name.clone()),
                &format!("Undefined property '{}'.", name.lexeme),
            ))
        }
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance {}>", self.class.name)
    }
}
