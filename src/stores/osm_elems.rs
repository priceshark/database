use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::Result;

use crate::{OsmId, Vendor};

use super::overpass;

pub fn load(vendor: Vendor) -> Result<BTreeMap<OsmId, overpass::OsmElement>> {
    let path = PathBuf::from(format!("data/stores/osm-elems-{}.json", vendor.slug()));
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        eprintln!("Fetching OSM elements for {vendor}...");
        let elems = overpass::query(match vendor {
            Vendor::Coles => {
                r#"
                    (
                        nwr["brand:wikidata"="Q1108172"];
                        nwr["brand:wikidata"="Q104850818"];
                    );
                    out tags center;
                "#
            }
            Vendor::Woolworths => {
                r#"
                    (
                        nwr["brand:wikidata"="Q3249145"];
                        nwr["brand:wikidata"="Q111772555"]["name"!="Woolworths MetroGo"];
                    );
                    out tags center;
                "#
            }
        })?;

        let mut map = BTreeMap::new();
        for x in elems {
            map.insert(x.id, x);
        }

        let mut contents = serde_json::to_string_pretty(&map)?;
        contents.push('\n');
        fs::write(path, &contents)?;
        Ok(map)
    }
}
