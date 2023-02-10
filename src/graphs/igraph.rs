use crate::{
    edge::{Edge, EdgeId},
    graphs::{
        graph::GraphId,
        subgraph::SubGraph,
    },
    node::{Node, NodeId},
};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An `IGraph` is an intermediate representation, to be transformed to `SubGraph` after parsing.
/// It holds ids of its children subgraphs, nodes, and edges.
///
/// `SubGraph` is a more compact form of an `IGraph`, in the sense that it holds indices of
/// children subgraphs, nodes, and edges to be referenced in `Graph`.
pub struct IGraph {
    /// Name of the igraph
    pub id: GraphId,

    /// Ids of its children subgraphs
    pub subgraphs: HashSet<GraphId>,
    /// Its own nodes
    pub nodes: HashSet<Node>,
    /// Its own edges
    pub edges: HashSet<Edge>,
}

impl Hash for IGraph {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<GraphId> for IGraph {
    fn borrow(&self) -> &GraphId {
        &self.id
    }
}

impl IGraph {
    /// Convert `IGraph` to `SubGraph`
    pub fn encode(&self) -> SubGraph {
        let id = self.id.clone();

        let subgraph_ids: HashSet<GraphId> = self.subgraphs.clone();

        let node_ids: HashSet<NodeId> = (self.nodes.par_iter())
            .map(|node| node.id.clone())
            .collect();

        let edge_ids: HashSet<EdgeId> = (self.edges.par_iter())
            .map(|edge| edge.id.clone())
            .collect();

        SubGraph { id, subgraph_ids, node_ids, edge_ids }
    }
}
