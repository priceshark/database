use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{osm::OsmId, Retailer};

pub struct Store {
    pub id: StoreId,
    pub osm_id: OsmId,
    pub lat: f64,
    pub lon: f64,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoreId {
    Coles(u32),
    Woolworths(u32),
}

impl StoreId {
    pub fn retailer(&self) -> Retailer {
        match self {
            Self::Coles(_) => Retailer::Coles,
            Self::Woolworths(_) => Retailer::Woolworths,
        }
    }

    pub fn coles(&self) -> Option<u32> {
        match self {
            Self::Coles(x) => Some(*x),
            _ => None,
        }
    }

    pub fn woolworths(&self) -> Option<u32> {
        match self {
            Self::Woolworths(x) => Some(*x),
            _ => None,
        }
    }
}

impl Retailer {
    pub fn parse_store_id(&self, id: &str) -> Option<StoreId> {
        Some(match self {
            Self::Coles => StoreId::Coles(id.parse().ok()?),
            Self::Woolworths => StoreId::Woolworths(id.parse().ok()?),
        })
    }

    pub fn parse_store_link(&self, url: &str) -> Option<StoreId> {
        Some(match self {
            Self::Coles => StoreId::Coles(
                url.strip_prefix("https://www.coles.com.au/find-stores/coles/")?
                    .rsplit_once("-")?
                    .1
                    .parse()
                    .ok()?,
            ),
            Self::Woolworths => StoreId::Woolworths(
                url.strip_prefix("https://www.woolworths.com.au/shop/storelocator/")?
                    .rsplit_once("-")?
                    .1
                    .parse()
                    .ok()?,
            ),
        })
    }
}

impl fmt::Debug for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coles(x) => write!(
                f,
                "[Coles Store {x}](https://www.coles.com.au/find-stores/coles/-/-{x})"
            ),
            Self::Woolworths(x) => write!(
                f,
                "[Woolworths Store {x}](https://www.woolworths.com.au/shop/storelocator/{x})"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_store_link() {
        assert_eq!(
            Retailer::Coles
                .parse_store_link("https://www.coles.com.au/find-stores/coles/state/slug-1234"),
            Some(StoreId::Coles(1234))
        );
        assert_eq!(
            Retailer::Woolworths
                .parse_store_link("https://www.woolworths.com.au/shop/storelocator/slug-1234"),
            Some(StoreId::Woolworths(1234))
        );
    }
}
