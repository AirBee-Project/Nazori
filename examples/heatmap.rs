use std::fs::read_to_string;

use kasane_logic::SpatialIdTable;

/// PLATEAUの建物データと道路データからドローンのリスクをマップしたヒートマップを出力する
fn main() {
    let bldg_xml = read_to_string("../sample/plateau/heatmap/53394598_bldg_6697_op.gml").unwrap();
    let tran_xml = read_to_string("../sample/plateau/heatmap/53394598_tran_6697_op.gml").unwrap();
}

/// 道路データから空間リスクを返す
fn tran_risk() -> SpatialIdTable<u8> {
    todo!()
}

/// 建物データから空間リスクを返す
fn bldg_risk() -> SpatialIdTable<u8> {
    todo!()
}
