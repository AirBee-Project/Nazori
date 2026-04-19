use kasane_logic::Coordinate;
use ordered_float::OrderedFloat;

/// PLATEAU の建築物属性をそのまま保持する型。
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) struct BldgAttribute {
    pub gml_id: String,
    pub uro_building_id: String,
    pub uro_city_code: String,
    pub class_code: String,
    pub measured_height: Option<OrderedFloat<f64>>,
    pub lod1_height_type: Option<i32>,
    pub uro_prefecture_code: Option<String>,
    pub usage_code: Option<i32>,
}

/// 建築物の面群を保持する最小の形状型。
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BldgShape {
    pub surfaces: Vec<Vec<Coordinate>>,
}
