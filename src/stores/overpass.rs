use std::collections::BTreeMap;

use anyhow::Result;
use geo::Point;
use serde::{Deserialize, Serialize};

use crate::{agent, OsmId};

pub fn query(q: &str) -> Result<Vec<OsmElement>> {
    let payload = format!("[out:json][timeout:25]; {q}");
    let response: OverpassResponse = agent()
        .post("https://overpass-api.de/api/interpreter")
        .send_form(&[("data", &payload)])?
        .into_json()?;

    Ok(response
        .elements
        .into_iter()
        .map(|x| x.simplify())
        .collect())
}

#[derive(Deserialize)]
struct OverpassResponse {
    elements: Vec<RawElement>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RawElement {
    Node {
        id: u64,
        #[serde(flatten)]
        center: RawPosition,
        tags: BTreeMap<String, String>,
    },
    Way {
        id: u64,
        center: RawPosition,
        tags: BTreeMap<String, String>,
    },
    Relation {
        id: u64,
        center: RawPosition,
        tags: BTreeMap<String, String>,
    },
}

impl RawElement {
    fn simplify(self) -> OsmElement {
        match self {
            Self::Node { id, center, tags } => OsmElement {
                id: OsmId::Node(id),
                point: center.simplify(),
                tags,
            },
            Self::Way { id, center, tags } => OsmElement {
                id: OsmId::Way(id),
                point: center.simplify(),
                tags,
            },
            Self::Relation { id, center, tags } => OsmElement {
                id: OsmId::Relation(id),
                point: center.simplify(),
                tags,
            },
        }
    }
}

#[derive(Deserialize)]
pub struct RawPosition {
    lat: f64,
    lon: f64,
}

impl RawPosition {
    fn simplify(self) -> Point {
        Point::new(self.lat, self.lon)
    }
}

#[derive(Deserialize, Serialize)]
pub struct OsmElement {
    pub id: OsmId,
    #[serde(flatten)]
    pub point: Point,
    pub tags: BTreeMap<String, String>,
}
