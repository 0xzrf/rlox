use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Commands {
    #[arg(long = "tokenize")]
    target_tokenise_file: Option<String>,
}

impl Commands {
    pub fn handle_command() -> Result<()> {
        Ok(())
    }
}
