use kasane_logic::{ConflictPolicy, SpatialIdCollection, SpatialIdTable};
use nazori::plateau::{bldg::BldgAttribute, tran::TranAttribute};
use rayon::prelude::*;
use std::{
    fs::{self, read_to_string},
    time::Instant,
};

/// 解析に使用するズームレベル
const ZOOM_LEVEL: u8 = 25;

/// システム全体における「最も標準的な道路・用途」のベースリスク値
const TRNA_RISK_BASE: u32 = 40;

/// 建物における「最も標準的な用途・高さ」のベースリスク値
const BLDG_RISK_BASE: u32 = 70;

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

    // 3. 建物テーブルを F=0 に潰してから滑らかな水平スプレッド（減衰）を実行
    let t = Instant::now();
    let bldg_risk_map = bldg_table
        .clone()
        .plan()
        .level_f(ZOOM_LEVEL, 0, 0) // F=0に潰す
        .execution()
        .unwrap();
    println!(
        "[3] bldg spread (2D)     : {:>10} us (cells = {})",
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

    // 5. 道路テーブルを F=0 に潰し、道路の周辺にもソフトバッファースプレッドを実行
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

    // 6. 建物リスクと道路リスクをそれぞれ高さ方向に減衰させながら3D展開
    let t = Instant::now();
    let bldg_risk_map_3d = expand_bldg_to_3d(bldg_risk_map, &bldg_table);
    println!(
        "[6] bldg expand 3D       : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        bldg_risk_map_3d.count()
    );

    let t = Instant::now();
    let tran_risk_map_3d = expand_tran_to_3d(tran_risk_map);
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

    // 6.4. 隙間の無いようにデフォルト値（0）で埋める
    let t = Instant::now();
    let riskmap = riskmap.plan().fill_default(0u8).execution().unwrap();
    println!(
        "[6.4] fill_default 3D    : {:>10} us (cells = {})",
        t.elapsed().as_micros(),
        riskmap.count()
    );

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

/// 2Dの建物リスクマップを受け取り、実際の建物高さを考慮しながら高さ方向に減衰（1mあたり-4）させながら3D展開する
fn expand_bldg_to_3d(
    bldg_2d: SpatialIdTable<u8>,
    bldg_table: &SpatialIdTable<u8>,
) -> SpatialIdTable<u8> {
    // 1. 各 (X, Y) 座標における建物の最大高さ (F値) をマップ化
    let mut bldg_max_height = std::collections::HashMap::new();
    for (flex_id, _) in bldg_table.iter() {
        let key = (flex_id.x_index(), flex_id.y_index());
        bldg_max_height
            .entry(key)
            .and_modify(|e| *e = std::cmp::max(*e, flex_id.f_index()))
            .or_insert(flex_id.f_index());
    }

    // 2. 2Dマップのセルを高さ方向に展開
    let mut result = SpatialIdTable::new();
    for (flex_id, val) in bldg_2d.iter() {
        let x = flex_id.x_index();
        let y = flex_id.y_index();
        let h = bldg_max_height.get(&(x, y)).copied().unwrap_or(0);

        for f in 0..=100 {
            // 建物の高さ以下なら減衰なし、屋上より高くなれば1メートルごとに4ポイント減衰
            let decayed = if f <= h {
                *val
            } else {
                val.saturating_sub(((f - h) * 4) as u8)
            };
            if decayed > 0 {
                let new_id = flex_id.level_f(ZOOM_LEVEL, f, f).unwrap().next().unwrap();
                result.insert(new_id, decayed);
            } else {
                break; // リスクが0になったらそれ以上の高度はスキップ
            }
        }
    }
    result
}

/// 2Dの道路リスクマップを受け取り、高度に応じて急速に減衰（1mあたり-15）させながら3D展開する
fn expand_tran_to_3d(tran_2d: SpatialIdTable<u8>) -> SpatialIdTable<u8> {
    let mut result = SpatialIdTable::new();
    for (flex_id, val) in tran_2d.iter() {
        for f in 0..=100 {
            // 道路は地上 (F=0) にのみ存在するため、上空へ行くほど急速に減衰
            let decayed = val.saturating_sub((f * 15) as u8);
            if decayed > 0 {
                let new_id = flex_id.level_f(ZOOM_LEVEL, f, f).unwrap().next().unwrap();
                result.insert(new_id, decayed);
            } else {
                break; // リスクが0になったらそれ以上の高度はスキップ
            }
        }
    }
    result
}
