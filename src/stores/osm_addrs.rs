use std::{
    collections::BTreeMap,
    fmt,
    fs::{read_to_string, write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use indicatif::ProgressBar;
use itertools::Itertools;
use serde::Deserialize;
use ureq::Agent;

use crate::{
    utils::{progress_style, EMAIL},
    OsmId, Vendor,
};

use super::models::StoreId;

pub fn load(vendor: Vendor, osm_ids: &BTreeMap<StoreId, OsmId>) -> Result<BTreeMap<OsmId, String>> {
    let raw_path = PathBuf::from(format!("data/stores/osm-addrs-{}.json", vendor.slug()));
    let mut raw: BTreeMap<OsmId, BTreeMap<String, String>> = if raw_path.exists() {
        serde_json::from_str(&read_to_string(&raw_path)?)?
    } else {
        BTreeMap::new()
    };

    let mut missing = Vec::new();
    for id in osm_ids.values() {
        if !raw.contains_key(id) {
            missing.push(id);
        }
    }

    if missing.len() > 0 {
        let agent = Agent::new();

        eprintln!(
            "Fetching OSM addresses for {} {vendor} stores...",
            missing.len()
        );
        let pb = ProgressBar::new(missing.len() as u64).with_style(progress_style());
        for chunk in missing.chunks(50) {
            let chunk = chunk.iter().join(",");

            let response: Vec<Geocoding> = agent
                .get(&format!(
                    "https://nominatim.openstreetmap.org/lookup?osm_ids={chunk}&format=json&email={EMAIL}"
                ))
                .call()?
                .into_json()?;
            pb.inc(response.len() as u64);
            for x in response {
                raw.insert(x.osm_id(), x.address);
            }
        }

        // write now to keep raw data
        write(raw_path, serde_json::to_string_pretty(&raw)?)?;
    }

    let mut output = BTreeMap::new();
    for (osm, addr) in raw {
        let mut name = String::new();
        for k in ["village", "town", "district", "city"] {
            if let Some(v) = addr.get(k) {
                if !name.is_empty() {
                    name.push(',');
                    name.push(' ');
                }

                name.push_str(
                    match v
                        .trim_end_matches(" City Council")
                        .trim_start_matches("District of ")
                    {
                        "Greater Brisbane" => "Brisbane",
                        "Gold Coast City" => "Gold Coast",
                        "Sunshine Coast Regional" => "Sunshine Coast",
                        x => x,
                    },
                )
            }
        }
        output.insert(osm, name);

        // let town = addr
        //     .town
        //     .map(|x| x.trim_start_matches("District of ").to_string());
        // let city = addr.city.map(|x| {
        //     x.trim_end_matches(" Council")
        //         .trim_end_matches(" City")
        //         .to_string()
        // });
    }

    Ok(output)
}

#[derive(Deserialize)]
struct Geocoding {
    osm_type: OsmType,
    osm_id: u64,
    address: BTreeMap<String, String>,
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

#[derive(Deserialize)]
pub struct OSMAddress {
    pub road: Option<String>,
}

// #[derive(Debug)]
// pub enum State {
//     NSW,
//     VIC,
//     QLD,
//     SA,
//     WA,
//     TAS,
//     NT,
//     ACT,
// }

// impl fmt::Display for State {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::NSW => write!(f, "NSW"),
//             Self::VIC => write!(f, "VIC"),
//             Self::QLD => write!(f, "QLD"),
//             Self::SA => write!(f, "SA"),
//             Self::WA => write!(f, "WA"),
//             Self::TAS => write!(f, "TAS"),
//             Self::NT => write!(f, "NT"),
//             Self::ACT => write!(f, "ACT"),
//         }
//     }
// }
