use std::{
    collections::BTreeMap,
    fs::{read_to_string, write},
    path::{Path, PathBuf},
};

use _model::OsmId;
use anyhow::Result;
use indicatif::{ProgressIterator, ProgressStyle};
use ureq::{post, Agent};

use crate::{OsmElement, OverpassResponse, Store};

const QUERY: &str = include_str!("parents.overpassql");

pub fn run(stores: &Vec<Store>) -> Result<()> {
    let raw_path = Path::new("raw/parents.json");
    let mut raw: BTreeMap<OsmId, Vec<OsmElement>> = if raw_path.exists() {
        serde_json::from_str(&read_to_string(raw_path)?)?
    } else {
        BTreeMap::new()
    };

    let mut missing = Vec::new();
    for store in stores {
        if !raw.contains_key(&store.osm) {
            missing.push(store);
        }
    }

    if missing.len() > 0 {
        let agent = Agent::new();

        eprintln!("Fetching parents of {} stores...", missing.len());
        for store in missing
            .iter()
            .progress_with_style(ProgressStyle::with_template(
                "{percent}% {pos}/{len} ({eta_precise})",
            )?)
        {
            let payload = QUERY.replace(
                "LOCATION",
                &format!("{},{}", store.point.x(), store.point.y()),
            );
            let response: OverpassResponse = agent
                .post("https://overpass-api.de/api/interpreter")
                .send_form(&[("data", &payload)])?
                .into_json()?;
            raw.insert(
                store.osm.clone(),
                response.elements.into_iter().map(|x| x.refine()).collect(),
            );
        }

        write(raw_path, serde_json::to_string_pretty(&raw)?)?;
    }

    todo!()
}
