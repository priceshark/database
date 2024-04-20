use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OsmId {
    Node(u64),
    Way(u64),
    Relation(u64),
}

impl OsmId {
    pub fn link(&self) -> String {
        match self {
            Self::Node(x) => format!("https://www.openstreetmap.org/node/{x}"),
            Self::Way(x) => format!("https://www.openstreetmap.org/way/{x}"),
            Self::Relation(x) => format!("https://www.openstreetmap.org/relation/{x}"),
        }
    }
}
