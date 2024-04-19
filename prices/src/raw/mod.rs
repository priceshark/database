use std::collections::{BTreeMap, BTreeSet};
use std::fs::{write, File};
use std::io::{BufRead, BufReader, Write};

use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_floats::tf32::NonNaN;

use crate::Retailer;

mod coles;
mod woolworths;

pub type RawPrices = BTreeMap<u32, Vec<RawPriceGroup>>;

pub fn run(retailer: Retailer, date: String) -> Result<()> {
    let folder = match retailer {
        Retailer::Coles => "coles-prices",
        Retailer::Woolworths => "woolworths-prices",
    };
    let input_path = format!("../internal/{folder}/output/{date}.jsonl.zst");
    let name = match retailer {
        Retailer::Coles => "coles",
        Retailer::Woolworths => "woolworths",
    };
    let output_path = format!("raw/{date}-{name}.bin.zst");
    let index_path = format!("raw/{date}-{name}.json");
    eprintln!("Reading {input_path} and writing to {output_path}");

    let input = BufReader::new(zstd::Decoder::new(File::open(input_path)?)?);

    let pb = ProgressBar::new(match retailer {
        Retailer::Coles => 17_600_000,
        Retailer::Woolworths => 51_400_000,
    })
    .with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {human_pos} {percent}% ({per_sec})")
            .expect("hardcoded"),
    );

    let mut product_prices: RawPrices = BTreeMap::new();
    let mut index = RawPriceIndex::new();
    for chunk in &input.lines().chunks(65535) {
        let chunk: Vec<_> = chunk.try_collect()?;
        let chunk: Vec<_> = chunk
            .par_iter()
            .map(|x| {
                match retailer {
                    Retailer::Coles => coles::extract(x),
                    Retailer::Woolworths => woolworths::extract(x),
                }
                .with_context(|| format!("Failed to extract: {x}"))
            })
            .collect();
        for record in chunk {
            let record = record?;
            index.stores.insert(record.store);
            index.products.insert(record.product);

            if record.info.price == 0.0 {
                // don't write this price to save storage

                if record.info.discounts.len() != 0 || record.info.promotion != Promotion::None {
                    bail!(
                        "price would have been ignored but it has info: {:?}",
                        &record
                    );
                }
            } else {
                let info = record.info;
                if let Some(prices) = product_prices.get_mut(&record.product) {
                    if let Some(price) = prices.iter_mut().find(|x| x.info == info) {
                        price.stores.push(record.store);
                    } else {
                        prices.push(RawPriceGroup {
                            stores: vec![record.store],
                            info,
                        });
                    }
                } else {
                    product_prices.insert(
                        record.product,
                        vec![RawPriceGroup {
                            stores: vec![record.store],
                            info,
                        }],
                    );
                }
            }

            pb.inc(1);
        }
    }

    let mut writer = zstd::Encoder::new(File::create(output_path)?, 0)?;
    let data = postcard::to_allocvec(&product_prices)?;
    writer.write_all(data.as_slice())?;
    writer.finish()?;

    write(index_path, serde_json::to_string(&index)?)?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPriceIndex {
    pub stores: BTreeSet<u32>,
    pub products: BTreeSet<u32>,
}

impl RawPriceIndex {
    fn new() -> Self {
        Self {
            stores: BTreeSet::new(),
            products: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPriceGroup {
    pub stores: Vec<u32>,
    pub info: RawPriceInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawPriceRecord {
    pub store: u32,
    pub product: u32,
    pub info: RawPriceInfo,
}

impl RawPriceRecord {
    fn new(store: u32, product: u32) -> Self {
        Self {
            store,
            product,
            info: RawPriceInfo {
                price: NonNaN::new(0.0).unwrap(),
                discounts: Vec::new(),
                promotion: Promotion::None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawPriceInfo {
    pub price: NonNaN,
    pub discounts: Vec<Discount>,
    pub promotion: Promotion,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Discount {
    // discounted, each
    pub price: NonNaN,
    pub quantity: u32,
    pub members_only: bool,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum Promotion {
    None = 0,
    New,
    Special,
    WhileStocksLast,
    ColesEveryday,
    ColesDownDown,
    ColesBonusCollectable,
    ColesDroppedAndLocked,
    ColesLocked,
    ColesFlybuysTriplePoints,
    ColesFlybuys100Points,
    WoolworthsLowPrice,
    WoolworthsPriceDropped,
}
