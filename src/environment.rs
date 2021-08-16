use std::collections::HashMap;

use crate::lox_type::LoxType;

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LoxType>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Box<Self>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn get(&self, name: &str) -> Option<LoxType> {
        let res = self.values.get(name);

        if res.is_some() {
            res.cloned()
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.get(name)
            } else {
                None
            }
        }
    }

    pub fn assign(&mut self, name: &str, value: LoxType) -> bool {
        if self.values.contains_key(name) {
            self.define(name, value);

            true
        } else {
            if let Some(enclosing) = &mut self.enclosing {
                enclosing.assign(name, value)
            } else {
                false
            }
        }
    }

    pub fn define(&mut self, name: &str, value: LoxType) {
        self.values.insert(name.to_string(), value);
    }
}
