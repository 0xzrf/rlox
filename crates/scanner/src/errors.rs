use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Unexpected character: {c}")]
    UnexpectedCharacter { c: char },

    #[error("Unterminated string.")]
    UnterminatedString,
}
