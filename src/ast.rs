use crate::{lox_type::LoxType, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var { name: Token, initializer: Expr },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),

    Literal(LoxType),

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable(Token),
}
