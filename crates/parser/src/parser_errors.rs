use interpreter_types::{Token, TokenType};
use thiserror::Error;

pub struct Report {
    msg: String,
}

impl Report {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("some")]
    ParseError,
}

impl ParserError {
    pub fn error(token: Token, error: &str) {
        if token.token_ty == TokenType::EOF {
            Report::new(format!("{} at end {}", token.line, error));
        } else {
            Report::new(format!("{} at '{}' {}", token.line, token.lexeme, error));
        }
    }
}
