use crate::{
    ast::{Expr, Stmt},
    lox,
    lox_type::LoxType,
    token::Token,
    token_type::TokenType,
};

#[derive(Debug)]
pub struct ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = if self.matches(vec![TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(LoxType::Nil)
        };

        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.matches(vec![TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;

        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;

        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        self.consume(TokenType::SemiColon, "Expect ';' after expression.")?;

        Ok(Stmt::Expression(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;

        if self.matches(vec![TokenType::Equal]) {
            let equals = self.previous();

            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(self.error(equals, "Invalid assignment target.")),
            }
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.matches(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();

            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.matches(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();

            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.matches(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();

            let right = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();

            let right = self.unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();

            let right = self.unary()?;

            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(vec![TokenType::False]) {
            Ok(Expr::Literal(LoxType::Boolean(false)))
        } else if self.matches(vec![TokenType::True]) {
            Ok(Expr::Literal(LoxType::Boolean(true)))
        } else if self.matches(vec![TokenType::Nil]) {
            Ok(Expr::Literal(LoxType::Nil))
        } else if self.matches(vec![TokenType::Number, TokenType::String])
            && self.previous().literal.is_some()
        {
            Ok(Expr::Literal(self.previous().literal.unwrap()))
        } else if self.matches(vec![TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous()))
        } else if self.matches(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self.error(self.peek(), "Expect expression."))
        }
    }

    fn matches(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in &types {
            if self.check(token_type.to_owned()) {
                self.advance();

                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(self.peek(), message))
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn error(&self, token: Token, message: &str) -> ParseError {
        lox::parse_error(token, message);

        ParseError {}
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
