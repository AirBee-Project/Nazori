use kasane_logic::{ConflictPolicy, SpatialIdCollection, SpatialIdTable};
use nazori::plateau::{bldg::BldgAttribute, tran::TranAttribute};
use rayon::prelude::*;
use std::{
    fs::{self, read_to_string},
    time::Instant,
};

/// 解析に使用するズームレベル
const ZOOM_LEVEL: u8 = 24;

/// システム全体における「最も標準的な道路・用途」のベースリスク値
const TRNA_RISK_BASE: u32 = 70;

/// 建物における「最も標準的な用途・高さ」のベースリスク値
const BLDG_RISK_BASE: u32 = 10;

/// 建物リスクを周囲へ同心円状に伝播させる半径（セル数）
const BLDG_SPREAD_RADIUS: u32 = 10;

/// 建物リスクの同心円伝播で1セル離れるごとに減衰させるポイント
const BLDG_SPREAD_DECAY: u32 = 10;

/// 道路・建物リスクが地上から覆う高さ（ZOOM_LEVEL=24 では F インデックス ≒ メートル）
const RISK_HEIGHT: i32 = 50;

/// PLATEAUの建物データと道路データからドローンのリスクをマップしたヒートマップを出力する
fn main() {
    let total = Instant::now();

    // 1. GML ファイルの読み込み
    let t = Instant::now();
    let bldg_xml = read_to_string("sample/plateau/heatmap/53394598_bldg_6697_op.gml").unwrap();
    let tran_xml = read_to_string("sample/plateau/heatmap/53394598_tran_6697_op.gml").unwrap();
    println!(
        "[1] read gml             : {:>10} us",
        t.elapsed().as_micros()
    );

    // 2. 建物データ -> SpatialIdTable 構築
    let t = Instant::now();
    let bldg_table = bldg_riskmap(bldg_xml);
    println!(
        "[2] bldg table build     : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        bldg_table.count()
    );

    // 3. 建物テーブルを F=0 に潰してから、建物の周りへ同心円状にリスクを伝播（減衰）させる
    let t = Instant::now();
    let bldg_risk_map = bldg_table
        .plan()
        .level_f(ZOOM_LEVEL, 0, 0) // F=0に潰す（高さ層の重複を除いて spread を軽くする）
        // 建物中心から離れるほどリスクが下がる同心円状の伝播。重なりは大きい方を採用(既定のMax)。
        .spread(ZOOM_LEVEL, BLDG_SPREAD_RADIUS, |v, dist| {
            let decayed = v.saturating_sub((dist * BLDG_SPREAD_DECAY) as u8);
            (decayed > 0).then_some(decayed)
        })
        .execution()
        .unwrap();
    println!(
        "[3] bldg spread          : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        bldg_risk_map.count()
    );

    // 4. 道路データ -> SpatialIdTable 構築
    let t = Instant::now();
    let tran_table = tran_riskmap(tran_xml);
    println!(
        "[4] tran table build     : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        tran_table.count()
    );

    // 5. 道路テーブルを F=0 に潰す（高さ方向の設定は 6.1 で level により行う）
    let t = Instant::now();
    let tran_risk_map = tran_table
        .plan()
        .level_f(ZOOM_LEVEL, 0, 0) // F=0に潰す
        .execution()
        .unwrap();
    println!(
        "[5] tran spread (2D)     : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        tran_risk_map.count()
    );

    // 6. 建物リスクは level で高さを揃え、地上から RISK_HEIGHT まで一定リスクのバンドにする
    let t = Instant::now();
    let bldg_risk_map_3d = bldg_risk_map
        .plan()
        .level_f(ZOOM_LEVEL, 0, RISK_HEIGHT)
        .execution()
        .unwrap();
    println!(
        "[6] bldg expand 3D       : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        bldg_risk_map_3d.count()
    );

    // 6.1. 道路リスクも建物と同じく level で地上から RISK_HEIGHT まで一定リスクのバンドにする
    let t = Instant::now();
    let tran_risk_map_3d = tran_risk_map
        .plan()
        .level_f(ZOOM_LEVEL, 0, RISK_HEIGHT)
        .execution()
        .unwrap();
    println!(
        "[6.1] tran expand 3D     : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        tran_risk_map_3d.count()
    );

    // 6.2. 3Dでの合成
    let t = Instant::now();
    let mut riskmap = bldg_risk_map_3d
        .plan()
        .union_with(tran_risk_map_3d, ConflictPolicy::Max)
        .execution()
        .unwrap();
    println!(
        "[6.2] union (bldg + tran) 3D: {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        riskmap.count()
    );

    // 6.3. 上空 100m までカバーするためのダミーセルを挿入
    let dummy_id = if let Some((first_id, _)) = riskmap.iter().next() {
        Some(
            first_id
                .level_f(ZOOM_LEVEL, 100, 100)
                .unwrap()
                .next()
                .unwrap(),
        )
    } else {
        None
    };
    if let Some(dummy_id) = dummy_id {
        riskmap.insert(dummy_id, 0u8);
    }

    // 7. JSON へシリアライズ
    let t = Instant::now();
    let result = riskmap.to_json();
    println!(
        "[7] to_json              : {:>10} us",
        t.elapsed().as_micros()
    );

    // 8. ファイル書き出し
    let t = Instant::now();
    fs::write("result.json", result).unwrap();
    println!(
        "[8] write file           : {:>10} us",
        t.elapsed().as_micros()
    );

    println!("--------------------------------------------");
    println!(
        "    total                : {:>10} us",
        total.elapsed().as_micros()
    );
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
///
/// 用途(usage) × 高さ(measured_height) × 構造種別(class) の3つの倍率を掛け合わせ、
/// 建物ごとにリスク値へばらつきを持たせる。
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

    // 2. 高さによる倍率（建物全体のリスクを変化させる主軸）
    //    高い建物ほど飛行高度に近く、衝突・落下時の影響が大きいためリスクが高い。
    //    1mあたり +8 ポイント加算し、高さによる差を大きく出す。
    let height_rate: u32 = match attr.measured_height {
        Some(h) => {
            // 例: 5m -> 140, 13m -> 204, 30m -> 340, 50m -> 500
            let h = h.into_inner().max(0.0) as u32;
            100 + h * 8
        }
        None => 100, // 高さ不明は標準(1.0倍)
    };

    // 3. 構造種別(bldg:class)による倍率（これまで未使用のパラメータ）
    //    堅牢な建物ほど規模が大きく密集、無壁舎(カーポート等)は人の滞留が少ない。
    let structure_rate: u32 = match attr.class_code.as_str() {
        "3001" => 100, // 普通建物
        "3002" => 130, // 堅牢建物（RC・鉄骨など、規模大）
        "3003" => 60,  // 普通無壁舎（カーポート等、滞留少）
        "3004" => 80,  // 堅牢無壁舎
        _ => 100,      // 不明・未分類
    };

    let total_rate = usage_rate * height_rate * structure_rate / (100 * 100);
    let calculated_point = (BLDG_RISK_BASE * total_rate) / 100;
    std::cmp::min(calculated_point, 100) as u8
}

/// 道路データから空間リスクを返す
fn tran_riskmap(xml: String) -> SpatialIdTable<u8> {
    let mut cells: Vec<_> = nazori::plateau::tran::single::with_attr(&xml, ZOOM_LEVEL, 0.0)
        .unwrap()
        .flat_map(|tran| {
            let (attr, ids) = tran.unwrap();
            let point = tran_point(attr);
            ids.into_iter().map(move |id| (id, point))
        })
        .collect();

    cells.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut result = SpatialIdTable::new();
    for (single_id, point) in cells {
        result.insert(single_id, point);
    }
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
