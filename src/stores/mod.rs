use anyhow::Result;

use crate::Vendor;

mod models;
mod osm_addrs;
mod osm_areas;
mod osm_elems;
mod osm_ids;
mod overpass;

pub fn main() -> Result<()> {
    for vendor in Vendor::all() {
        let osm_elems = osm_elems::load(vendor)?;
        let osm_ids = osm_ids::load(vendor, &osm_elems)?;
        let osm_addrs = osm_addrs::load(vendor, &osm_ids)?;
        let osm_areas = osm_areas::load(vendor, &osm_elems, &osm_ids)?;
    }

    Ok(())
}
