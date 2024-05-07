use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Write,
    fs,
};

use anyhow::{bail, Result};
use geo::Point;
use itertools::Itertools;
use serde::Serialize;

use crate::{utils::title_case, OsmId, Vendor};

use self::models::StoreId;

mod gnaf_addrs;
mod models;
mod osm_addrs;
mod osm_areas;
mod osm_elems;
mod osm_ids;
mod overpass;

pub fn main() -> Result<()> {
    let mut output = Vec::new();

    for vendor in Vendor::all() {
        let osm_elems = osm_elems::load(vendor)?;
        let osm_ids = osm_ids::load(vendor, &osm_elems)?;
        let gnaf_addrs = gnaf_addrs::load(vendor, &osm_elems)?;
        let osm_addrs = osm_addrs::load(vendor, &osm_ids)?;
        let osm_areas = osm_areas::load(vendor, &osm_elems, &osm_ids)?;

        for (store, osm) in &osm_ids {
            let elem = osm_elems.get(&osm).unwrap();
            let gnaf_addr = gnaf_addrs.get(&osm).unwrap();
            let osm_addr = osm_addrs.get(&osm).unwrap();

            let mut name = elem
                .tags
                .get("brand")
                .expect("couldn't have been found")
                .clone();
            name.push(' ');
            let locality = title_case(&gnaf_addr.locality);

            let mut desc = String::new();

            if let Some(x) = elem.tags.get("branch") {
                name.push_str(x);
            } else if let Some(x) = elem.tags.get("full_name") {
                name.push_str(x.trim_start_matches(&elem.tags["brand"]).trim())
            } else {
                name += &locality;
            }

            if let Some(x) = osm_areas.get(&osm) {
                write!(desc, "{x}, ")?;
            }
            if osm_addr.is_empty() {
                desc += &locality;
            } else {
                desc += &osm_addr;
            }
            write!(desc, " {} {}", gnaf_addr.state, gnaf_addr.postcode)?;

            output.push(Store {
                id: *store,
                name,
                desc,
                osm: *osm,
                point: elem.point.clone(),
            })
        }
    }

    fs::write(
        "data/stores/output.json",
        serde_json::to_string_pretty(&output)?,
    )?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Store {
    #[serde(flatten)]
    id: StoreId,
    name: String,
    desc: String,
    osm: OsmId,
    #[serde(flatten)]
    point: Point,
}
