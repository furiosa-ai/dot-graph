extern crate dot_graph;

use dot_graph::parser::parse;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut graph = parse(&path);
    graph.topsort();

    println!("{:?}", graph.subgraphs);

    for node in &graph.nodes {
        println!("node [ id: {}, parent: {} ]", node.id, node.parent);
    }

    for edge in &graph.edges {
        println!("edge [ {} -> {} ]", edge.from, edge.to);
    }
}
