use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Vendor;

use self::{size::Size, tokens::Tokenizer};

mod coles;
mod size;
mod tokens;
mod woolworths;

pub fn main() -> Result<()> {
    let path = Path::new("data/products/raw.jsonl");
    let mut raw: Vec<UpstreamProduct> = Vec::new();
    if path.exists() {
        for result in BufReader::new(File::open(path)?).lines() {
            raw.push(serde_json::from_str(&result?)?);
        }
    } else {
        for vendor in Vendor::all() {
            let ranks = vendor.load_product_ranks()?;
            let products = match vendor {
                Vendor::Coles => coles::load()?,
                Vendor::Woolworths => woolworths::load()?,
            };
            let total = products.len();
            let mut skipped = 0usize;
            for product in products {
                if let Some(rank) = ranks.get(&(product.id as u64)) {
                    if *rank > 1000 {
                        raw.push(product);
                    } else {
                        skipped += 1;
                    }
                }
            }
            eprintln!(
                "Loaded {} {vendor} products (skipped {skipped})",
                total - skipped
            )
        }

        let mut file = File::create(path)?;
        for x in &raw {
            writeln!(file, "{}", serde_json::to_string(x)?)?;
        }
    };

    let tokenizer = Tokenizer::load(Path::new("data/products/tokens.yaml"))?;
    for product in raw {
        let suggestions = tokenizer.suggest(&product.name);
        if suggestions.len() > 1 {
            println!("{} {suggestions:?}", product.name)
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct UpstreamProduct {
    vendor: Vendor,
    id: u32,
    brand: String,
    name: String,
    description: String,
    size: Option<String>,
}
