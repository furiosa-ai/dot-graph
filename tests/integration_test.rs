use std::str;

use dot_graph::prelude::*;

use serial_test::serial;

// all example dot files are from the Graphviz gallery
// https://graphviz.org/gallery/

#[test]
#[serial]
fn bazel_build_system() -> Result<(), DotGraphError> {
    parse_print_parse("bazel_build_system.dot")
}

#[test]
#[serial]
fn git_basics() -> Result<(), DotGraphError> {
    parse_print_parse("git_basics.dot")
}

#[test]
#[serial]
fn linux_kernel_diagram() -> Result<(), DotGraphError> {
    parse_print_parse("linux_kernel_diagram.dot")
}

#[test]
#[serial]
fn neural_network_keras() -> Result<(), DotGraphError> {
    parse_print_parse("neural_network_keras.dot")
}

#[test]
#[serial]
fn uml_class_diagram_demo() -> Result<(), DotGraphError> {
    parse_print_parse("uml_class_diagram_demo.dot")
}

#[test]
#[serial]
fn world_dynamics() -> Result<(), DotGraphError> {
    parse_print_parse("world_dynamics.dot")
}

fn parse_print_parse(filename: &str) -> Result<(), DotGraphError> {
    let path = &format!("./tests/examples/{filename}");

    // first parse from file
    let graph = parser::parse_from_file(path)?;

    // then print to dot
    let mut dot = Vec::new();
    graph.to_dot(&mut dot).expect("to_dot should succeed");
    let dot = str::from_utf8(&dot).unwrap();

    // then parse from memory
    parser::parse_from_memory(dot)?;

    Ok(())
}
