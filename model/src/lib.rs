mod id;

pub use id::ProductID;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
