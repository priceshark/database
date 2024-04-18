use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use typed_floats::tf32::NonNaN;

use crate::Retailer;

mod coles;
mod woolworths;

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
    let output_path = format!("raw/{name}-{date}.bin.zst");
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

    let mut product_prices: BTreeMap<u32, Vec<RawPriceGroup>> = BTreeMap::new();
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
            pb.inc(1);
        }
    }

    let mut writer = zstd::Encoder::new(File::create(output_path)?, 0)?;
    let data = postcard::to_allocvec(&product_prices)?;
    writer.write_all(data.as_slice())?;
    writer.finish()?;

    Ok(())
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
                discount_price: NonNaN::new(0.0).unwrap(),
                discount_quantity: 1,
                promotion: Promotion::None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawPriceInfo {
    pub price: NonNaN,
    pub discount_price: NonNaN,
    pub discount_quantity: u32,
    pub promotion: Promotion,
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
