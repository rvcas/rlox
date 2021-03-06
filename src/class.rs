use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{function::Function, interpreter::InterpreterError, lox_type::LoxType, token::Token};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Function>,
    superclass: Option<Rc<RefCell<LoxClass>>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        methods: HashMap<String, Function>,
        superclass: Option<Rc<RefCell<LoxClass>>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            methods,
            superclass,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        if self.methods.contains_key(name) {
            self.methods.get(name).cloned()
        } else {
            if let Some(ref sc) = self.superclass {
                sc.borrow().find_method(name)
            } else {
                None
            }
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
    class: Rc<RefCell<LoxClass>>,
    fields: HashMap<String, LoxType>,
}

impl LoxInstance {
    pub fn new(class: &Rc<RefCell<LoxClass>>) -> Self {
        Self {
            class: Rc::clone(class),
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token, instance: &LoxType) -> Result<LoxType, InterpreterError> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Ok(field.clone())
        } else if let Some(method) = self.class.borrow().find_method(&name.lexeme) {
            Ok(LoxType::Callable(method.bind(instance.clone())))
        } else {
            Err(InterpreterError::runtime_error(
                Some(name.clone()),
                &format!("Undefined property '{}'.", name.lexeme),
            ))
        }
    }

    pub fn set(&mut self, name: &Token, value: LoxType) {
        self.fields.insert(name.lexeme.to_string(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<instance {}>", self.class.borrow().name)
    }
}
