use insta::glob;

use nazori::plateau;
use std::fs;

#[test]
fn test_bldg_snapshots() {
    glob!("../../sample/plateau/bldg", "**/*.gml", |path| {
        let gml = fs::read_to_string(path).unwrap();
        let a = plateau::bldg(&gml, 23, 1e-9).unwrap();

        let mut results: Vec<String> = Vec::new();

        for res in a {
            match res {
                Ok(v) => {
                    results.push(format!("{}", v));
                }
                Err(e) => {
                    results.push(format!("Error: {}", e));
                }
            }
        }

        results.sort();
        results.dedup();
        let output = results.join(",\n");

        insta::assert_snapshot!(output);
    });
}
