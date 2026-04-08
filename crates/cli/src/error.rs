use thiserror::Error;

pub type InterpreterErrors<T> = Result<T, CliErrors>;

#[derive(Error, Debug)]
pub enum CliErrors {
    #[error("Invalid file type")]
    InvalidFileType,

    #[error("Invalid command")]
    InvalidCommand,

    #[error("Scanner Error: {}", reason)]
    ScannerError { reason: String },
}
