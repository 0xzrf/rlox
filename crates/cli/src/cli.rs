use clap::Parser;
use parser::Parser as InterpreterParser;
use scanner::Scanner;

use crate::InterpreterErrors;
use crate::error::CliErrors;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Commands {
    cmd: String,
    file_path: String,
}

impl Commands {
    pub fn handle_command(&self) -> InterpreterErrors<i32> {
        match self.cmd.as_str() {
            "tokenize" => {
                let scanner = Scanner::new(self.file_path.clone());

                match scanner.scan() {
                    Ok((_scanner, error_code)) => Ok(error_code),
                    Err(e) => Err(CliErrors::ScannerError { reason: e }),
                }
            }

            "parse" => {
                let scanner = Scanner::new(self.file_path.clone());

                match scanner.scan() {
                    Ok((scanner, _error_code)) => {
                        let tokens = scanner.get_tokens();

                        if let Err(e) = InterpreterParser::new(&tokens).parse() {
                            Err(CliErrors::ParserError { reason: e.to_string() })
                        } else {
                            Ok(65)
                        }
                    }
                    Err(e) => Err(CliErrors::ScannerError { reason: e }),
                }
            }
            _ => Err(CliErrors::InvalidCommand),
        }
    }
}
