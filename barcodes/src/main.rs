use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
};

use _model::{ProductID, Retailer};
use anyhow::Result;
use serde::Deserialize;

fn main() -> Result<()> {
    let mut barcodes: BTreeMap<String, Vec<ProductID>> = BTreeMap::new();

    for r in [Retailer::Coles, Retailer::Woolworths] {
        let path = format!("raw/{}.jsonl.zst", r.slug());
        eprintln!("Loading {path}...");

        let lines = BufReader::new(zstd::Decoder::new(File::open(path)?)?);
        for line in lines.lines() {
            let product: Product = serde_json::from_str(&line?)?;
            if let Some(id) = r.parse_product_id(&product.id) {
                match id {
                    ProductID::Woolworths(x) if x >= 1_000_000_000 => continue,
                    _ => (),
                }

                for ean in product.eans {
                    if let Some(x) = barcodes.get_mut(&ean) {
                        x.push(id.clone())
                    } else {
                        barcodes.insert(ean, vec![id.clone()]);
                    }
                }
            }
        }
    }

    let mut output = BTreeMap::new();
    for (barcode, mut products) in barcodes {
        // let mut retailers = Vec::new();
        // for p in &products {
        //     let r = p.retailer();
        //     if !retailers.contains(&r) {
        //         retailers.push(r);
        //     }
        // }

        products.sort();
        // if retailers.len() > 1 {
        output.insert(barcode, products);
        // }
    }

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Product {
    id: String,
    active: bool,
    brand: Brand,
    name: Option<String>,
    #[serde(rename = "EANs")]
    eans: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Brand {
    name: Option<String>,
}
