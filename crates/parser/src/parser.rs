use crate::ast::{Expr, Literal};
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
        let mut expr = self.term();

        while self.match_any(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.term();
            expr = Expr::new_binary(expr, operator, right)
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_any(&[MINUS, PLUS]) {
            let operator = self.prev().clone();
            let right = self.factor();
            expr = Expr::new_binary(expr, operator, right)
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_any(&[SLASH, STAR]) {
            let operator = self.prev().clone();
            let right = self.unary();
            expr = Expr::new_binary(expr, operator, right)
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_any(&[BANG, MINUS]) {
            let operator = self.prev().clone();
            let right = self.unary();
            return Expr::new_unary(operator, right);
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.check(&FALSE) {
            return Expr::new_primary(Literal::False);
        }
        if self.check(&TRUE) {
            return Expr::new_primary(Literal::True);
        }

        if self.check(&NUMBER) {
            return Expr::new_primary(Literal::Number(self.prev().literal.clone()));
        }

        if self.check(&STRING) {
            return Expr::new_primary(Literal::String(self.prev().literal.clone()));
        }

        if self.check(&LEFT_PAREN) {
            let expr = self.expression();
            return Expr::new_grouping(expr);
        }

        unreachable!() // TODO: Need to implement syntax error
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
            && is_token == &token.token_ty
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
