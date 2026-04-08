use std::fmt::Display;

use crate::TokenType;

pub struct Token {
    token_ty: TokenType,
    line: usize,
    lexeme: String,
}


impl Token {
    pub fn new(token_ty: TokenType, line: usize, lexeme: String) -> Self {
        Self { token_ty, line, lexeme }
    }

    pub fn to_string(&self) -> String {
        format!("{:#?} {} null", self.token_ty, self.lexeme)
    }
}
