pub mod building;

use kasane_logic::{CoverSingleIds, Error, SingleId, Solid};

pub fn plateau_building(xml: &str, zoom: u8, epsilon: f64) -> Result<Vec<SingleId>, Error> {
    plateau_building_bytes(xml.as_bytes(), zoom, epsilon)
}

pub fn plateau_building_bytes(xml: &[u8], zoom: u8, epsilon: f64) -> Result<Vec<SingleId>, Error> {
    let mut result = Vec::new();

    for shape in building::parse_building_shapes(xml) {
        let solid = Solid::new(shape.surfaces, epsilon)?;
        result.extend(solid.cover_single_ids(zoom)?);
    }

    Ok(result)
}
