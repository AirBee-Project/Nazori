use insta::glob;

use nazori::plateau;
use std::fs;

#[test]
fn test_tran_snapshots() {
    glob!("../../sample/plateau/tran", "**/*.gml", |path| {
        let gml = fs::read_to_string(path).unwrap();
        let a = plateau::tran::single::flat(&gml, 24, 1e-9).unwrap();

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

#[test]
fn test_tran_with_attr_snapshots() {
    glob!("../../sample/plateau/tran", "**/*.gml", |path| {
        let gml = fs::read_to_string(path).unwrap();
        let a = plateau::tran::single::with_attr(&gml, 24, 1e-9).unwrap();

        let mut results: Vec<String> = Vec::new();

        for res in a {
            match res {
                Ok((attr, ids)) => {
                    results.push(format!("{:?} - {} IDs", attr, ids.len()));
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
