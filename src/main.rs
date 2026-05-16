use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use kasane_logic::{IterFlexIds, RangeId, SpatialIdSet};
use nazori::plateau;

fn main() {
    let gml = read_to_string("sample/tran.gml").unwrap();

    let a = plateau::tran(&gml, 25, 0.1);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("output.txt")
        .unwrap();

    let mut set = SpatialIdSet::new();

    for ele in a {
        set.insert(ele.unwrap())
    }

    for ele in set.iter_flex_ids() {
        let _ = write!(file, "{},", RangeId::from(ele));
    }
}
