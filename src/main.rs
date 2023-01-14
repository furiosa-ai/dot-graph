extern crate dot_parser;

use dot_parser::parser::parse;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let graph = parse(&path);

    println!("{:?}", graph.subgraphs);

    for node in &graph.nodes {
        println!("node [ id: {}, parent: {} ]", node.id, node.parent);
    }

    for edge in &graph.edges {
        println!("edge [ {} -> {} ]", edge.from, edge.to);
    }
}
