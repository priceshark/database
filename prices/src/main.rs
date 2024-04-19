use anyhow::Result;
use clap::{Parser, Subcommand};

use _model::Retailer;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Raw { retailer: Retailer, date: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Raw { retailer, date } => {
            _prices::raw::run(retailer, date)?;
        }
    }

    Ok(())
}
