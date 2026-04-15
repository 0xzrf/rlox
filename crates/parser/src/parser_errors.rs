use interpreter_types::{Token, TokenType};
use thiserror::Error;

pub type ParserResult<T> = Result<T, ParserError>;

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
    #[error("Parser Error: {msg}")]
    ParseError { msg: String },
}

impl ParserError {
    pub fn get_error(token: &Token, error: &str) -> Self {
        let report = if token.token_ty == TokenType::EOF {
            Report::new(format!("{} at end {}", token.line, error))
        } else {
            Report::new(format!("{} at '{}' {}", token.line, token.lexeme, error))
        };

        ParserError::ParseError { msg: report.msg }
    }
}
