use std::collections::BTreeMap;

use _model::{OsmId, Retailer, StoreId};
use anyhow::Result;
use geo::Point;
use serde::{Deserialize, Serialize};

mod conflation;
mod nominatim;
mod parents;

fn main() -> Result<()> {
    let mut stores = Vec::new();
    for retailer in Retailer::all() {
        stores.extend(conflation::run(&retailer)?);
    }

    let parents = parents::run(&stores)?;
    let addresses = nominatim::run(&stores)?;

    for store in stores {
        let parent = parents.get(&store.osm);
        let address = addresses.get(&store.osm).unwrap();
        let retailer = store.id.retailer();
        let mut output = format!("{retailer}");
        if let Some(x) = address.places.first() {
            output.push(' ');
            output.push_str(x);
        }

        println!("{:?}", store.id);
        println!("{output}");
        let mut context = String::new();
        if let Some(p) = parent {
            context.push_str(p);
        }

        for place in address.places.iter() {
            if !context.is_empty() {
                context.push(',');
                context.push(' ');
            }
            context.push_str(place);
        }

        context.push_str(&format!(
            " {} {}",
            address.state,
            address.postcode.as_deref().unwrap_or_default()
        ));
        println!("{context}");
        println!();
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Store {
    #[serde(flatten)]
    pub id: StoreId,
    pub osm: OsmId,
    #[serde(flatten)]
    pub point: Point,
    // pub name: String,
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
    fn refine(self) -> OsmElement {
        match self {
            Self::Node { id, center, tags } => OsmElement {
                id: OsmId::Node(id),
                point: center.refine(),
                tags,
            },
            Self::Way { id, center, tags } => OsmElement {
                id: OsmId::Way(id),
                point: center.refine(),
                tags,
            },
            Self::Relation { id, center, tags } => OsmElement {
                id: OsmId::Relation(id),
                point: center.refine(),
                tags,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RawPosition {
    lat: f64,
    lon: f64,
}
impl RawPosition {
    fn refine(self) -> Point {
        Point::new(self.lat, self.lon)
    }
}

#[derive(Serialize, Deserialize)]
pub struct OsmElement {
    id: OsmId,
    #[serde(flatten)]
    point: Point,
    tags: BTreeMap<String, String>,
}
