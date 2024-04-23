use std::{
    collections::BTreeMap,
    fs::{read_to_string, write},
    path::Path,
};

use _model::{Address, OsmId, State};
use anyhow::{bail, Result};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use ureq::Agent;

use crate::Store;

pub fn run(stores: &Vec<Store>) -> Result<BTreeMap<OsmId, Address>> {
    let raw_path = Path::new("raw/nominatim.json");
    let mut raw: BTreeMap<OsmId, Value> = if raw_path.exists() {
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

        eprintln!("Fetching addresses for {} stores...", missing.len());
        for chunk in stores.chunks(50) {
            let chunk = chunk.iter().map(|x| &x.osm).join(",");

            let response: Vec<Geocoding> = agent
                .get(&format!(
                    "https://nominatim.openstreetmap.org/lookup?osm_ids={chunk}&format=json"
                ))
                .call()?
                .into_json()?;
            for x in response {
                raw.insert(x.osm_id(), x.address);
            }
            println!("{}", raw.len());
        }

        write(raw_path, serde_json::to_string_pretty(&raw)?)?;
    }

    let mut output = BTreeMap::new();
    for (osm, addr) in raw {
        // properly parse after writing as i would like to hold onto unused info for now
        let addr: RawAddress = serde_json::from_value(addr)?;

        let state = match addr.state_code.as_str() {
            "AU-ACT" => State::ACT,
            "AU-NSW" => State::NSW,
            "AU-NT" => State::NT,
            "AU-QLD" => State::QLD,
            "AU-SA" => State::SA,
            "AU-TAS" => State::TAS,
            "AU-VIC" => State::VIC,
            "AU-WA" => State::WA,
            x => bail!("Unknown state code: {x}"),
        };

        if let Some(x) = &addr.postcode {
            let _: u32 = x.parse()?;
            if x.len() != 4 {
                bail!("invalid postcode: {x}");
            }
        }

        let mut name = String::new();
        let places = [
            addr.suburb.as_deref(),
            addr.town
                .as_deref()
                .map(|x| x.trim_start_matches("District of ")),
            addr.city
                .as_deref()
                .map(|x| x.trim_end_matches(" Council").trim_end_matches(" City")),
        ]
        .iter()
        .flat_map(|x| x.map(|x| x.to_string()))
        .collect();

        output.insert(
            osm,
            Address {
                places,
                state,
                postcode: addr.postcode,
            },
        );
    }

    Ok(output)
}

#[derive(Deserialize)]
struct Geocoding {
    osm_type: OsmType,
    osm_id: u64,
    address: Value,
}

impl Geocoding {
    fn osm_id(&self) -> OsmId {
        match self.osm_type {
            OsmType::Node => OsmId::Node(self.osm_id),
            OsmType::Way => OsmId::Way(self.osm_id),
            OsmType::Relation => OsmId::Relation(self.osm_id),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OsmType {
    Node,
    Way,
    Relation,
}

// ignoring for now:
// - house numbers: less than half have one
// - roads: lots of parking aisle, bus stop, drive through names
#[derive(Debug, Deserialize)]
struct RawAddress {
    #[serde(rename = "ISO3166-2-lvl4")]
    state_code: String,
    postcode: Option<String>,
    suburb: Option<String>,
    town: Option<String>,
    city: Option<String>,
}
