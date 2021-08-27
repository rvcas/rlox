use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, InterpreterError},
    lox_type::LoxType,
    token::Token,
};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: fn(&[LoxType]) -> Result<LoxType, InterpreterError>,
    },
    User {
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    },
}

impl Function {
    pub fn arity(&self) -> usize {
        use Function::*;

        match self {
            Native { arity, .. } => *arity,
            User { params, .. } => params.len(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LoxType],
    ) -> Result<LoxType, InterpreterError> {
        use Function::*;

        match self {
            Native { body, .. } => body(arguments),
            User {
                body,
                params,
                closure,
                is_initializer,
                ..
            } => {
                let env = Rc::new(RefCell::new(Environment::with_enclosing(closure)));

                for (param, arg) in params.iter().zip(arguments) {
                    env.borrow_mut().define(&param.lexeme, arg.clone());
                }

                match interpreter.execute_block(body, env) {
                    Ok(()) => {
                        if *is_initializer {
                            if let Some(value) = closure.borrow().get_at(0, "this") {
                                Ok(value)
                            } else {
                                Err(InterpreterError::runtime_error(
                                    None,
                                    "expect initializer to return this",
                                ))
                            }
                        } else {
                            Ok(LoxType::Nil)
                        }
                    }
                    Err(InterpreterError::Return(value)) => {
                        if *is_initializer {
                            if let Some(value) = closure.borrow().get_at(0, "this") {
                                Ok(value)
                            } else {
                                Err(InterpreterError::runtime_error(
                                    None,
                                    "expect initializer to return this",
                                ))
                            }
                        } else {
                            Ok(value)
                        }
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }

    pub fn bind(&self, instance: LoxType) -> Self {
        match self {
            Self::User {
                name,
                params,
                body,
                closure,
                is_initializer,
            } => {
                let env = Rc::new(RefCell::new(Environment::with_enclosing(closure)));

                env.borrow_mut().define("this", instance);

                Self::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: env,
                    is_initializer: *is_initializer,
                }
            }
            Self::Native { .. } => unreachable!(),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Function::*;

        match self {
            Native { .. } => write!(f, "<native func>"),
            User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        use Function::*;

        match self {
            Native { .. } => write!(f, "<native func>"),
            User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
