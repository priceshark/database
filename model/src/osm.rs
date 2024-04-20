use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OsmId {
    Node(u64),
    Way(u64),
    Relation(u64),
}

impl fmt::Debug for OsmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(x) => write!(f, "[n{x}](https://www.openstreetmap.org/node/{x})"),
            Self::Way(x) => write!(f, "[w{x}](https://www.openstreetmap.org/way/{x})"),
            Self::Relation(x) => write!(f, "[r{x}](https://www.openstreetmap.org/relation/{x})"),
        }
    }
}
