use crate::ast::node::Node;
use crate::ast::edge::Edge;

#[derive(Debug)]
pub struct Graph {
    pub subgraphs: Vec<SubGraph>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug)]
pub struct SubGraph {
    pub id: String,
    pub parent: String,
}
