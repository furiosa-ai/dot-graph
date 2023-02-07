use crate::{edge::Edge, graphs::subgraph::SubGraph, node::Node};
use bimap::BiMap;
use rayon::prelude::*;

type SubGraphIndex = usize;
type NodeIndex = usize;
type EdgeIndex = usize;

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
        slookup: &BiMap<String, SubGraphIndex>,
        nlookup: &BiMap<String, NodeIndex>,
        elookup: &BiMap<(String, String), EdgeIndex>,
    ) -> SubGraph {
        let id = self.id.clone();

        let subgraph_idxs: Vec<SubGraphIndex> = (self.subgraphs.par_iter())
            .map(|subgraph| slookup.get_by_left(subgraph).unwrap())
            .cloned()
            .collect();

        let node_idxs: Vec<NodeIndex> = (self.nodes.par_iter())
            .map(|node| nlookup.get_by_left(&node.id).unwrap())
            .cloned()
            .collect();

        let edge_idxs: Vec<EdgeIndex> = (self.edges.par_iter())
            .map(|edge| elookup.get_by_left(&(edge.from.clone(), edge.to.clone())).unwrap())
            .cloned()
            .collect();

        SubGraph { id, subgraph_idxs, node_idxs, edge_idxs }
    }
}
