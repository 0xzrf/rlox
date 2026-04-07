use thiserror::Error;

pub type InterpreterErrors<T> = Result<T, CliErrors>;

#[derive(Error, Debug)]
pub enum CliErrors {
    #[error("Invalid file type")]
    InvalidFileType,
}
