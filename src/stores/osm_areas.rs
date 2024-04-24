use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs::{read_to_string, write},
    path::PathBuf,
};

use anyhow::Result;
use indicatif::ProgressIterator;

use crate::{stores::overpass, utils::progress_style, OsmId, Vendor};

use super::models::StoreId;

pub fn load(
    vendor: Vendor,
    osm_elems: &BTreeMap<OsmId, overpass::OsmElement>,
    osm_ids: &BTreeMap<StoreId, OsmId>,
) -> Result<BTreeMap<OsmId, String>> {
    let raw_path = PathBuf::from(format!("data/stores/osm-areas-{}.json", vendor.slug()));
    let mut raw: BTreeMap<OsmId, Vec<overpass::OsmElement>> = if raw_path.exists() {
        serde_json::from_str(&read_to_string(&raw_path)?)?
    } else {
        BTreeMap::new()
    };

    let mut missing = Vec::new();
    for (store, osm) in osm_ids {
        if !raw.contains_key(&osm) {
            missing.push((store, osm));
        }
    }

    if missing.len() > 0 {
        eprintln!("Fetching areas for {} {vendor} stores...", missing.len());
        for (_, osm) in missing.iter().progress_with_style(progress_style()) {
            let elem = osm_elems
                .get(osm)
                .expect("stores should only reference known elements");

            let (x, y) = elem.point.x_y();
            raw.insert(
                **osm,
                overpass::query(&format!(
                    r#"
                        is_in({x},{y}) ->.b;
                        (
                        	area(pivot.b)["building"];
                        	relation(pivot.b)["building"];
                        	area(pivot.b)["landuse"="retail"];
                        	relation(pivot.b)["landuse"="retail"];
                        );
                        out tags center;
                    "#
                ))?,
            );
        }

        write(&raw_path, serde_json::to_string_pretty(&raw)?)?;
    }

    let mut output = BTreeMap::new();
    let mut todo = String::new();
    for (store, osm) in osm_ids {
        let raw = raw.get(osm).unwrap();
        let parents: Vec<_> = raw
            .iter()
            .filter(|x| !x.tags.get("shop").is_some_and(|x| x == "supermarket"))
            // .filter(|x| x.tags.get("shop").is_some_and(|x| x == "mall"))
            .collect();
        if parents.len() == 0 {
            writeln!(todo, "- {store:?} {osm:?} has no parents")?;
        }
        // if parents.len() == 2 {
        //     writeln!(
        //         todo,
        //         "- {:?} {:?} has multiple parents",
        //         store.id, store.osm
        //     )?;
        // }

        let names = BTreeSet::from_iter(parents.iter().flat_map(|x| x.tags.get("name")));
        // let websites = BTreeSet::from_iter(parents.iter().flat_map(|x| x.tags.get("website")));

        if names.len() > 1 {
            writeln!(
                todo,
                "- {store:?} {osm:?} has conflicting parent names: {names:?}",
            )?;
        } else {
            if let Some(x) = names.into_iter().next() {
                output.insert(*osm, x.clone());
            }
        }
    }
    write(raw_path.with_extension("md"), todo)?;

    Ok(output)
}
