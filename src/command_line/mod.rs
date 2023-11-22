use clap::{Parser, Subcommand};

use crate::result::Result;

mod serve;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve(serve::Arguments),
}

impl Cli {
    pub fn try_parse() -> Result<Self> {
        <Self as Parser>::try_parse().map_err(Into::into)
    }

    pub fn execute(self) -> Result<()> {
        match self.command {
            Commands::Serve(args) => args.execute(),
        }
    }
}
