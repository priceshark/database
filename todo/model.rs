use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceRecord {
    pub timestamp: u32,
    pub retailer: Retailer,
    pub product: u32,
    pub store: u32,
    pub price: f64,
    pub discount_price: f64,
    pub discount_member_only: bool,
    pub discount_online_only: bool,
    pub discount_quantity: u32,
    pub discount_collection: u32,
    pub promotion: PromotionType,
}

#[derive(
    Debug, Clone, Copy, Serialize_repr, Deserialize_repr, sqlx::Type, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
pub enum Retailer {
    Coles,
    Woolworths,
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, sqlx::Type)]
#[repr(u8)]
pub enum PromotionType {
    None = 0,
    New,
    Special,
    WhileStocksLast,
    ColesEveryday,
    ColesDownDown,
    ColesBonusCollectable,
    ColesDroppedAndLocked,
    ColesLocked,
    ColesFlybuysTriplePoints,
    ColesFlybuys100Points,
    WoolworthsLowPrice,
    WoolworthsPriceDropped,
}
