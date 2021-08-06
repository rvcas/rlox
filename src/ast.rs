use crate::{lox_type::LoxType, token::Token};

#[derive(Clone, Debug)]
pub enum Expr {
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
}
