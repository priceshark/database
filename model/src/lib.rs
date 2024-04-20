use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod osm;
mod product;
mod store;

pub use osm::OsmId;
pub use product::ProductId;
pub use store::StoreId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Retailer {
    Coles,
    Woolworths,
}

impl Retailer {
    pub fn all() -> Vec<Self> {
        vec![Retailer::Coles, Retailer::Woolworths]
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Coles => "coles",
            Self::Woolworths => "woolworths",
        }
    }

    pub fn wikidata(&self) -> &'static str {
        match self {
            Self::Coles => "Q1108172",
            Self::Woolworths => "Q3249145",
        }
    }
}
