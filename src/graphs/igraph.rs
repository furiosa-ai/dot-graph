use crate::{edge::Edge, graphs::subgraph::SubGraph, node::Node};
use bimap::BiMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct IGraph {
    pub id: String,
    pub subgraphs: Vec<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl IGraph {
    pub fn encode(
        &self,
        slookup: &BiMap<String, usize>,
        nlookup: &BiMap<String, usize>,
        elookup: &BiMap<(String, String), usize>,
    ) -> SubGraph {
        let subgraphs: Vec<usize> = (self.subgraphs.par_iter())
            .map(|subgraph| slookup.get_by_left(subgraph).unwrap())
            .cloned()
            .collect();

        let nodes: Vec<usize> = (self.nodes.par_iter())
            .map(|node| nlookup.get_by_left(&node.id).unwrap())
            .cloned()
            .collect();

        let edges: Vec<usize> = (self.edges.par_iter())
            .map(|edge| elookup.get_by_left(&(edge.from.clone(), edge.to.clone())).unwrap())
            .cloned()
            .collect();

        SubGraph { id: self.id.clone(), subgraphs, nodes, edges }
    }
}
