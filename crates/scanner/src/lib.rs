//! Scanner crate (library).
//!
//! Codecrafters stages will grow this into the tokenizer/scanner for the interpreter.

pub mod scanner;
pub use scanner::Scanner;
mod constants;
use constants::*;
