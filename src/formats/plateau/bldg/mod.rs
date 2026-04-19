mod model;
mod parser;

use kasane_logic::{CoverSingleIds, SingleId, Solid};

use crate::Error;

pub(crate) use parser::parse_bldg_shapes;

/// PLATEAU の建築物 XML を受け取り、空間 ID を返す。
pub fn plateau_bldg(xml: &str, zoom: u8, epsilon: f64) -> Result<Vec<SingleId>, Error> {
    let mut result = Vec::new();

    for shape in parse_bldg_shapes(xml.as_bytes()).map_err(Error::from)? {
        let solid = Solid::new(shape.surfaces, epsilon).map_err(Error::from)?;
        result.extend(solid.cover_single_ids(zoom).map_err(Error::from)?);
    }

    Ok(result)
}
