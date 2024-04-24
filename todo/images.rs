use std::{
    collections::HashMap,
    fs::{write, File},
    io::Write,
    path::Path,
    time::Duration,
};

use anyhow::{Context, Result};
use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    key::Key,
    wd::Capabilities,
    Client, ClientBuilder, Locator,
};
use httpdate::parse_http_date;
use reqwest::header::LAST_MODIFIED;
use serde_json::json;
use tokio::time::sleep;
use url::form_urlencoded::byte_serialize;

use crate::RawProduct;

// after closing a window/tab the webdriver seems to still try and have that tab
// selected, resulting in an error when trying to create a new window? this just
// selects the first window to prevent that from happening
async fn closed_windows_fix(c: &Client) -> Result<()> {
    c.switch_to_window(
        c.windows()
            .await?
            .into_iter()
            .next()
            .context("No windows open")?,
    )
    .await?;

    Ok(())
}

pub async fn missing_images(products: &mut Vec<RawProduct>) -> Result<()> {
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
    let http = reqwest::Client::builder()
        .user_agent("priceshark/database")
        .build()?;

    for i in 0..products.len() {
        let product = &products[i];
        if product.image.is_some() || product.links.len() == 0 {
            continue;
        }

        closed_windows_fix(&browser).await?;
        let submit = browser.new_window(true).await?.handle;
        browser.switch_to_window(submit.clone()).await?;
        browser
            .goto(&format!(
                "data:text/html,<title>[ &%2310003; ]</title><p>Close this tab to submit.</p>"
            ))
            .await?;

        // close any old tabs
        for w in browser.windows().await? {
            if w != submit {
                browser.switch_to_window(w.clone()).await?;
                browser.close_window().await?;
            }
        }
        closed_windows_fix(&browser).await?;

        let mut query = format!("{} {}", product.name, product.size);
        for tag in &product.tags {
            if let Some(x) = tag.strip_prefix("container/") {
                query.push(' ');
                query.push_str(x);
            }
        }

        let search = browser.new_window(true).await?.handle;
        browser.switch_to_window(search.clone()).await?;
        browser
            .goto(&format!(
                "https://duckduckgo.com/?q={}",
                byte_serialize(query.as_bytes()).collect::<String>()
            ))
            .await?;
        for x in &product.links {
            let handle = browser.new_window(true).await?.handle;
            browser.switch_to_window(handle).await?;
            browser.goto(x).await?;
        }
        browser.switch_to_window(search).await?;

        loop {
            let ws = browser.windows().await?;
            if !ws.contains(&submit) {
                if ws.len() == 1 {
                    let w = ws.into_iter().next().unwrap();
                    closed_windows_fix(&browser).await?;
                    browser.switch_to_window(w).await?;
                    let url = browser.current_url().await?.to_string();

                    let response = http.get(&url).send().await?.error_for_status()?;
                    let date = match response.headers().get(LAST_MODIFIED) {
                        Some(x) => Some(parse_http_date(x.to_str()?)?),
                        _ => None,
                    };

                    let data = response.bytes().await?;
                    let hash = sha256::digest(data.as_ref());

                    let mut file = File::create(Path::new("cache").join(&hash))?;
                    file.write_all(&data)?;
                    if let Some(date) = date {
                        file.set_modified(date)?;
                    }

                    let image = crate::Image { url, hash };
                    println!("Setting image: {image:?}");
                    products[i].image = Some(image);
                } else {
                    eprintln!("Unexpected window count - skipping");
                }

                break;
            }
            sleep(Duration::from_secs(1)).await;
        }

        let mut output = serde_json::to_string_pretty(&products)?;
        output.push('\n');
        write("products.json", &output)?;
    }

    Ok(())
}
