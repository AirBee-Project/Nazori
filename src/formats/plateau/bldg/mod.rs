mod model;
mod parser;

use crate::Error;
use kasane_logic::{CoverSingleIds, Polygon, SingleId, Solid};
use parser::BldgParser;
use std::io::Cursor;

#[cfg(feature = "parallel")]
use rayon::prelude::*;
#[cfg(feature = "parallel")]
use std::sync::mpsc;

/// PLATEAU の建築物 XML を受け取り、空間 ID のイテレーターを返す。
#[cfg(feature = "parallel")]
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

    let parser = BldgParser::new(Cursor::new(xml.as_bytes()));
    let items = parser.collect::<Result<Vec<_>, _>>().map_err(Error::Xml)?;

    let (tx, rx) = mpsc::sync_channel(1024);

    rayon::spawn(move || {
        items.into_par_iter().for_each_with(tx, |tx, item| {
            let process = || -> Result<Vec<SingleId>, Error> {
                let (_, shape) = item;
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
                Ok(ids) => {
                    for id in ids {
                        let _ = tx.send(Ok(id));
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });
    });

    // 4. イテレーターとしてチャネルの Receiver を返す
    Ok(rx.into_iter())
}

/// PLATEAU の建築物 XML を受け取り、空間 ID のイテレーターを返す。
#[cfg(not(feature = "parallel"))]
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
