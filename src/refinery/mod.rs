use anyhow::Result;
use clap::ValueEnum;
use rayon::prelude::*;

use crate::{PriceRecord, Retailer};

mod coles_prices;
mod woolworths_prices;

#[derive(Clone, Debug, ValueEnum)]
pub enum Source {
    ColesPrices,
    WoolworthsPrices,
}

impl Source {
    pub fn retailer(&self) -> Retailer {
        match self {
            Self::ColesPrices => Retailer::Coles,
            Self::WoolworthsPrices => Retailer::Woolworths,
        }
    }

    pub fn extract(&self, x: &str) -> Result<PriceRecord> {
        match self {
            Self::ColesPrices => coles_prices::extract(x),
            Self::WoolworthsPrices => woolworths_prices::extract(x),
        }
    }

    pub fn refine(&self, x: Vec<String>) -> Result<Vec<PriceRecord>> {
        x.par_iter().map(|x| self.extract(x)).collect()
    }

    pub fn suggested_len(&self) -> u64 {
        match self {
            Self::ColesPrices => 17_600_000,
            Self::WoolworthsPrices => 51_400_000,
        }
    }
}

// impl Source {
//     pub async fn refine(&self, conn: &SqlitePool) -> Result<()> {
//         let pb = ProgressBar::new(self.suggested_len()).with_style(ProgressStyle::with_template(
//             "[{elapsed_precise}] {human_pos} {percent}% ({per_sec})",
//         )?);
//         let mut tx = conn.begin().await?;

//         let mut lines = stdin().lines();
//         loop {
//             let mut batch = Vec::new();
//             for _ in 0..BATCH_SIZE {
//                 if let Some(line) = lines.next() {
//                     batch.push(line?)
//                 } else {
//                     break;
//                 }
//             }
//             if batch.is_empty() {
//                 break;
//             }

//             let batch: Vec<_> = batch
//                 .par_iter()
//                 .map(|x| match self {
//                     Self::ColesPrices => coles_prices::extract(x),
//                     Self::WoolworthsPrices => woolworths_prices::extract(x),
//                 })
//                 .collect();

//             for result in batch {
//                 let record = result?;
//                 query!(
//                 "insert into raw_price (retailer, store, product, price, discount_price, discount_member_only, discount_online_only, discount_quantity, promotion)
//                  values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
//                 ", record.retailer, record.store, record.product, record.price, record.discount_price, record.discount_member_only, record.discount_online_only, record.discount_quantity, record.promotion
//             ).execute(&mut *tx).await?;
//                 pb.inc(1);
//             }
//         }
//         tx.commit().await?;

//         Ok(())
//     }

// }
