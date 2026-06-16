use crate::plateau::{bldg, tran};
use kasane_logic::{IterFlexIds, RangeId, SpatialIdSet};
use std::fmt::Write as _;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert(
    xmls: Vec<String>,
    zoom: u8,
    epsilon: f64,
    target: &str, // "bldg" or "tran"
    method: &str, // "flat_single", "flat_range", "with_attr_single", "with_attr_range"
    format: &str, // "txt" or "json"
) -> Result<String, JsValue> {
    match (target, method, format) {
        ("bldg", "flat_single", "txt") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = bldg::single::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            let mut out = String::new();
            for ele in set.iter_flex_ids() {
                let _ = write!(out, "{},", RangeId::from(ele));
            }
            Ok(out)
        }
        ("bldg", "flat_single", "json") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = bldg::single::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            Ok(set.to_json())
        }
        ("bldg", "with_attr_single", _) => {
            let mut out = String::new();
            out.push_str("[");
            let mut first = true;
            for xml in xmls.iter() {
                let iter = bldg::single::with_attr(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    let (attr, ids) = res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                    if !first {
                        out.push_str(",");
                    }
                    first = false;
                    let attr_json = serde_json::to_string(&attr)
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    out.push_str(&format!("{{\"attribute\":{}, \"ids\":[", attr_json));
                    let mut first_id = true;
                    for id in ids {
                        if !first_id {
                            out.push_str(",");
                        }
                        first_id = false;
                        let _ = write!(out, "\"{}\"", id); // SingleId uses Display
                    }
                    out.push_str("]}");
                }
            }
            out.push_str("]");
            Ok(out)
        }
        ("bldg", "flat_range", "txt") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = bldg::range::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            let mut out = String::new();
            for ele in set.iter_flex_ids() {
                let _ = write!(out, "{},", RangeId::from(ele));
            }
            Ok(out)
        }
        ("bldg", "flat_range", "json") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = bldg::range::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            Ok(set.to_json())
        }
        ("bldg", "with_attr_range", _) => {
            let mut out = String::new();
            out.push_str("[");
            let mut first = true;
            for xml in xmls.iter() {
                let iter = bldg::range::with_attr(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    let (attr, ids) = res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                    if !first {
                        out.push_str(",");
                    }
                    first = false;
                    let attr_json = serde_json::to_string(&attr)
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    out.push_str(&format!("{{\"attribute\":{}, \"ids\":[", attr_json));
                    let mut first_id = true;
                    for id in ids {
                        if !first_id {
                            out.push_str(",");
                        }
                        first_id = false;
                        let _ = write!(out, "\"{}\"", RangeId::from(id));
                    }
                    out.push_str("]}");
                }
            }
            out.push_str("]");
            Ok(out)
        }
        ("tran", "flat_single", "txt") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = tran::single::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            let mut out = String::new();
            for ele in set.iter_flex_ids() {
                let _ = write!(out, "{},", RangeId::from(ele));
            }
            Ok(out)
        }
        ("tran", "flat_single", "json") => {
            let mut set = SpatialIdSet::new();
            for xml in xmls.iter() {
                let iter = tran::single::flat(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    set.insert(res.map_err(|e| JsValue::from_str(&e.to_string()))?);
                }
            }
            Ok(set.to_json())
        }
        ("tran", "with_attr_single", _) => {
            let mut out = String::new();
            out.push_str("[");
            let mut first = true;
            for xml in xmls.iter() {
                let iter = tran::single::with_attr(xml, zoom, epsilon)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                for res in iter {
                    let (attr, ids) = res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                    if !first {
                        out.push_str(",");
                    }
                    first = false;
                    let attr_json = serde_json::to_string(&attr)
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    out.push_str(&format!("{{\"attribute\":{}, \"ids\":[", attr_json));
                    let mut first_id = true;
                    for id in ids {
                        if !first_id {
                            out.push_str(",");
                        }
                        first_id = false;
                        let _ = write!(out, "\"{}\"", id); // SingleId
                    }
                    out.push_str("]}");
                }
            }
            out.push_str("]");
            Ok(out)
        }
        _ => Err(JsValue::from_str("Invalid combinations")),
    }
}
