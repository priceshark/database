use std::{collections::HashSet, fmt::Display, fs::read_to_string, str::FromStr};

use anyhow::{bail, Context, Result};
use futures_util::{future::try_join_all, io::BufReader};
use itertools::Itertools;
use serde::{de::Error, Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{Product, Retailer};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ProductID {
    Coles(u32),
    Woolworths(u32),
}

impl ProductID {
    pub fn tmp(self) -> u32 {
        match self {
            Self::Coles(x) => x,
            Self::Woolworths(x) => x,
        }
    }
}

#[derive(Debug, Clone, DeserializeFromStr, SerializeDisplay)]
pub enum Link {
    ColesProduct { slug: String, id: u32 },
    WoolworthsProduct { slug: String, id: u32 },
    Unknown(String),
}

impl Link {
    pub fn url(&self) -> String {
        match self {
            Self::ColesProduct { slug, id } => {
                format!("https://www.coles.com.au/product/{slug}-{id}")
            }
            Self::WoolworthsProduct { slug, id } => {
                format!("https://www.woolworths.com.au/shop/productdetails/{id}/{slug}")
            }
            Self::Unknown(x) => x.clone(),
        }
    }

    pub fn slug(&self) -> Option<&str> {
        match self {
            Self::ColesProduct { slug, .. } => Some(&slug),
            Self::WoolworthsProduct { slug, .. } => Some(&slug),
            _ => None,
        }
    }

    pub fn product_id(&self) -> Option<ProductID> {
        match self {
            Self::ColesProduct { id, .. } => Some(ProductID::Coles(*id)),
            Self::WoolworthsProduct { id, .. } => Some(ProductID::Woolworths(*id)),
            _ => None,
        }
    }
}

impl FromStr for Link {
    type Err = anyhow::Error;

    fn from_str(url: &str) -> Result<Self> {
        if let Some(page) = url.strip_prefix("https://www.coles.com.au/product/") {
            let (slug, id) = page.rsplit_once('-').context("invalid link")?;
            return Ok(Self::ColesProduct {
                slug: slug.to_string(),
                id: id.parse()?,
            });
        }

        if let Some(page) = url.strip_prefix("https://www.woolworths.com.au/shop/productdetails/") {
            if page != "/" {
                let (id, slug) = page.split_once('/').context("invalid link")?;
                return Ok(Self::WoolworthsProduct {
                    slug: slug.to_string(),
                    id: id.parse()?,
                });
            }
        }

        Ok(Self::Unknown(url.to_string()))
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.url())
    }
}

impl Retailer {
    pub fn domain(&self) -> &'static str {
        match self {
            Retailer::Coles => "coles.com.au",
            Retailer::Woolworths => "woolworths.com.au",
        }
    }

    pub async fn links(&self) -> Result<Vec<Link>> {
        let text = reqwest::get(format!(
            "https://pub.joel.net.au/cache/sitemaps/{}.txt",
            self.domain()
        ))
        .await?
        .error_for_status()?
        .text()
        .await?;

        text.lines()
            .map(|x| Link::from_str(x).with_context(|| format!("Failed to parse link: {x}")))
            .try_collect()
    }
}

pub async fn missing_product_links(products: &Vec<Product>) -> Result<Vec<Link>> {
    let mut existing = HashSet::new();
    for x in products {
        for x in &x.links {
            if let Some(id) = x.product_id() {
                if !existing.insert(id) {
                    bail!("duplicate product link: {x:?}")
                }
            }
        }
    }
    // let ignored = read_to_string("ignored.txt")?;
    // for x in ignored.lines() {
    //     if !existing.insert(x) {
    //         bail!("product references ignored link: {x}")
    //     };
    // }

    let mut missing = Vec::new();
    for link in all_links().await? {
        if let Some(id) = link.product_id() {
            if !existing.contains(&id) {
                missing.push(link);
            }
        }
    }
    missing.sort_by(|a, b| a.slug().cmp(&b.slug()));
    Ok(missing)
}

pub async fn all_links() -> Result<Vec<Link>> {
    Ok(try_join_all(
        [Retailer::Coles, Retailer::Woolworths]
            .into_iter()
            .map(|x| async move { x.links().await }),
    )
    .await?
    .concat())
}
