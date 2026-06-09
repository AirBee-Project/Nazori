mod model;
mod parser;

pub use model::{TranAttribute, TranClass, TranFunction, TranUsage};

use crate::Error;
use kasane_logic::{CoverSingleIds, Polygon, SingleId};
use parser::TranParser;
use std::io::Cursor;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub mod single {
    use super::*;

    /// PLATEAU の道 XML を受け取り、属性情報と空間 ID (SingleId) のペアのイテレーターを返す。
    #[cfg(feature = "parallel")]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(TranAttribute, Vec<SingleId>), Error>> + 'a, Error>
    {
        if !xml.contains("tran:Road")
            && !xml.contains("tran:Square")
            && !xml.contains("tran:Railway")
            && !xml.contains("tran:Track")
        {
            return Err(Error::InvalidFormat(
                "The provided XML does not appear to contain any Transportation data (Road, Square, etc.).".to_string(),
            ));
        }

        let mut parser = TranParser::new(Cursor::new(xml.as_bytes()));

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

            let results: Vec<Result<(TranAttribute, Vec<SingleId>), Error>> = chunk
                .into_par_iter()
                .map(|item_res| {
                    let (attr, shape) = item_res.map_err(Error::Xml)?;
                    let mut ids = Vec::new();
                    for polygon_points in shape.surfaces {
                        let polygon = Polygon::new(polygon_points, epsilon);
                        ids.extend(polygon.cover_single_ids(zoom)?);
                    }
                    Ok((attr, ids))
                })
                .collect();

            Some(results.into_iter())
        })
        .flatten())
    }

    /// PLATEAU の道 XML を受け取り、空間 ID のイテレーターを返す。
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

    /// PLATEAU の道 XML を受け取り、属性情報と空間 ID (SingleId) のペアのイテレーターを返す。
    #[cfg(not(feature = "parallel"))]
    pub fn with_attr<'a>(
        xml: &'a str,
        zoom: u8,
        epsilon: f64,
    ) -> Result<impl Iterator<Item = Result<(TranAttribute, Vec<SingleId>), Error>> + 'a, Error>
    {
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
            TranParser::new(Cursor::new(xml.as_bytes())).map(move |item_res| {
                let (attr, shape) = item_res.map_err(Error::Xml)?;
                let mut ids = Vec::new();
                for polygon_points in shape.surfaces {
                    let polygon = Polygon::new(polygon_points, epsilon);
                    ids.extend(polygon.cover_single_ids(zoom)?);
                }
                Ok((attr, ids))
            }),
        )
    }

    /// PLATEAU の道 XML を受け取り、空間 ID のイテレーターを返す。
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
