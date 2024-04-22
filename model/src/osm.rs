use std::{
    fmt::{self, Display},
    str::FromStr,
};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr)]
pub enum OsmId {
    Node(u64),
    Relation(u64),
    Way(u64),
}

impl Display for OsmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(x) => write!(f, "n{x}"),
            Self::Relation(x) => write!(f, "r{x}"),
            Self::Way(x) => write!(f, "w{x}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseOsmIdError;

impl Display for ParseOsmIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse OSM ID")
    }
}

impl FromStr for OsmId {
    type Err = ParseOsmIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= 2 {
            return Err(ParseOsmIdError);
        }

        let (p, x) = s.split_at(1);
        match p {
            "n" => Ok(OsmId::Node(x.parse().map_err(|_| ParseOsmIdError)?)),
            "w" => Ok(OsmId::Way(x.parse().map_err(|_| ParseOsmIdError)?)),
            "r" => Ok(OsmId::Relation(x.parse().map_err(|_| ParseOsmIdError)?)),
            _ => Err(ParseOsmIdError),
        }
    }
}

impl OsmId {
    pub fn overpass(&self) -> String {
        match self {
            Self::Node(x) => format!("node({x})"),
            Self::Relation(x) => format!("relation({x})"),
            Self::Way(x) => format!("way({x})"),
        }
    }
}

impl fmt::Debug for OsmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(x) => write!(f, "[n{x}](https://www.openstreetmap.org/node/{x})"),
            Self::Relation(x) => write!(f, "[r{x}](https://www.openstreetmap.org/relation/{x})"),
            Self::Way(x) => write!(f, "[w{x}](https://www.openstreetmap.org/way/{x})"),
        }
    }
}
