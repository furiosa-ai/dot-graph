use crate::{
    attr::Attr,
    edge::{Edge, EdgeId},
    graphs::{graph::GraphId, subgraph::SubGraph},
    node::{Node, NodeId},
};

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use rayon::prelude::*;

#[derive(Debug, Clone, Eq)]
/// An `IGraph` is an intermediate representation, to be transformed to `SubGraph` after parsing.
/// It holds ids of its children subgraphs, nodes, and edges.
///
/// `SubGraph` is a more compact form of an `IGraph`, in the sense that it holds indices of
/// children subgraphs, nodes, and edges to be referenced in `Graph`.
pub(crate) struct IGraph {
    /// Name of the igraph
    id: GraphId,
    /// Its children subgraphs
    igraphs: HashSet<IGraph>,
    /// Its own nodes
    nodes: HashSet<Node>,
    /// Its own edges
    edges: HashSet<Edge>,
    /// Attributes of the graph in key, value mappings
    attrs: HashSet<Attr>,
}

impl PartialEq for IGraph {
    fn eq(&self, other: &IGraph) -> bool {
        self.id == other.id
    }
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
    pub(crate) fn new(
        id: GraphId,
        igraphs: HashSet<IGraph>,
        nodes: HashSet<Node>,
        edges: HashSet<Edge>,
        attrs: HashSet<Attr>,
    ) -> IGraph {
        IGraph { id, igraphs, nodes, edges, attrs }
    }

    /// Convert `IGraph` to a set of `SubGraph`s, an unfolded subgraph tree
    pub(crate) fn encode(&self) -> HashSet<SubGraph> {
        let mut subgraphs = self
            .igraphs
            .iter()
            .map(|igraph| igraph.encode())
            .fold(HashSet::new(), |acc, subgraphs| acc.union(&subgraphs).cloned().collect());

        let id = self.id.clone();

        let subgraph_ids: HashSet<GraphId> =
            (self.igraphs.par_iter()).map(|igraph| igraph.id.clone()).collect();

        let node_ids: HashSet<NodeId> =
            (self.nodes.par_iter()).map(|node| node.id.clone()).collect();

        let edge_ids: HashSet<EdgeId> =
            (self.edges.par_iter()).map(|edge| edge.id.clone()).collect();

        let attrs = self.attrs.clone();

        let subgraph = SubGraph { id, subgraph_ids, node_ids, edge_ids, attrs };

        subgraphs.insert(subgraph);

        subgraphs
    }
}
