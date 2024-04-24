use std::fs::write;

use anyhow::{bail, Result};
use indexmap::IndexMap;

use crate::{Product, RawProduct, RawProducts, Tokens};

pub fn write_products(tokens: Tokens, mut products: Vec<Product>) -> Result<()> {
    products.sort_by(|a, b| a.size.comparable().total_cmp(&b.size.comparable()));
    products.sort_by(|a, b| a.name.cmp(&b.name));

    let mut raw = RawProducts {
        tokens,
        products: IndexMap::new(),
    };
    for product in products {
        if let Some(x) = raw.products.get_mut(&product.name_raw) {
            let raw_product = RawProduct {
                image: product.image,
                links: product.links,
            };
            if let Some(_) = x.insert(product.size_raw, raw_product) {
                bail!("Duplicate product keys")
            }
        } else {
            let raw_product = RawProduct {
                image: product.image,
                links: product.links,
            };
            let mut x = IndexMap::new();
            x.insert(product.size_raw, raw_product);
            raw.products.insert(product.name_raw, x);
        }
    }

    let mut output = serde_json::to_string_pretty(&raw)?;
    output.push('\n');
    write("products.json", &output)?;

    Ok(())
}
