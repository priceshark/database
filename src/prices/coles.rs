use anyhow::{bail, Context, Result};
use serde::Deserialize;
// use serde_with::chrono::{DateTime, Utc};
use typed_floats::tf32::NonNaN;

use super::{Discount, Promotion, RawPriceRecord};

pub fn extract(line: &str) -> Result<RawPriceRecord> {
    let item: Item = serde_json::from_str(line)?;
    let mut record = RawPriceRecord::new(item.store, item.product);

    if item.store == 0 {
        return Ok(record);
    }

    if let Some(pricing) = item.pricing {
        if pricing.was == 0.0 {
            record.info.price = pricing.now;
        } else {
            record.info.price = pricing.was;
            record.info.discounts.push(Discount {
                price: pricing.now,
                quantity: 1,
                members_only: false,
            });
        }

        if let Some(x) = pricing.multi_buy_promotion {
            record.info.discounts.push(Discount {
                price: x.reward,
                quantity: x.min_quantity,
                members_only: false,
            });
        }

        record.info.promotion = match pricing.promotion_type {
            None => Promotion::None,
            Some(RawPromotionType::Special) => Promotion::Special,
            Some(RawPromotionType::Everyday) => Promotion::ColesEveryday,
            Some(RawPromotionType::Downdown) => Promotion::ColesDownDown,
            Some(RawPromotionType::Bonuscollectable) => Promotion::ColesBonusCollectable,
            Some(RawPromotionType::Flybuys) => match pricing
                .offer_description
                .context("Flybuys promotion without description")?
                .as_ref()
            {
                "Triple points" => Promotion::ColesFlybuysTriplePoints,
                "100 Bonus Points" => Promotion::ColesFlybuys100Points,
                x => bail!("Unknown flybuys description: {x}"),
            },
            Some(RawPromotionType::New) => Promotion::New,
            Some(RawPromotionType::Droppedlocked) => Promotion::ColesDroppedAndLocked,
            Some(RawPromotionType::Locked) => Promotion::ColesLocked,
            // seems to be something internal?
            Some(RawPromotionType::Continuityredeemable) => Promotion::None,
        };
    }

    Ok(record)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(rename = "id")]
    product: u32,
    store: u32,
    // last_updated: DateTime<Utc>,
    pricing: Option<Pricing>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pricing {
    comparable: String,
    unit: UnitPricing,
    now: NonNaN,
    was: NonNaN,
    promotion_type: Option<RawPromotionType>,
    online_special: bool,
    special_type: Option<SpecialType>,
    save_amount: Option<NonNaN>,
    save_statement: Option<String>,
    multi_buy_promotion: Option<MultiBuyPricing>,
    offer_description: Option<String>,
    price_description: Option<String>,
    promotion_description: Option<String>,
    save_percent: Option<NonNaN>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MultiBuyPricing {
    id: String,
    min_quantity: u32,
    reward: NonNaN,
    r#type: MultiBuyType,
}

#[derive(Debug, Deserialize)]
enum MultiBuyType {
    MultibuySingleSku,
    MultibuyMultiSku,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct UnitPricing {
    is_weighted: Option<bool>,
    of_measure_quantity: Option<u32>,
    of_measure_type: Option<String>,
    of_measure_units: Option<String>,
    price: Option<NonNaN>,
    quantity: NonNaN,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RawPromotionType {
    Special,
    Everyday,
    Downdown,
    Bonuscollectable,
    Flybuys,
    New,
    Droppedlocked,
    Locked,
    Continuityredeemable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SpecialType {
    MultiSave,
    PercentOff,
    WhileStocksLast,
}
