use std::{clone::Clone, collections::HashMap, iter::Peekable, str::Chars};

use crate::{
    lox,
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Scanner<'a> {
    source: String,
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    keywords: HashMap<&'a str, TokenType>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();

        keywords.insert("and", TokenType::And);
        keywords.insert("class", TokenType::Class);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fun", TokenType::Fun);
        keywords.insert("if", TokenType::If);
        keywords.insert("nil", TokenType::Nil);
        keywords.insert("or", TokenType::Or);
        keywords.insert("print", TokenType::Print);
        keywords.insert("return", TokenType::Return);
        keywords.insert("super", TokenType::Super);
        keywords.insert("this", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("var", TokenType::Var);
        keywords.insert("while", TokenType::While);

        Self {
            source: source.to_string(),
            chars: source.chars().peekable(),
            tokens: Vec::new(),
            keywords,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;

            self.scan_token();
        }

        let end_token = Token::new(TokenType::Eof, String::new(), Literal::None, self.line);

        self.tokens.push(end_token);

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token_type);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => { /*  do nothing */ }
            '\n' => self.increment_line(),
            '"' => self.string(),
            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if is_alpha(c) {
                    self.indentifier();
                } else {
                    lox::error(self.line, &format!("Unexpected character -> {} <-", c));
                }
            }
        }
    }

    fn indentifier(&mut self) {
        while is_alpha_numberic(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        let token_type = self
            .keywords
            .get(text)
            .map_or(TokenType::Identifier, Clone::clone);

        self.add_token(token_type);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current].parse().unwrap();

        self.add_token_with_literal(TokenType::Number, Literal::Number(value));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.increment_line();
            }

            self.advance();
        }

        if self.is_at_end() {
            lox::error(self.line, "Unterminated string.");

            return;
        }

        self.advance();

        let value = self.source[(self.start + 1)..(self.current - 1)].to_string();

        self.add_token_with_literal(TokenType::String, Literal::String(value));
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() != expected {
            false
        } else {
            self.advance();

            true
        }
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            *self.chars.peek().unwrap()
        }
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.chars.nth(self.current + 1).unwrap()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;

        self.chars.next().unwrap()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, Literal::None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) {
        let lexeme = self.source[self.start..self.current].to_string();
        let token = Token::new(token_type, lexeme, literal, self.line);

        self.tokens.push(token);
    }

    fn increment_line(&mut self) {
        self.line += 1;
    }
}

fn is_alpha(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

fn is_alpha_numberic(c: char) -> bool {
    is_alpha(c) || c.is_digit(10)
}
