use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;
use typed_floats::tf32::NonNaN;

use super::{Discount, Promotion, RawPriceRecord};

pub fn extract(line: &str) -> Result<RawPriceRecord> {
    let item: Item = serde_json::from_str(&line)?;
    let mut record = RawPriceRecord::new(item.store, item.product_id.parse()?);

    if item.store == 0 {
        // specials seem to be combining from multiple stores?
        // or upstream db issue
        return Ok(record);
    }

    let promotion = item
        .promotion_info
        .map(|x| x.r#type)
        .unwrap_or(RawPromotionType::None);

    let mut price = item.price.unwrap_or(0);
    if let Some(raw) = item.was_price {
        if promotion == RawPromotionType::Special {
            record.info.discounts.push(Discount {
                price: NonNaN::new(price as f32 / 100.0)?,
                quantity: 1,
                members_only: false,
            });
            price = parse_price(raw.strip_prefix("Was ").context("Invalid price")?)?;
        }
    }
    record.info.price = NonNaN::new(price as f32 / 100.0)?;

    if let Some(x) = &item.multi_buy_price_info {
        let (quantity, price) = parse_quantity_price(&x.price)?;
        record.info.discounts.push(Discount {
            price: NonNaN::new(price as f32 / 100.0)?,
            quantity,
            members_only: true,
        })
    }
    if let Some(x) = &item.member_price_info {
        let (quantity, price) = parse_quantity_price(&x.title)?;
        record.info.discounts.push(Discount {
            price: NonNaN::new(price as f32 / quantity as f32 / 100.0)?,
            quantity,
            members_only: true,
        })
    }

    let promotion = match promotion {
        RawPromotionType::None => Promotion::None,
        RawPromotionType::Special => Promotion::Special,
        RawPromotionType::LowPrice => Promotion::WoolworthsLowPrice,
        RawPromotionType::PriceDropped => Promotion::WoolworthsPriceDropped,
    };
    record.info.promotion = promotion;

    Ok(record)

    // Ok(Some(PriceRecord {
    //     timestamp: item.timestamp,
    //     product,
    //     store,
    //     price: (price as f64) / 100.0,
    //     discount_price: (discount_price as f64) / 100.0,
    //     discount_member_only: item.member_price_info.is_some(),
    //     discount_online_only: false, // TODO
    //     discount_quantity,
    //     discount_collection: 0, // TODO
    //     promotion,
    // }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    product_id: String,
    store: u32,
    // timestamp: u32,
    is_available: bool,
    in_store_availability_info: Option<InStoreAvailability>,
    member_price_info: Option<MemberPrice>,
    multi_buy_price_info: Option<MultiBuyPrice>,
    promotion_info: Option<PromotionInfo>,

    price: Option<u32>,
    was_price: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InStoreAvailability {
    status: AvailabilityStatus,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AvailabilityStatus {
    InStock,
    Unavailable,
    SeeInStore,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemberPrice {
    // header: String, // "Member Price"
    // subtitle: String, // unit price
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MultiBuyPrice {
    price: String,
    // unit_price: Option<String>, // "$2.00 per 100g"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromotionInfo {
    // label: String, // "LOW PRICE", "SAVE $1.75", "PRICES DROPPED"
    r#type: RawPromotionType,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RawPromotionType {
    None,
    Special,
    LowPrice,
    PriceDropped,
}

pub fn parse_price(raw: &str) -> Result<u32> {
    let raw = raw.strip_prefix("$").context("Invalid price")?;
    if let Some((a, b)) = raw.split_once(".") {
        let a: u32 = a.parse()?;
        let b: u32 = b.parse()?;
        return Ok(a * 100 + b);
    } else {
        let a: u32 = raw.parse()?;
        return Ok(a * 100);
    }
}

pub fn parse_quantity_price(raw: &str) -> Result<(u32, u32)> {
    let raw: Vec<_> = raw.split(' ').collect();
    if raw.len() == 3 {
        ensure!(raw[1] == "for", "Invalid price");
        Ok((raw[0].parse()?, parse_price(raw[2])?))
    } else if raw.len() == 1 {
        Ok((1, parse_price(raw[0])?))
    } else {
        bail!("Invalid price")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price() {
        assert_eq!(parse_price("$1.23").unwrap(), 123);
        assert_eq!(parse_price("$4.50").unwrap(), 450);
        assert_eq!(parse_price("$5.06").unwrap(), 506);
        assert_eq!(parse_price("$7").unwrap(), 700);
        assert_eq!(parse_price("$89.00").unwrap(), 8900);
    }

    #[test]
    fn test_quantity_price() {
        assert_eq!(parse_quantity_price("1 for $2").unwrap(), (1, 200));
        assert_eq!(parse_quantity_price("3 for $4.50").unwrap(), (3, 450));
        assert_eq!(parse_quantity_price("67 for $8.99").unwrap(), (67, 899));
        assert_eq!(parse_quantity_price("$1.23").unwrap(), (1, 123));
        assert_eq!(parse_quantity_price("$4").unwrap(), (1, 400));
    }
}
