use anyhow::Result;
use clap::{Parser, Subcommand};

mod osm;
mod prices;
mod stores;
mod utils;
mod vendors;

pub use osm::OsmId;
pub use utils::agent;
pub use vendors::Vendor;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    module: Module,
}

#[derive(Debug, Subcommand)]
enum Module {
    Prices,
    Stores,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.module {
        Module::Prices => prices::main(),
        Module::Stores => stores::main(),
    }
}
