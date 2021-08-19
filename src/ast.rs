use crate::{lox_type::LoxType, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        opt_else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var {
        name: Token,
        initializer: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
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

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable(Token),
}
