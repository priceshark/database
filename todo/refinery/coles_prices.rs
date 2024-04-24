use crate::model::{PriceRecord, PromotionType, Retailer};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_with::chrono::{DateTime, Utc};
use typed_floats::tf32::NonNaN;

pub fn extract(line: &str) -> Result<PriceRecord> {
    let line: Item = serde_json::from_str(line)?;

    let timestamp: u32 = line.last_updated.timestamp().try_into()?;
    let record = if let Some(pricing) = line.pricing {
        let (price, discount_price) = if pricing.was == 0.0 {
            (pricing.now, 0.0)
        } else {
            (pricing.was, pricing.now)
        };

        let (discount_quantity, discount_collection) = if let Some(x) = pricing.multi_buy_promotion
        {
            (
                x.min_quantity,
                match x.r#type {
                    MultiBuyType::MultibuySingleSku => 0,
                    MultiBuyType::MultibuyMultiSku => x.id.parse()?,
                },
            )
        } else {
            (1, 0)
        };

        let promotion = match pricing.promotion_type {
            None => PromotionType::None,
            Some(RawPromotionType::Special) => PromotionType::Special,
            Some(RawPromotionType::Everyday) => PromotionType::ColesEveryday,
            Some(RawPromotionType::Downdown) => PromotionType::ColesDownDown,
            Some(RawPromotionType::Bonuscollectable) => PromotionType::ColesBonusCollectable,
            Some(RawPromotionType::Flybuys) => match pricing
                .offer_description
                .context("Flybuys promotion without description")?
                .as_ref()
            {
                "Triple points" => PromotionType::ColesFlybuysTriplePoints,
                "100 Bonus Points" => PromotionType::ColesFlybuys100Points,
                x => bail!("Unknown flybuys description: {x}"),
            },
            Some(RawPromotionType::New) => PromotionType::New,
            Some(RawPromotionType::Droppedlocked) => PromotionType::ColesDroppedAndLocked,
            Some(RawPromotionType::Locked) => PromotionType::ColesLocked,
        };

        PriceRecord {
            timestamp,
            retailer: Retailer::Coles,
            product: line.product,
            store: line.store,
            price,
            discount_price,
            discount_member_only: false,
            discount_online_only: pricing.online_special,
            discount_quantity,
            discount_collection,
            promotion,
        }
    } else {
        PriceRecord {
            timestamp,
            retailer: Retailer::Coles,
            product: line.product,
            store: line.store,
            price: 0.0,
            discount_price: 0.0,
            discount_member_only: false,
            discount_online_only: false,
            discount_quantity: 1,
            discount_collection: 0,
            promotion: PromotionType::None,
        }
    };

    Ok(record)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(rename = "id")]
    product: u32,
    store: u32,
    last_updated: DateTime<Utc>,
    pricing: Option<Pricing>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pricing {
    comparable: String,
    unit: UnitPricing,
    now: f64,
    was: f64,
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SpecialType {
    MultiSave,
    PercentOff,
    WhileStocksLast,
}
