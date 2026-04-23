use clap::{Parser, Subcommand};
use parser::{Interpret, Parser as InterpreterParser};
use scanner::Scanner;

use crate::InterpreterErrors;
use crate::error::CliErrors;

#[derive(Parser)]
#[command(rename_all = "kebab-case")]
pub struct Commands {
    #[command(subcommand)]
    cmd: InterpreterCommands,
}

#[derive(Subcommand, Clone)]
pub enum InterpreterCommands {
    Tokenize { file_path: String },
    Parse { file_path: String },
}

impl Commands {
    pub fn handle_command(&self) -> InterpreterErrors<i32> {
        match &self.cmd {
            InterpreterCommands::Tokenize { file_path } => {
                if !file_path.ends_with(".lox") {
                    return Err(CliErrors::InvalidFileType);
                }
                let scanner = Scanner::new(file_path.clone());

                match scanner.scan(true) {
                    Ok((_scanner, error_code)) => Ok(error_code),
                    Err(e) => Err(CliErrors::ScannerError { reason: e }),
                }
            }

            InterpreterCommands::Parse { file_path } => {
                if !file_path.ends_with(".lox") {
                    return Err(CliErrors::InvalidFileType);
                }
                let scanner = Scanner::new(file_path.clone());

                match scanner.scan(false) {
                    Ok((scanner, _error_code)) => {
                        let tokens = scanner.get_tokens();

                        match InterpreterParser::new(&tokens).parse() {
                            Ok(stmts) => {
                                let mut interpreter = Interpret::new();
                                interpreter.interpret_stmts(&stmts).map_err(|e| {
                                    CliErrors::RuntimeError {
                                        reason: e.to_string(),
                                    }
                                })?;

                                Ok(0)
                            }
                            Err(e) => Err(CliErrors::ParserError {
                                reason: e.to_string(),
                            }),
                        }
                    }
                    Err(e) => Err(CliErrors::ScannerError { reason: e }),
                }
            }
        }
    }
}
