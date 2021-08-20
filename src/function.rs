use std::fmt;

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, RuntimeError},
    lox_type::LoxType,
    token::Token,
};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: fn(&[LoxType]) -> Result<LoxType, RuntimeError>,
    },
    User {
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
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
    ) -> Result<LoxType, RuntimeError> {
        use Function::*;

        match self {
            Native { body, .. } => body(arguments),
            User { body, params, .. } => {
                let mut env = Environment::with_enclosing(Box::new(interpreter.globals.clone()));

                for (param, arg) in params.iter().zip(arguments) {
                    env.define(&param.lexeme, arg.clone());
                }

                interpreter.execute_block(body, env)?;

                Ok(LoxType::Nil)
            }
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
