use std::fs;
use std::str;

use dot_graph::prelude::*;

#[test]
fn parse_print_parse() -> Result<(), DotGraphError> {
    // all example dot files are from Graphviz gallery
    // https://graphviz.org/gallery/
    for file in fs::read_dir("./tests/examples").unwrap() {
        let path = file.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".dot") {
            // first parse from file
            let graph = parser::parse_from_file(path)?;

            // then print to dot
            let mut dot = Vec::new();
            graph.to_dot(&mut dot).expect("to_dot should succeed");
            let dot = str::from_utf8(&dot).unwrap();

            // then parse from memory
            parser::parse_from_memory(dot)?;
        }
    }

    Ok(())
}
