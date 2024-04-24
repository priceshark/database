use std::{
    collections::{BTreeMap, HashSet},
    fs::{create_dir_all, read_to_string, OpenOptions},
    io::{stdin, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use counter::Counter;
use fantoccini::ClientBuilder;
use indexmap::IndexMap;
use itertools::Itertools;
use links::Link;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{
    query,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    ConnectOptions, Pool, SqlitePool,
};

// mod images;
mod helper;
mod links;
mod model;
mod raw;
mod refinery;
mod size;
mod tokens;
mod utils;

use helper::Maybe;
pub use model::*;
use size::Size;

type Tokens = BTreeMap<String, String>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawProducts {
    tokens: Tokens,
    products: IndexMap<String, IndexMap<String, RawProduct>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(default)]
    links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Image {
    url: String,
    hash: String,
}

#[derive(Debug, Clone)]
struct Product {
    name: String,
    name_raw: String,
    tags: Vec<String>,
    size: Size,
    size_raw: String,
    image: Option<Image>,
    links: Vec<Link>,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Build,
    MissingImages,
    MissingLinks,
    PriorityLinks { source: refinery::Source },
    Insert { source: refinery::Source },
    LinkHelper { filter: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    create_dir_all("cache")?;

    let pool: SqlitePool = Pool::connect_with(
        SqliteConnectOptions::from_str("ps.db")?
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off),
    )
    .await?;

    let raw: RawProducts = serde_json::from_str(&read_to_string("products.json")?)?;
    let mut products: Vec<Product> = Vec::new();
    let mut tokens = raw.tokens;

    for (name_raw, sizes) in raw.products {
        let (name, tags) = tokens::eval(&tokens, &name_raw)?;
        for (size_raw, product_raw) in sizes {
            let size = size_raw.parse()?;
            products.push(Product {
                name: name.clone(),
                name_raw: name_raw.clone(),
                tags: tags.clone(),
                size,
                size_raw,
                image: product_raw.image,
                links: product_raw.links,
            })
        }
    }

    match cli.command {
        Command::Build => {
            for product in &products {
                println!("{} {}", product.name, product.size);
                for l in &product.links {
                    dbg!(l);
                }
            }
            // let mut output = serde_json::to_string_pretty(&products)?;
            // output.push('\n');
            // write("products.json", &output)?;
        }
        Command::MissingImages => {
            todo!()
            // images::missing_images(&mut products).await?;
        }
        Command::MissingLinks => {
            for x in links::missing_product_links(&products).await? {
                println!("{}", x.url());
            }
        }
        Command::PriorityLinks { source } => {
            let mut links = BTreeMap::new();
            for link in source.retailer().links().await? {
                if let Some(id) = link.product_id() {
                    links.insert(id.tmp(), link);
                }
            }

            let mut counts = BTreeMap::new();
            for id in links.keys() {
                counts.insert(id, 0usize);
            }

            let mut stores = HashSet::new();
            let pb = utils::progress_bar(source.suggested_len());
            for chunk in &stdin().lines().chunks(65535) {
                for record in source.refine(chunk.try_collect()?)? {
                    if record.price == 0.0 {
                        if let Some(count) = counts.get_mut(&record.product) {
                            *count += 1;
                        }
                    }
                    stores.insert(record.store);
                    pb.inc(1);
                }
            }
            pb.finish_and_clear();

            let mut counts: Vec<_> = counts.into_iter().collect();
            counts.sort_by(|(_, a), (_, b)| a.cmp(b));
            let stores = stores.len() as f64;
            for (id, count) in counts {
                if count == 0 {
                    // no price information recorded for this product
                    continue;
                }

                let score = (stores - (count as f64)) / stores * 100.0;
                if let Some(link) = links.get(id) {
                    println!("{link} {score:.1}%");
                }
            }
        }
        Command::Insert { source } => {
            let mut tx = pool.begin().await?;
            let pb = utils::progress_bar(source.suggested_len());
            for chunk in &stdin().lines().chunks(65535) {
                for record in source.refine(chunk.try_collect()?)? {
                    query!(
                        "
                            insert into raw_price (retailer, store, product, price, discount_price, discount_member_only, discount_online_only, discount_quantity, promotion)
                            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                        ",
                        record.retailer, record.store, record.product, record.price, record.discount_price, record.discount_member_only, record.discount_online_only, record.discount_quantity, record.promotion
                        )
                        .execute(&mut *tx).await?;
                    pb.inc(1);
                }
            }
            tx.commit().await?;
        }
        Command::LinkHelper { filter } => {
            let links: Vec<Link> = links::missing_product_links(&products)
                .await?
                .into_iter()
                .filter(|x| x.slug().unwrap().contains(&filter))
                .collect();
            eprintln!("Found {} links", links.len());

            for x in links {
                println!("{}", x.url());
            }

            // let browser = ClientBuilder::native()
            //     .capabilities(serde_json::from_value(json!({
            //         "pageLoadStrategy": "none",
            //         "goog:chromeOptions": {
            //             "debuggerAddress": "localhost:9222",
            //         }
            //     }))?)
            //     .connect("http://localhost:9515")
            //     .await
            //     .context("Failed to connect to browser")?;

            // for link in links {
            //     browser.goto(&link).await?;

            //     println!();
            //     match helper::link_helper(&mut tokens, &products)? {
            //         Maybe::Skip => (),
            //         Maybe::Ignore => {
            //             writeln!(
            //                 OpenOptions::new()
            //                     .write(true)
            //                     .append(true)
            //                     .open("ignored.txt")?,
            //                 "{link}"
            //             )?;
            //         }
            //         Maybe::Something((name_raw, size_raw)) => {
            //             let (name, tags) = tokens::eval(&tokens, &name_raw)?;
            //             let size = size_raw.parse()?;

            //             if let Some(x) = products
            //                 .iter_mut()
            //                 .find(|x| x.name_raw == name_raw && x.size_raw == size_raw)
            //             {
            //                 x.links.push(link);
            //             } else {
            //                 products.push(Product {
            //                     name,
            //                     name_raw,
            //                     tags,
            //                     size,
            //                     size_raw,
            //                     image: None,
            //                     links: vec![link],
            //                 })
            //             }
            //             raw::write_products(tokens.clone(), products.to_vec())?;
            //         }
            // }
            // }
        }
    }

    Ok(())
}
