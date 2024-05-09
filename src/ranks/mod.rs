use std::{
    collections::BTreeMap,
    fs::{self, read_dir, write},
};

use anyhow::Result;
use itertools::Itertools;

use crate::{prices::load, Vendor};

pub fn main() -> Result<()> {
    let mut prices = BTreeMap::new();
    for x in Vendor::all() {
        prices.insert(x, Vec::new());
    }

    let entries = read_dir("data/prices")?;
    for x in entries {
        let path = x?.path();
        let name = path.file_name().unwrap().to_string_lossy();

        if let Some(x) = name.strip_suffix(".bin.zst") {
            if let Some((name, vendor)) = x.rsplit_once('-') {
                prices
                    .get_mut(&vendor.parse()?)
                    .unwrap()
                    .push(name.to_string());
            }
        }
    }

    for (vendor, mut prices) in prices {
        prices.sort();
        prices.reverse();
        let mut prices = prices.into_iter().take(14).collect_vec();
        prices.reverse();
        println!("Ranking {vendor} from {prices:?}");

        let mut products = BTreeMap::new();
        let mut stores = BTreeMap::new();
        for file in prices {
            let prices = load(&file, vendor)?;
            for (product, groups) in prices {
                for group in groups {
                    let points = 1 + (group.info.discounts.len() * 2);

                    if let Some(x) = products.get_mut(&product) {
                        *x += group.stores.len() * points;
                    } else {
                        products.insert(product, group.stores.len());
                    }

                    for store in group.stores {
                        if let Some(x) = stores.get_mut(&store) {
                            *x += points;
                        } else {
                            stores.insert(store, 1);
                        }
                    }
                }
            }
        }

        let slug = vendor.slug();
        write(
            format!("data/ranks/products-{slug}.json"),
            serde_json::to_string(&products)?,
        )?;
        write(
            format!("data/ranks/stores-{slug}.json"),
            serde_json::to_string(&stores)?,
        )?;
    }

    Ok(())
}

impl Vendor {
    pub fn load_product_ranks(&self) -> Result<BTreeMap<u64, usize>> {
        let data = fs::read_to_string(format!("data/ranks/products-{}.json", self.slug()))?;
        Ok(serde_json::from_str(&data)?)
    }
}
