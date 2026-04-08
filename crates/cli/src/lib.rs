mod cli;
use clap::Parser;
use cli::Commands;
mod error;
use error::*;

pub fn run() -> InterpreterErrors<()> {
    Commands::parse().handle_command()
}
