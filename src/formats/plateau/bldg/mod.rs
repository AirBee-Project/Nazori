mod model;
mod parser;

pub use model::BldgAttribute;

use crate::Error;
use kasane_logic::{CoverRangeIds, CoverSingleIds, Polygon, RangeId, SingleId, Solid};
use parser::BldgParser;
use std::io::Cursor;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub mod single {
    use super::*;

    /// PLATEAU の建築物 XML を受け取り、属性情報と空間 ID (SingleId) のペアのイテレーターを返す。
    #[cfg(feature = "parallel")]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(BldgAttribute, Vec<SingleId>), Error>> + 'a, Error>
    {
        if !xml.contains("bldg:Building") && !xml.contains("<Building") {
            return Err(Error::InvalidFormat(
                "The provided XML does not appear to contain any Building data.".to_string(),
            ));
        }

        let mut parser = BldgParser::new(Cursor::new(xml.as_bytes()));

        Ok(std::iter::from_fn(move || {
            let mut chunk = Vec::with_capacity(1024);
            for _ in 0..1024 {
                match parser.next() {
                    Some(item) => chunk.push(item),
                    None => break,
                }
            }

            if chunk.is_empty() {
                return None;
            }

            let results: Vec<Result<(BldgAttribute, Vec<SingleId>), Error>> = chunk
                .into_par_iter()
                .map(|item_res| {
                    let (attr, shape) = item_res.map_err(Error::Xml)?;
                    let mut polygons = Vec::new();
                    for polygon_points in shape.surfaces {
                        let polygon = Polygon::new(polygon_points, epsilon);
                        polygons.push(polygon);
                    }
                    let solid = Solid::new(polygons, epsilon)?;
                    let ids = solid.cover_single_ids(zoom)?.collect::<Vec<_>>();
                    Ok((attr, ids))
                })
                .collect();

            Some(results.into_iter())
        })
        .flatten())
    }

    /// PLATEAU の建築物 XML を受け取り、空間 ID のイテレーターを返す。
    #[cfg(feature = "parallel")]
    pub fn flat<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<SingleId, Error>> + 'a, Error> {
        Ok(with_attr(xml, zoom, epsilon)?.flat_map(|res| match res {
            Ok((_, ids)) => Box::new(ids.into_iter().map(Ok))
                as Box<dyn Iterator<Item = Result<SingleId, Error>>>,
            Err(e) => Box::new(std::iter::once(Err(e))) as _,
        }))
    }

    /// PLATEAU の建築物 XML を受け取り、属性情報と空間 ID (SingleId) のペアのイテレーターを返す。
    #[cfg(not(feature = "parallel"))]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(BldgAttribute, Vec<SingleId>), Error>> + 'a, Error>
    {
        if !xml.contains("bldg:Building") && !xml.contains("<Building") {
            return Err(Error::InvalidFormat(
                "The provided XML does not appear to contain any Building data.".to_string(),
            ));
        }

        Ok(
            BldgParser::new(Cursor::new(xml.as_bytes())).map(move |item_res| {
                let (attr, shape) = item_res.map_err(Error::Xml)?;
                let mut polygons = Vec::new();
                for polygon_points in shape.surfaces {
                    let polygon = Polygon::new(polygon_points, epsilon);
                    polygons.push(polygon);
                }
                let solid = Solid::new(polygons, epsilon)?;
                let ids = solid.cover_single_ids(zoom)?.collect::<Vec<_>>();
                Ok((attr, ids))
            }),
        )
    }

    /// PLATEAU の建築物 XML を受け取り、空間 ID のイテレーターを返す。
    #[cfg(not(feature = "parallel"))]
    pub fn flat<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<SingleId, Error>> + 'a, Error> {
        Ok(with_attr(xml, zoom, epsilon)?.flat_map(|res| match res {
            Ok((_, ids)) => Box::new(ids.into_iter().map(Ok))
                as Box<dyn Iterator<Item = Result<SingleId, Error>>>,
            Err(e) => Box::new(std::iter::once(Err(e))) as _,
        }))
    }
}

pub mod range {
    use super::*;

    /// PLATEAU の建築物 XML を受け取り、属性情報と空間 ID (RangeId) のペアのイテレーターを返す。
    #[cfg(feature = "parallel")]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(BldgAttribute, Vec<RangeId>), Error>> + 'a, Error>
    {
        if !xml.contains("bldg:Building") && !xml.contains("<Building") {
            return Err(Error::InvalidFormat(
                "The provided XML does not appear to contain any Building data.".to_string(),
            ));
        }

        let mut parser = BldgParser::new(Cursor::new(xml.as_bytes()));

        Ok(std::iter::from_fn(move || {
            let mut chunk = Vec::with_capacity(1024);
            for _ in 0..1024 {
                match parser.next() {
                    Some(item) => chunk.push(item),
                    None => break,
                }
            }

            if chunk.is_empty() {
                return None;
            }

            let results: Vec<Result<(BldgAttribute, Vec<RangeId>), Error>> = chunk
                .into_par_iter()
                .map(|item_res| {
                    let (attr, shape) = item_res.map_err(Error::Xml)?;
                    let mut polygons = Vec::new();
                    for polygon_points in shape.surfaces {
                        let polygon = Polygon::new(polygon_points, epsilon);
                        polygons.push(polygon);
                    }
                    let solid = Solid::new(polygons, epsilon)?;
                    let ids = solid.cover_range_ids(zoom)?.collect::<Vec<_>>();
                    Ok((attr, ids))
                })
                .collect();

            Some(results.into_iter())
        })
        .flatten())
    }

    /// PLATEAU の建築物 XML を受け取り、空間 ID の範囲表現 (RangeId) のイテレーターを返す。
    #[cfg(feature = "parallel")]
    pub fn flat<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<RangeId, Error>> + 'a, Error> {
        Ok(with_attr(xml, zoom, epsilon)?.flat_map(|res| match res {
            Ok((_, ids)) => Box::new(ids.into_iter().map(Ok))
                as Box<dyn Iterator<Item = Result<RangeId, Error>>>,
            Err(e) => Box::new(std::iter::once(Err(e))) as _,
        }))
    }

    /// PLATEAU の建築物 XML を受け取り、属性情報と空間 ID (RangeId) のペアのイテレーターを返す。
    #[cfg(not(feature = "parallel"))]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(BldgAttribute, Vec<RangeId>), Error>> + 'a, Error>
    {
        if !xml.contains("bldg:Building") && !xml.contains("<Building") {
            return Err(Error::InvalidFormat(
                "The provided XML does not appear to contain any Building data.".to_string(),
            ));
        }

        Ok(
            BldgParser::new(Cursor::new(xml.as_bytes())).map(move |item_res| {
                let (attr, shape) = item_res.map_err(Error::Xml)?;
                let mut polygons = Vec::new();
                for polygon_points in shape.surfaces {
                    let polygon = Polygon::new(polygon_points, epsilon);
                    polygons.push(polygon);
                }
                let solid = Solid::new(polygons, epsilon)?;
                let ids = solid.cover_range_ids(zoom)?.collect::<Vec<_>>();
                Ok((attr, ids))
            }),
        )
    }

    /// PLATEAU の建築物 XML を受け取り、空間 ID の範囲表現 (RangeId) のイテレーターを返す。
    #[cfg(not(feature = "parallel"))]
    pub fn flat<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<RangeId, Error>> + 'a, Error> {
        Ok(with_attr(xml, zoom, epsilon)?.flat_map(|res| match res {
            Ok((_, ids)) => Box::new(ids.into_iter().map(Ok))
                as Box<dyn Iterator<Item = Result<RangeId, Error>>>,
            Err(e) => Box::new(std::iter::once(Err(e))) as _,
        }))
    }
}
