use kasane_logic::SpatialIdTable;
use nazori::plateau::{bldg::BldgAttribute, tran::TranAttribute};
use std::fs::read_to_string;

/// 解析に使用するズームレベル
const ZOOM_LEVEL: u8 = 21;

/// システム全体における「最も標準的な道路・用途」のベースリスク値
const TRNA_RISK_BASE: u32 = 10;

/// 建物における「最も標準的な用途・高さ」のベースリスク値
const BLDG_RISK_BASE: u32 = 10;

/// PLATEAUの建物データと道路データからドローンのリスクをマップしたヒートマップを出力する
fn main() {
    let bldg_xml = read_to_string("../sample/plateau/heatmap/53394598_bldg_6697_op.gml").unwrap();
    let tran_xml = read_to_string("../sample/plateau/heatmap/53394598_tran_6697_op.gml").unwrap();

    let bldg_risk_map = bldg_riskmap(bldg_xml);
    let tran_risk_map = tran_riskmap(tran_xml);
}

/// 建物データから空間リスクを返す
fn bldg_riskmap(xml: String) -> SpatialIdTable<u8> {
    let mut result = SpatialIdTable::new();

    nazori::plateau::bldg::range::with_attr(&xml, ZOOM_LEVEL, 0.0)
        .unwrap()
        .for_each(|bldg| {
            // 建物データを取り出す
            let (attr, ids) = bldg.unwrap();

            // 当該の建物のリスクを計算する
            let point = bldg_point(attr);

            for single_id in ids {
                result.insert(single_id, point);
            }
        });

    result
}

/// 建物1つ1つに対してリスク評価を行う (戻り値は 0 〜 100 の範囲)
fn bldg_point(attr: BldgAttribute) -> u8 {
    use nazori::plateau::bldg::BldgUsage;

    // 1. 用途による倍率（100 = 1.0倍 を基準とする）
    //    人の滞留・密集が多い用途ほどリスクが高い
    let usage_rate: u32 = match attr.usage {
        Some(v) => match v {
            BldgUsage::Residential => 200,      // 2.0倍: 住宅（在宅者あり）
            BldgUsage::Commercial => 300,       // 3.0倍: 商業（人の出入りが多い）
            BldgUsage::Public => 350,           // 3.5倍: 公共（学校・病院など）
            BldgUsage::Industrial => 150,       // 1.5倍: 工業
            BldgUsage::Agricultural => 80,      // 0.8倍: 農業（人が少ない）
            BldgUsage::Sports => 250,           // 2.5倍: スポーツ施設
            BldgUsage::TrafficOrStorage => 120, // 1.2倍: 交通・倉庫
            BldgUsage::Retail => 300,           // 3.0倍: 物品販売店舗
            BldgUsage::Restaurant => 320,       // 3.2倍: 飲食店
            BldgUsage::Wholesale => 200,        // 2.0倍: 卸売店舗
            BldgUsage::Amusement => 380,        // 3.8倍: 遊技場（密集）
            BldgUsage::Hotel => 330,            // 3.3倍: 宿泊施設
            BldgUsage::OrdinaryBuilding => 180, // 1.8倍: 一般建物
            BldgUsage::RobustBuilding => 220,   // 2.2倍: 堅牢建物
            BldgUsage::OrdinaryWallLess => 100, // 1.0倍: 普通無壁舎
            BldgUsage::RobustWallLess => 120,   // 1.2倍: 堅牢無壁舎
            BldgUsage::Unclassified => 100,
            BldgUsage::Other(_) => 100,
        },
        None => 100, // 指定なしは標準(1.0倍)
    };

    // 2. 高さによる倍率
    //    高い建物ほど飛行高度に近く、衝突・落下時の影響が大きいためリスクが高い
    let height_rate: u32 = match attr.measured_height {
        Some(h) => {
            // 高さ(m)に応じて 100(基準) から段階的に加算する
            // 例: 3m -> 115, 20m -> 200, 60m -> 400
            let h = h.into_inner().max(0.0) as u32;
            100 + h * 5
        }
        None => 100, // 高さ不明は標準(1.0倍)
    };

    let total_rate = (usage_rate * height_rate) / 100;
    let calculated_point = (BLDG_RISK_BASE * total_rate) / 100;
    std::cmp::min(calculated_point, 100) as u8
}

/// 道路データから空間リスクを返す
fn tran_riskmap(xml: String) -> SpatialIdTable<u8> {
    let mut result = SpatialIdTable::new();

    nazori::plateau::tran::single::with_attr(&xml, ZOOM_LEVEL, 0.0)
        .unwrap()
        .for_each(|tran| {
            // 道路データを取り出す
            let (attr, ids) = tran.unwrap();

            // 当該の道路のリスクを計算する
            let point = tran_point(attr);

            for single_id in ids {
                result.insert(single_id, point);
            }
        });

    result
}

/// 道路1つ1つに対してリスク評価を行う (戻り値は 0 〜 100 の範囲)
fn tran_point(attr: TranAttribute) -> u8 {
    // 1. 用途による倍率（100 = 1.0倍 を基準とする）
    let usage_rate: u32 = match attr.usage {
        Some(v) => match v {
            nazori::plateau::tran::TranUsage::Car => 100, // 1.0倍
            nazori::plateau::tran::TranUsage::Pedestrian => 300, // 3.0倍
            nazori::plateau::tran::TranUsage::Bicycle => 400, // 4.0倍
            nazori::plateau::tran::TranUsage::Other(_) => 100,
        },
        None => 100, // 指定なしは標準(1.0倍)
    };

    // 2. 道路クラスによる倍率
    let class_rate: u32 = match attr.class {
        Some(v) => match v {
            nazori::plateau::tran::TranClass::Expressway => 300, // 3.0倍
            nazori::plateau::tran::TranClass::NationalRoad => 200, // 2.0倍
            nazori::plateau::tran::TranClass::PrefecturalRoad => 150, // 1.5倍
            nazori::plateau::tran::TranClass::MunicipalRoad => 100, // 1.0倍
            nazori::plateau::tran::TranClass::Other(_) => 100,
        },
        None => 100,
    };

    let total_rate = (usage_rate * class_rate) / 100;
    let calculated_point = (TRNA_RISK_BASE * total_rate) / 100;
    std::cmp::min(calculated_point, 100) as u8
}
