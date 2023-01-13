use crate::ast::node::Node;
use crate::ast::edge::Edge;

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
