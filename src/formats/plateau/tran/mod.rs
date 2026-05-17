mod model;
mod parser;

use crate::Error;
use kasane_logic::{CoverSingleIds, Polygon, SingleId};
use parser::TranParser;
use std::io::Cursor;

/// PLATEAU の道 XML を受け取り、空間 ID のイテレーターを返す。
pub fn tran<'a>(
    xml: &'a str,
    zoom: u8,
    epsilon: f64,
) -> Result<impl Iterator<Item = Result<SingleId, Error>> + 'a, Error> {
    if !xml.contains("tran:Road")
        && !xml.contains("tran:Square")
        && !xml.contains("tran:Railway")
        && !xml.contains("tran:Track")
    {
        return Err(Error::InvalidFormat(
            "The provided XML does not appear to contain any Transportation data (Road, Square, etc.).".to_string(),
        ));
    }

    Ok(
        TranParser::new(Cursor::new(xml.as_bytes())).flat_map(move |item_res| {
            let process = move || -> Result<Vec<SingleId>, Error> {
                let (_, shape) = item_res?;
                let mut ids = Vec::new();
                for polygon_points in shape.surfaces {
                    let polygon = Polygon::new(polygon_points, epsilon);
                    ids.extend(polygon.cover_single_ids(zoom)?);
                }
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
