use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use kasane_logic::{IterFlexIds, RangeId, SpatialIdSet};
use nazori::plateau;

fn main() {
    let gml =
        read_to_string("C:/Users/tomor/Downloads/13113_shibuya-ku_pref_2025_citygml_1_op/udx/tran/53393575_tran_6697_op.gml")
            .unwrap();

    let a = plateau::tran(&gml, 24, 0.0).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("output.txt")
        .unwrap();

    let mut set = SpatialIdSet::new();

    for res in a {
        match res {
            Ok(v) => {
                set.insert(v);
            }
            Err(e) => {
                eprintln!("Error occurred: {}", e);
            }
        }
    }

    for ele in set.iter_flex_ids() {
        let _ = write!(file, "{},", RangeId::from(ele));
    }
}
