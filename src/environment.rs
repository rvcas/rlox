use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lox_type::LoxType;

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LoxType>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
        }
    }

    pub fn get(&self, name: &str) -> Option<LoxType> {
        let res = self.values.get(name);

        if res.is_some() {
            res.cloned()
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow().get(name)
            } else {
                None
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Option<LoxType> {
        if distance > 0 {
            Some(
                self.ancestor(distance)
                    .borrow()
                    .values
                    .get(name)
                    .expect(&format!("Undefined variable '{}'", name))
                    .clone(),
            )
        } else {
            Some(
                self.values
                    .get(name)
                    .expect(&format!("Undefined variable '{}'", name))
                    .clone(),
            )
        }
    }

    pub fn assign(&mut self, name: &str, value: LoxType) -> bool {
        if self.values.contains_key(name) {
            self.define(name, value);

            true
        } else {
            if let Some(ref enclosing) = self.enclosing {
                enclosing.borrow_mut().assign(name, value)
            } else {
                false
            }
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &str, value: LoxType) -> bool {
        if distance > 0 {
            self.ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.to_string(), value);
        } else {
            self.values.insert(name.to_string(), value);
        }

        true
    }

    pub fn define(&mut self, name: &str, value: LoxType) {
        self.values.insert(name.to_string(), value);
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        // Get first ancestor
        let parent = self
            .enclosing
            .clone()
            .expect(&format!("No enclosing environment at {}", 1));
        let mut environment = Rc::clone(&parent);

        // Get next ancestors
        for i in 1..distance {
            let parent = environment
                .borrow()
                .enclosing
                .clone()
                .expect(&format!("No enclosing environment at {}", i));
            environment = Rc::clone(&parent);
        }

        environment
    }
}
