use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::Vendor;

use super::UpstreamProduct;

// use super::size::{self, Size};

pub fn load() -> Result<Vec<UpstreamProduct>> {
    let mut output = Vec::new();
    let mut reader = BufReader::new(File::open("internal/coles-products/raw.jsonl")?);
    for result in reader.lines() {
        let line = result?;
        let raw: RawProduct =
            serde_json::from_str(&line).with_context(|| format!("Failed to load: {line}"))?;

        // let pack: Option<u32> = raw
        //     .size
        //     .strip_suffix(" Pack")
        //     .or(raw.size.strip_suffix(" pack"))
        //     .and_then(|x| x.parse().ok());

        // if let Some(pack) = pack {
        //     if let Some((amount, unit)) = raw
        //         .nutrition
        //         .as_ref()
        //         .and_then(|x| size::find_amount(x.serving_size))
        //     {
        //         Size {
        //             pack,
        //             amount_is_total: false,
        //             amount,
        //             unit,
        //         }
        //     } else {
        //         Size {
        //             pack: 1,
        //             amount_is_total: false,
        //             amount: pack,
        //             unit:

        //         }

        //     }
        // }

        // let nutrition_pack: Option<f32> = raw
        //     .nutrition
        //     .as_ref()
        //     .and_then(|x| x.servings_per_package.parse().ok());

        // if let Some(pack) = pack {
        //     if let Some(x) = nutrition_pack {
        //         dbg!(raw);
        //     }
        // }

        // let size = match pack {
        //     Some(pack) => if let Some(x) = raw.nutrition {
        //         if
        //     }

        // }

        // if let Some((a, u)) = size::find_amount(&raw.size) {}

        // output.insert(raw.id, raw);

        output.push(UpstreamProduct {
            vendor: Vendor::Coles,
            id: raw.id,
            brand: raw.brand,
            name: raw.name,
            description: raw.long_description.unwrap_or_default(),
            size: None,
        })
    }

    Ok(output)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawProduct {
    brand: String,
    id: u32,
    long_description: Option<String>,
    nutrition: Option<RawNutrition>,
    name: String,
    size: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawNutrition {
    serving_size: String,
    servings_per_package: String,
}
