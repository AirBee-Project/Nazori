mod model;
mod parser;

use crate::Error;
use kasane_logic::{CoverSingleIds, Polygon, SingleId, Solid};
use parser::BldgParser;
use std::io::Cursor;

/// PLATEAU の建築物 XML を受け取り、空間 ID のイテレーターを返す。
pub fn bldg<'a>(
    xml: &'a str,
    zoom: u8,
    epsilon: f64,
) -> Result<impl Iterator<Item = Result<SingleId, Error>> + 'a, Error> {
    if !xml.contains("bldg:Building") && !xml.contains("<Building") {
        return Err(Error::InvalidFormat(
            "The provided XML does not appear to contain any Building data.".to_string(),
        ));
    }

    Ok(
        BldgParser::new(Cursor::new(xml.as_bytes())).flat_map(move |item_res| {
            let process = move || -> Result<Vec<SingleId>, Error> {
                let (_, shape) = item_res?;
                let mut polygons = Vec::new();
                for polygon_points in shape.surfaces {
                    let polygon = Polygon::new(polygon_points, epsilon);
                    polygons.push(polygon);
                }
                let solid = Solid::new(polygons, epsilon)?;
                let ids = solid.cover_single_ids(zoom)?.collect::<Vec<_>>();
                Ok(ids)
            };

            match process() {
                Ok(ids) => Box::new(ids.into_iter().map(Ok))
                    as Box<dyn Iterator<Item = Result<SingleId, Error>>>,
                Err(e) => Box::new(std::iter::once(Err(e))) as _,
            }
        }),
    )
}
