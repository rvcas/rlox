use crate::{lox_type::LoxType, token::Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),

    Class {
        name: Token,
        methods: Vec<Stmt>,
        opt_superclass: Option<Expr>,
    },

    Expression(Expr),

    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        opt_else_branch: Option<Box<Stmt>>,
    },

    Print(Expr),

    Return {
        keyword: Token,
        value: Expr,
    },

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

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    Get {
        object: Box<Expr>,
        name: Token,
    },

    Grouping(Box<Expr>),

    Literal(LoxType),

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },

    Super {
        keyword: Token,
        method: Token,
    },

    This(Token),

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable(Token),
}

impl Expr {
    pub fn is_nil(&self) -> bool {
        use Expr::*;

        match self {
            Literal(LoxType::Nil) => true,
            _ => false,
        }
    }
}
