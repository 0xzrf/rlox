use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::Token;

pub struct Parser<'a> {
    tokens: Peekable<Enumerate<Iter<'a, Token>>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let tokens = tokens.iter().enumerate().peekable();

        Self { tokens }
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
        if let Some((_, token)) = self.tokens.peek()
            && is_token == *token
        {
            return true;
        }
        false
    }

    fn advance(&mut self) {
        self.tokens.next();
    }
}
