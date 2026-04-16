use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::Token;
use interpreter_types::TokenType::{self, *};

use crate::ast::{Expr, Literal};
use crate::parser_errors::{ParserError, ParserResult};

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

    /// The top level interface to create the syntax tree
    pub fn parse(&mut self) -> ParserResult<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparision()?;

        while self.match_any(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.comparision()?;
            expr = Expr::new_binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParserResult<Expr> {
        let mut expr = self.term()?;

        while self.match_any(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.term()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParserResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_any(&[MINUS, PLUS]) {
            let operator = self.prev().clone();
            let right = self.factor()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_any(&[SLASH, STAR]) {
            let operator = self.prev().clone();
            let right = self.unary()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.match_any(&[BANG, MINUS]) {
            let operator = self.prev().clone();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, right));
        }

        self.primary()
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        if self.match_any(&[FALSE]) {
            return Ok(Expr::new_primary(Literal::False));
        }
        if self.match_any(&[TRUE]) {
            return Ok(Expr::new_primary(Literal::True));
        }

        if self.match_any(&[NUMBER]) {
            return Ok(Expr::new_primary(Literal::Number(self.prev().literal.clone())));
        }

        if self.match_any(&[STRING]) {
            return Ok(Expr::new_primary(Literal::String(self.prev().literal.clone())));
        }

        if self.match_any(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RIGHT_PAREN, "Expect ')' after expression")?;
            return Ok(Expr::new_grouping(expr));
        }

        Err(ParserError::get_error(self.peek().unwrap().1, "Expected an expression"))
    }

    fn synchronize(&mut self) {
        self.advance();


        while !self.is_at_end() {
            if self.prev().token_ty == SEMICOLON {
                return;
            }

            match self.peek() {
                Some((_ix, token_ty)) if TokenType::is_token_starting_stmt(&token_ty.token_ty) => {
                    return;
                }
                None => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_ty: &TokenType, error: &str) -> Result<Token, ParserError> {
        if self.check(token_ty) {
            return Ok(self.advance().unwrap().1.clone());
        }

        let peek_token = self.peek().unwrap().1.clone();
        let parse_err = self.error(&peek_token, error);

        eprintln!("{parse_err}");
        Err(parse_err)
    }

    fn error(&mut self, token: &Token, error: &str) -> ParserError {
        ParserError::get_error(token, error)
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
        match self.peek() {
            Some((_ix, token)) => token.token_ty == *is_token,
            None => false,
        }
    }

    fn advance(&mut self) -> Option<(usize, &Token)> {
        self.tokens_peekable.next()
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek(), Some((_ix, token)) if token.token_ty == EOF)
    }

    fn peek(&mut self) -> Option<&(usize, &Token)> {
        self.tokens_peekable.peek()
    }


    fn prev(&mut self) -> &Token {
        let next = self
            .tokens_peekable
            .peek()
            .map(|(ix, _)| *ix)
            .unwrap_or(self.original_tokens.len());

        let prev_ix =
            next.checked_sub(1).expect("Parser::prev() called before consuming any token");

        &self.original_tokens[prev_ix]
    }
}
