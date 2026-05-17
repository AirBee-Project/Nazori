use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use kasane_logic::{IterFlexIds, RangeId, SpatialIdSet};
use nazori::plateau;

fn main() {
    let gml = read_to_string("sample/plateau/tran/53394680_tran_6697_op.gml").unwrap();

    let a = plateau::bldg(&gml, 26, 0.1).unwrap();

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
