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

    pub fn assign(&mut self, name: &str, value: LoxType) -> bool {
        if self.contains(name) {
            self.define(name, value);

            true
        } else {
            false
        }
    }

    pub fn define(&mut self, name: &str, value: LoxType) {
        self.values.insert(name.to_string(), value);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
}
