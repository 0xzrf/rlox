use crate::ast::Expr;
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::{
    Token,
    TokenType::{self, *},
};

pub struct Parser<'a> {
    tokens_peekable: Peekable<Enumerate<Iter<'a, Token>>>,
    original_tokens: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let tokens_peekable = tokens.iter().enumerate().peekable();

        Self {
            tokens_peekable,
            original_tokens: tokens.to_vec(),
        }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparision();

        while self.match_any(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.comparision();
            expr = Expr::new_binary(expr, operator, right)
        }
        expr
    }

    fn comparision(&mut self) -> Expr {
        todo!()
    }

    fn match_any(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, is_token: &TokenType) -> bool {
        if let Some((_, token)) = self.tokens_peekable.peek()
            && is_token == token.get_type()
            && !self.is_at_end()
        {
            return true;
        }
        false
    }

    fn advance(&mut self) {
        self.tokens_peekable.next();
    }

    fn is_at_end(&mut self) -> bool {
        self.check(&EOF)
    }

    fn prev(&mut self) -> &Token {
        let next = self.tokens_peekable.peek().unwrap().0;
        &self.original_tokens[next - 1]
    }
}
