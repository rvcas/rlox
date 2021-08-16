use std::collections::HashMap;

use crate::lox_type::LoxType;

pub struct Environment {
    values: HashMap<String, LoxType>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<LoxType> {
        self.values.get(name).cloned()
    }

    pub fn define(&mut self, name: &str, value: LoxType) {
        self.values.insert(name.to_string(), value);
    }
}
