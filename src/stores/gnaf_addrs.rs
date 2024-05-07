use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::Result;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::{OsmId, Vendor};

use super::overpass::OsmElement;

pub fn load(
    vendor: Vendor,
    osm_elems: &BTreeMap<OsmId, OsmElement>,
) -> Result<BTreeMap<OsmId, GNAFAddress>> {
    let raw_path = PathBuf::from(format!("data/stores/gnaf-addrs-{}.json", vendor.slug()));
    let mut raw: BTreeMap<OsmId, GNAFAddress> = if raw_path.exists() {
        serde_json::from_str(&fs::read_to_string(&raw_path)?)?
    } else {
        BTreeMap::new()
    };

    let mut missing = Vec::new();
    for (id, x) in osm_elems {
        if !raw.contains_key(id) {
            missing.push(x);
        }
    }

    if missing.len() > 0 {
        let agent = crate::agent();

        eprintln!(
            "Fetching GNAF addresses for {} {vendor} stores...",
            missing.len()
        );
        for elem in missing {
            let (x, y) = elem.point.x_y();

            let addr: GNAFAddress = agent
                .get(&format!("https://api.joel.net.au/gnafr/{x}/{y}"))
                .call()?
                .into_json()?;
            raw.insert(elem.id, addr);
        }

        fs::write(raw_path, serde_json::to_string_pretty(&raw)?)?;
    }

    Ok(raw)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GNAFAddress {
    pub latitude: f64,
    pub longitude: f64,
    pub locality: String,
    pub postcode: String,
    pub state: String,
}
