use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs::{read_to_string, write},
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use geo::{HaversineDistance, Point};
use serde::{Deserialize, Serialize};

use crate::{OsmId, Vendor};

use super::{models::StoreId, overpass::OsmElement};

// metres
const MATCH_RADIUS: f64 = 500.0;
const WARNING_RADIUS: f64 = 400.0;

pub fn load(
    vendor: Vendor,
    osm_elems: &BTreeMap<OsmId, OsmElement>,
) -> Result<BTreeMap<StoreId, OsmId>> {
    let output_path = PathBuf::from(format!("data/stores/osm-ids-{}.jsonl", vendor.slug()));
    if output_path.exists() {
        let mut output = BTreeMap::new();
        for line in read_to_string(output_path)?.lines() {
            let dump: DataDump = serde_json::from_str(line)?;
            output.insert(dump.id, dump.osm);
        }
        return Ok(output);
    }

    let mut output = BTreeMap::new();
    let mut todo = String::new();

    let mut raw = Vec::new();
    for x in read_to_string(format!("internal/{}-stores/output.jsonl", vendor.slug()))
        .context("failed to read internal data source")?
        .lines()
    {
        let x: RawStore = serde_json::from_str(&x)?;
        raw.push(x);
    }

    let mut forced = BTreeSet::new();
    for x in osm_elems.values() {
        if let Some(url) = x.tags.get("website") {
            if let Some(id) = vendor.parse_store_link(url) {
                forced.insert(x.id.clone());
                if let Some(conflict) = output.insert(id, x.id) {
                    bail!(
                        "Two elements reference store {id:?}: {:?} {:?}",
                        x.id,
                        conflict
                    );
                }
            }
        }
    }

    let mut nearby: BTreeMap<StoreId, Vec<(f64, &OsmElement)>> = BTreeMap::new();
    let mut nearest: BTreeMap<OsmId, f64> = BTreeMap::new();
    for store in &raw {
        let id = match vendor {
            Vendor::Coles => StoreId::Coles(store.id),
            Vendor::Woolworths => StoreId::Woolworths(store.id),
        };
        if output.contains_key(&id) {
            // matched based on url
            continue;
        }

        // find osm objects nearby
        let mut this_nearby = Vec::new();
        for x in osm_elems.values() {
            if forced.contains(&x.id) {
                // matched based on url
                continue;
            }

            let distance = store.point.haversine_distance(&x.point);
            if distance < MATCH_RADIUS {
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

    let missing = nearby
        .iter()
        .filter(|(_, x)| x.len() == 0)
        .collect::<Vec<_>>();

    if missing.len() > 0 {
        writeln!(todo, "- didn't match with osm:")?;
        for (k, _) in missing {
            writeln!(todo, "  - {k:?}")?;
        }
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
                    if let Some(x) = output.get(store) {
                        writeln!(
                            todo,
                            "- {store:?} is conflicted with {:?} and {:?}",
                            x, next.id
                        )?;
                    } else {
                        if pass > 1 {
                            writeln!(
                                todo,
                                "- {:?} is #{pass} closest to {store:?} ({d:.00}m)",
                                next.id
                            )?;
                        } else if d > WARNING_RADIUS {
                            writeln!(
                                todo,
                                "- {store:?}: {:?} is {d:.00}m away from raw location",
                                next.id
                            )?;
                        }
                        output.insert(store.clone(), next.id);
                    }
                }
            }
        }
    }

    let mut existing = BTreeSet::from_iter(forced.iter());
    existing.extend(output.iter().map(|(_, v)| v));
    let missing = osm_elems
        .iter()
        .map(|x| x.1.id)
        .filter(|x| !existing.contains(x))
        .collect::<Vec<_>>();
    if missing.len() > 0 {
        writeln!(todo, "- didn't match with raw:")?;
        for x in missing {
            writeln!(todo, "  - {x:?}")?;
        }
    }

    let mut md = String::new();
    writeln!(md, "## Statistics\n")?;
    writeln!(md, "- {} raw, {} osm", raw.len(), osm_elems.len())?;
    writeln!(md, "- {} sourced from url", forced.len())?;
    writeln!(md, "- {} matched on distance", output.len() - forced.len())?;
    writeln!(
        md,
        "- {:.01}% conflated",
        output.len() as f64 / raw.len() as f64 * 100.0
    )?;
    writeln!(md)?;
    if !todo.is_empty() {
        writeln!(md, "## Todo\n")?;
        writeln!(md, "{todo}")?;
    }
    write(output_path.with_extension("md"), md)?;

    let mut contents = String::new();
    for (id, osm) in &output {
        let dump = DataDump { id: *id, osm: *osm };
        contents.push_str(&serde_json::to_string(&dump)?);
        contents.push('\n');
    }
    write(output_path, contents)?;

    Ok(output)
}

#[derive(Deserialize)]
struct RawStore {
    id: u32,
    #[serde(flatten)]
    point: Point,
}

#[derive(Deserialize, Serialize)]
struct DataDump {
    #[serde(flatten)]
    id: StoreId,
    osm: OsmId,
}
