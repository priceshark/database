use std::{
    collections::HashSet,
    fs::{read_to_string, write},
    process,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use ureq::get;

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct RawProduct {
    name: String,
    size: String,
    #[serde(default = "id")]
    id: String,
    // images: Vec<Image>,
    #[serde(default)]
    links: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

fn id() -> String {
    nanoid!(
        7,
        &[
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
            'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
            'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
            'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ]
    )
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Check,
    NextLink { filter: String },
}

fn link_slug(url: &str) -> Result<Option<(String, String)>> {
    if let Some(page) = url.strip_prefix("https://www.coles.com.au/product/") {
        let (slug, _) = page.rsplit_once('-').context("Failed to split link")?;
        return Ok(Some((url.to_string(), slug.to_string())));
    }

    if let Some(page) = url.strip_prefix("https://www.woolworths.com.au/shop/productdetails/") {
        if page != "/" {
            let (_, slug) = page.split_once('/').context("Failed to split link")?;
            return Ok(Some((url.to_string(), slug.to_string())));
        }
    }

    return Ok(None);
}

fn main() -> Result<()> {
    let mut products: Vec<RawProduct> = serde_json::from_str(&read_to_string("products.json")?)?;
    let mut should_write = false;

    let cli = Cli::parse();
    match cli.command {
        Command::Check => {
            should_write = true;
        }
        Command::NextLink { filter } => {
            let mut existing = HashSet::new();
            for x in &products {
                for x in &x.links {
                    if !existing.insert(&**x) {
                        bail!("don't think you want this duplicate link: {x}")
                    }
                }
            }
            let ignored = read_to_string("ignored.txt")?;
            for x in ignored.lines() {
                existing.insert(x);
            }

            let mut missing = Vec::new();
            for domain in ["coles.com.au", "woolworths.com.au"] {
                for url in get(&format!(
                    "https://pub.joel.net.au/cache/sitemaps/{domain}.txt"
                ))
                .call()?
                .into_string()?
                .lines()
                {
                    if let Some((link, slug)) = link_slug(url)? {
                        if !existing.contains(&*link) && link.contains(&filter) {
                            missing.push((link, slug));
                        }
                    }
                }
            }

            // sort by slug
            missing.sort_by(|a, b| a.1.cmp(&b.1));
            missing.reverse();
            if let Some((link, _slug)) = missing.pop() {
                process::Command::new("wl-copy")
                    .arg(&link)
                    .spawn()?
                    .wait()?;
                process::Command::new("xdg-open")
                    .arg(&link)
                    .spawn()?
                    .wait()?;
                println!("next up: {link}");
            } else {
                println!("all done!");
            }
        }
    }

    if should_write {
        products.sort();
        let mut output = serde_json::to_string_pretty(&products)?;
        output.push('\n');
        write("products.json", &output)?;
    }

    Ok(())
}
