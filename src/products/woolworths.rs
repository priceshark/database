use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::Vendor;

use super::UpstreamProduct;

pub fn load() -> Result<Vec<UpstreamProduct>> {
    let mut output = Vec::new();
    let reader = BufReader::new(zstd::Decoder::new(File::open(
        "internal/woolworths-products/raw.jsonl.zst",
    )?)?);
    for result in reader.lines() {
        let line = result?;
        let raw: RawProduct =
            serde_json::from_str(&line).with_context(|| format!("Failed to load: {line}"))?;

        let brand = match raw.brand {
            Some(x) => x,
            None => continue,
        };
        let description = match raw.rich_description {
            Some(x) => x,
            None => continue,
        };
        output.push(UpstreamProduct {
            vendor: Vendor::Woolworths,
            id: raw.stockcode,
            brand,
            name: raw.name,
            description,
            size: None,
        })
    }

    Ok(output)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawProduct {
    pub brand: Option<String>,
    pub name: String,
    pub package_size: String,
    pub rich_description: Option<String>,
    pub stockcode: u32,
}
