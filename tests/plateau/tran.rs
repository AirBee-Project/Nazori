use insta::glob;

use nazori::plateau;
use std::fs;

#[test]
fn test_tran_snapshots() {
    glob!("../../sample/plateau/tran", "**/*.gml", |path| {
        let gml = fs::read_to_string(path).unwrap();
        let a = plateau::tran(&gml, 23, 0.0).unwrap();

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
        let output = results.join(",\n");

        insta::assert_snapshot!(output);
    });
}
