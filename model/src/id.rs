use serde::{Deserialize, Serialize};

use crate::Retailer;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductID {
    Coles(u32),
    Woolworths(u32),
}

impl ProductID {
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
    pub fn parse_product_id(&self, id: &str) -> Option<ProductID> {
        Some(match self {
            Self::Coles => ProductID::Coles(id.parse().ok()?),
            Self::Woolworths => ProductID::Woolworths(id.parse().ok()?),
        })
    }

    pub fn parse_product_link(&self, url: &str) -> Option<ProductID> {
        Some(match self {
            Self::Coles => ProductID::Coles(
                url.strip_prefix("https://www.coles.com.au/product/")?
                    .rsplit_once("-")?
                    .1
                    .parse()
                    .ok()?,
            ),
            Self::Woolworths => {
                let slug =
                    url.strip_prefix("https://www.woolworths.com.au/shop/productdetails/")?;
                if slug == "/" {
                    return None;
                }

                ProductID::Woolworths(slug.split_once("/")?.0.parse().ok()?)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_product_link() {
        assert_eq!(
            Retailer::Coles.parse_product_link("https://www.coles.com.au/product/slug-1234"),
            Some(ProductID::Coles(1234))
        );
        assert_eq!(
            Retailer::Coles.parse_product_link("https://www.coles.com.au/product/slug"),
            None
        );

        assert_eq!(
            Retailer::Woolworths
                .parse_product_link("https://www.woolworths.com.au/shop/productdetails/1234/slug"),
            Some(ProductID::Woolworths(1234))
        );
        assert_eq!(
            // in sitemap
            Retailer::Woolworths
                .parse_product_link("https://www.woolworths.com.au/shop/productdetails//"),
            None
        );
    }
}
