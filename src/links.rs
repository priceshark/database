use std::{collections::HashSet, fs::read_to_string};

use anyhow::{bail, Context, Result};
use futures_util::future::try_join_all;

use crate::Product;

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

pub async fn missing_links(products: &Vec<Product>) -> Result<Vec<String>> {
    let mut existing = HashSet::new();
    for x in products {
        for x in &x.links {
            if !existing.insert(&**x) {
                bail!("don't think you want this duplicate link: {x}")
            }
        }
    }
    let ignored = read_to_string("ignored.txt")?;
    for x in ignored.lines() {
        if !existing.insert(x) {
            bail!("product references ignored link: {x}")
        };
    }

    let mut missing = Vec::new();
    for link in all_links().await? {
        if let Some((link, slug)) = link_slug(&link)? {
            if !existing.contains(&*link) {
                missing.push((link, slug));
            }
        }
    }

    // sort by slug
    missing.sort_by(|a, b| a.1.cmp(&b.1));

    Ok(missing.into_iter().map(|(link, _slug)| link).collect())
}

async fn all_links() -> Result<Vec<String>> {
    Ok(try_join_all(
        ["coles.com.au", "woolworths.com.au"]
            .into_iter()
            .map(|x| async move {
                reqwest::get(format!("https://pub.joel.net.au/cache/sitemaps/{x}.txt"))
                    .await?
                    .error_for_status()?
                    .text()
                    .await
            }),
    )
    .await?
    .into_iter()
    .flat_map(|x| x.lines().map(|x| x.to_string()).collect::<Vec<_>>())
    .collect())
}
