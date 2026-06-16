use kasane_logic::Coordinate;

/// PLATEAU 道路の分類（TranClass）を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub enum TranClass {
    NationalRoad,    // "一般国道"
    PrefecturalRoad, // "都道府県道"
    MunicipalRoad,   // "市町村道"
    Expressway,      // "高速自動車国道等"
    Other(String),
}

impl From<&str> for TranClass {
    fn from(value: &str) -> Self {
        match value {
            "一般国道" => Self::NationalRoad,
            "都道府県道" => Self::PrefecturalRoad,
            "市町村道" => Self::MunicipalRoad,
            "高速自動車国道等" => Self::Expressway,
            _ => Self::Other(value.to_string()),
        }
    }
}

/// PLATEAU 道路の機能（TranFunction）を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub enum TranFunction {
    Carriageway,  // "車道"
    Sidewalk,     // "歩道"
    BicycleLane,  // "自転車道"
    Intersection, // "交差点"
    Other(String),
}

impl From<&str> for TranFunction {
    fn from(value: &str) -> Self {
        match value {
            "車道" => Self::Carriageway,
            "歩道" => Self::Sidewalk,
            "自転車道" => Self::BicycleLane,
            "交差点" => Self::Intersection,
            _ => Self::Other(value.to_string()),
        }
    }
}

/// PLATEAU 道路の用途（TranUsage）を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub enum TranUsage {
    Car,        // "自動車交通"
    Pedestrian, // "歩行者交通"
    Bicycle,    // "自転車交通"
    Other(String),
}

impl From<&str> for TranUsage {
    fn from(value: &str) -> Self {
        match value {
            "自動車交通" => Self::Car,
            "歩行者交通" => Self::Pedestrian,
            "自転車交通" => Self::Bicycle,
            _ => Self::Other(value.to_string()),
        }
    }
}

/// PLATEAU の道属性をそのまま保持する型。
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
pub struct TranAttribute {
    pub gml_id: String,
    pub class: Option<TranClass>,
    pub function: Option<TranFunction>,
    pub usage: Option<TranUsage>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct TranShape {
    pub surfaces: Vec<Vec<Coordinate>>,
}
