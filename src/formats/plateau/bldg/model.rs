use kasane_logic::Coordinate;
use ordered_float::OrderedFloat;

/// PLATEAU の建築物属性をそのまま保持する型。
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct BldgAttribute {
    pub gml_id: String,
    pub uro_building_id: String,
    pub uro_city_code: String,
    pub class_code: String,
    pub measured_height: Option<OrderedFloat<f64>>,
    pub lod1_height_type: Option<i32>,
    pub uro_prefecture_code: Option<String>,
    pub usage_code: Option<i32>,
    pub usage: Option<BldgUsage>,
}

/// PLATEAU 建築物の用途（Usage / Class）を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BldgUsage {
    // ---- CityGML 標準（1000番台） ----
    Residential,      // 1000
    Commercial,       // 1010
    Public,           // 1020
    Industrial,       // 1030
    Agricultural,     // 1040
    Sports,           // 1050
    TrafficOrStorage, // 1070

    // ---- PLATEAU 拡張（3000番台） ----
    OrdinaryBuilding, // 3001
    RobustBuilding,   // 3002
    OrdinaryWallLess, // 3003
    RobustWallLess,   // 3004

    // ---- 都市計画基礎調査（400番台など） ----
    Retail,     // 411: 物品販売業店舗
    Restaurant, // 412: 飲食店
    Wholesale,  // 413: 卸売業店舗
    Amusement,  // 454: 遊技場
    Hotel,      // 461: 宿泊施設

    // ---- 分類しない・不明な分類 ----
    Unclassified, // 1060, 3000

    // ---- 未知のコード用 ----
    Other(i32),
}

impl From<i32> for BldgUsage {
    fn from(value: i32) -> Self {
        match value {
            1000 => Self::Residential,
            1010 => Self::Commercial,
            1020 => Self::Public,
            1030 => Self::Industrial,
            1040 => Self::Agricultural,
            1050 => Self::Sports,
            1060 | 3000 => Self::Unclassified,
            1070 => Self::TrafficOrStorage,
            411 => Self::Retail,
            412 => Self::Restaurant,
            413 => Self::Wholesale,
            454 => Self::Amusement,
            461 => Self::Hotel,
            3001 => Self::OrdinaryBuilding,
            3002 => Self::RobustBuilding,
            3003 => Self::OrdinaryWallLess,
            3004 => Self::RobustWallLess,
            _ => Self::Other(value),
        }
    }
}

/// 建築物の面群を保持する最小の形状型。
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BldgShape {
    pub surfaces: Vec<Vec<Coordinate>>,
}
