use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{read_to_string, write},
    path::Path,
};

use _model::{OsmId, Retailer, StoreId};
use anyhow::{bail, Result};
use geo::{HaversineDistance, Point};
use serde::{Deserialize, Serialize};
use ureq::AgentBuilder;

fn main() -> Result<()> {
    let path = Path::new("raw.json");
    let mut osm: BTreeMap<Retailer, Vec<OsmElement>>;
    if path.exists() {
        osm = serde_json::from_str(&read_to_string(path)?)?
    } else {
        let agent = AgentBuilder::new()
            .user_agent("price-shark-database (+https://github.com/priceshark/database)")
            .build();
        osm = BTreeMap::new();
        for x in Retailer::all() {
            let payload = match x {
                Retailer::Coles => include_str!("coles.overpassql"),
                Retailer::Woolworths => include_str!("woolworths.overpassql"),
            };
            eprintln!("Fetching {x:?} OSM data...");
            let response: OverpassResponse = agent
                .post("https://overpass-api.de/api/interpreter")
                .send_form(&[("data", &payload)])?
                .into_json()?;
            osm.insert(
                x,
                response.elements.into_iter().map(|x| x.refine()).collect(),
            );
        }
        write(path, serde_json::to_string_pretty(&osm)?)?;
    }

    let mut stores = BTreeMap::new();
    for (retailer, osm) in &osm {
        let prev_count = stores.len();
        println!("# {retailer:?}");

        let mut raw = Vec::new();
        for x in read_to_string(format!(
            "../internal/{}-stores/output.jsonl",
            retailer.slug()
        ))?
        .lines()
        {
            let x: RawStore = serde_json::from_str(&x)?;
            raw.push(x);
        }

        let mut forced = BTreeSet::new();
        for x in osm {
            if let Some(url) = x.tags.get("website") {
                if let Some(id) = retailer.parse_store_link(url) {
                    forced.insert(x.id.clone());
                    if let Some(conflict) = stores.insert(id.clone(), x) {
                        bail!(
                            "Two elements reference store {id:?}: {} {}",
                            x.id.link(),
                            conflict.id.link()
                        );
                    }
                }
            }
        }

        println!("## Warnings");
        let mut nearby: BTreeMap<StoreId, Vec<(f64, &OsmElement)>> = BTreeMap::new();
        let mut nearest: BTreeMap<OsmId, f64> = BTreeMap::new();
        for store in &raw {
            let id = match retailer {
                Retailer::Coles => StoreId::Coles(store.id),
                Retailer::Woolworths => StoreId::Woolworths(store.id),
            };
            if stores.contains_key(&id) {
                // matched based on url
                continue;
            }

            // find stores within 1km
            let mut this_nearby = Vec::new();
            for x in osm {
                if forced.contains(&x.id) {
                    // matched based on url
                    continue;
                }

                let distance = store.point.haversine_distance(&x.point);
                if distance < 1000.0 {
                    if let Some(x) = nearest.get_mut(&x.id) {
                        if *x > distance {
                            *x = distance;
                        }
                    } else {
                        nearest.insert(x.id.clone(), distance);
                    }

                    this_nearby.push((distance, x));
                }
            }
            this_nearby.sort_by(|(a, _), (b, _)| a.total_cmp(b));
            this_nearby.reverse(); // can use .pop() to get nearest
            nearby.insert(id, this_nearby);
        }

        let mut keep_going: bool = true;
        let mut pass = 0;
        while keep_going {
            keep_going = false;
            pass += 1;

            for (store, this_nearby) in nearby.iter_mut() {
                if let Some((d, next)) = this_nearby.pop() {
                    keep_going = true;

                    if *nearest.get(&next.id).unwrap() == d {
                        if let Some(x) = stores.get(store) {
                            println!("- {store:?} conflict: {} {}", x.id.link(), next.id.link());
                        } else {
                            if d > 400.0 || pass > 1 {
                                // TODO: keep decreasing and reviewing warning distance
                                println!(
                                    "- {store:?} is #{pass} closest ({d:.00}m) to {}",
                                    next.id.link()
                                );
                            }
                            stores.insert(store.clone(), next);
                        }
                    }
                }
            }
        }

        let new = stores.len() - prev_count;
        println!("## Statistics");
        println!("- {} raw, {} osm", raw.len(), osm.len());
        println!("- {} sourced from url", forced.len());
        println!("- {} matched on distance", new - forced.len());
        println!("- {:.01}% conflated", new as f64 / raw.len() as f64 * 100.0);
        println!();
    }

    let mut output = Vec::new();
    for (id, osm) in stores {
        output.push(Store {
            id,
            osm: osm.id.clone(),
            point: osm.point,
        });
    }
    let mut output = serde_json::to_string_pretty(&output)?;
    output.push('\n');
    write("output.json", output)?;

    Ok(())
}

#[derive(Deserialize)]
struct RawStore {
    id: u32,
    #[serde(flatten)]
    point: Point,
}

#[derive(Serialize, Deserialize)]
pub struct Store {
    #[serde(flatten)]
    pub id: StoreId,
    #[serde(flatten)]
    pub osm: OsmId,
    #[serde(flatten)]
    pub point: Point,
    // pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct OsmElement {
    #[serde(flatten)]
    id: OsmId,
    #[serde(flatten)]
    point: Point,
    tags: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct OverpassResponse {
    elements: Vec<RawElement>,
}

#[derive(Serialize, Deserialize)]
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
