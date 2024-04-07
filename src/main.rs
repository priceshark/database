use std::{
    collections::{BTreeMap, HashSet},
    fs::{create_dir_all, read_to_string, write},
    process,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use fantoccini::ClientBuilder;
use indexmap::IndexMap;
use inquire::{Autocomplete, Text};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ureq::get;

// mod images;
mod helper;
mod links;
mod raw;
mod size;
mod tokens;

use size::Size;

type Tokens = BTreeMap<String, String>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawProducts {
    tokens: Tokens,
    products: IndexMap<String, IndexMap<String, RawProduct>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct RawProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
    #[serde(default)]
    links: Vec<String>,
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
    links: Vec<String>,
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
    LinkHelper { filter: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    create_dir_all("cache")?;

    let raw: RawProducts = serde_json::from_str(&read_to_string("products.json")?)?;
    let mut products: Vec<Product> = Vec::new();
    let tokens = raw.tokens;

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
            // let mut output = serde_json::to_string_pretty(&products)?;
            // output.push('\n');
            // write("products.json", &output)?;
        }
        Command::MissingImages => {
            todo!()
            // images::missing_images(&mut products).await?;
        }
        Command::MissingLinks => {
            for x in links::missing_links(&products).await? {
                println!("{x}");
            }
        }
        Command::LinkHelper { filter } => {
            let links: Vec<String> = links::missing_links(&products)
                .await?
                .into_iter()
                .filter(|x| x.contains(&filter))
                .collect();
            eprintln!("Found {} links", links.len());

            let browser = ClientBuilder::native()
                .capabilities(serde_json::from_value(json!({
                    "pageLoadStrategy": "none",
                    "goog:chromeOptions": {
                        "debuggerAddress": "localhost:9222",
                    }
                }))?)
                .connect("http://localhost:9515")
                .await
                .context("Failed to connect to browser")?;

            for link in links {
                browser.goto(&link).await?;
                let (name_raw, size_raw) = helper::link_helper(&products)?;
                let (name, tags) = tokens::eval(&tokens, &name_raw)?;
                let size = size_raw.parse()?;

                if let Some(x) = products
                    .iter_mut()
                    .find(|x| x.name_raw == name_raw && x.size_raw == size_raw)
                {
                    x.links.push(link);
                } else {
                    products.push(Product {
                        name,
                        name_raw,
                        tags,
                        size,
                        size_raw,
                        image: None,
                        links: vec![link],
                    })
                }
                raw::write_products(tokens.clone(), products.to_vec())?;
            }
        }
    }

    Ok(())
}
