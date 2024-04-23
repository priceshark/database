use core::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod address;
mod osm;
mod product;
mod store;

pub use address::{Address, State};
pub use osm::OsmId;
pub use product::ProductId;
pub use store::StoreId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Retailer {
    Coles,
    Woolworths,
}

impl fmt::Display for Retailer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coles => write!(f, "Coles"),
            Self::Woolworths => write!(f, "Woolworths"),
        }
    }
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
}
