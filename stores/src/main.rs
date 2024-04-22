use _model::{OsmId, Retailer, StoreId};
use anyhow::Result;
use geo::Point;
use serde::{Deserialize, Serialize};

mod conflation;

fn main() -> Result<()> {
    let mut stores = Vec::new();
    for retailer in Retailer::all() {
        stores.extend(conflation::run(&retailer)?);
    }

    Ok(())
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
