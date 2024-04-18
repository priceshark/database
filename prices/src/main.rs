use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

mod raw;

#[derive(Clone, Debug, ValueEnum)]
enum Retailer {
    Coles,
    Woolworths,
}

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
            raw::run(retailer, date)?;
        }
    }

    Ok(())
}
