use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::{Token, TokenType};

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

    fn expression(&mut self) {}

    fn match_any(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, is_token: &Token) -> bool {
        if let Some((_, token)) = self.tokens_peekable.peek()
            && is_token == *token
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
        if let Some((_, token)) = self.tokens_peekable.peek()
            && *token.get_type() == TokenType::EOF
        {
            return true;
        }
        false
    }

    fn prev(&mut self) -> &Token {
        let next = self.tokens_peekable.peek().unwrap().0;
        &self.original_tokens[next - 1]
    }
}
