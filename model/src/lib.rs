use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod id;

pub use id::ProductID;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
pub enum Retailer {
    Coles,
    Woolworths,
}

impl Retailer {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Coles => "coles",
            Self::Woolworths => "woolworths",
        }
    }
}
