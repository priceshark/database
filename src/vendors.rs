use core::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Vendor {
    Coles,
    Woolworths,
}

impl fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Vendor {
    pub fn all() -> Vec<Self> {
        vec![Vendor::Coles, Vendor::Woolworths]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Coles => "Coles",
            Self::Woolworths => "Woolworths",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Coles => "coles",
            Self::Woolworths => "woolworths",
        }
    }
}
