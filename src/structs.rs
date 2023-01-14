use std::collections::HashMap;

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

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub parent: String,
    pub attrs: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Edge {
    pub from: String,
    pub to: String,
}
