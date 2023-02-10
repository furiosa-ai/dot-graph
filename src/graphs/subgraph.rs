use crate::{
    edge::EdgeId,
    graphs::graph::{Graph, GraphId},
    node::NodeId,
};
use rayon::prelude::*;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A `SubGraph` holds indices of its own nodes and edges,
/// and its children subgraphs.
///
/// ```
/// subgraph A {
///     subgraph B {
///         node C
///     }
/// }
/// ```
/// In such a case, `subgraph B` holds `node C`, not `subgraph A`.
pub struct SubGraph {
    /// Name of the subgraph
    pub id: GraphId,

    /// Ids of its children subgraphs, referenced in `Graph`
    pub subgraph_ids: HashSet<GraphId>,
    /// Ids of its own nodes, referened in `Graph`
    pub node_ids: HashSet<NodeId>,
    /// Ids of its own edges, referenced in `Graph`
    pub edge_ids: HashSet<EdgeId>,
}

impl Hash for SubGraph {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<GraphId> for SubGraph {
    fn borrow(&self) -> &GraphId {
        &self.id
    }
}

impl SubGraph {
    pub fn extract_nodes_and_edges(
        &self,
        node_ids: &HashSet<&NodeId>,
        edge_ids: &HashSet<&EdgeId>,
    ) -> SubGraph {
        let id = self.id.clone();

        let subgraph_ids = self.subgraph_ids.clone();

        let node_ids: HashSet<NodeId> =
            self.node_ids.par_iter().filter(|id| node_ids.contains(id)).cloned().collect();

        let edge_ids: HashSet<EdgeId> =
            self.edge_ids.par_iter().filter(|id| edge_ids.contains(id)).cloned().collect();

        SubGraph { id, subgraph_ids, node_ids, edge_ids }
    }

    pub fn extract_subgraph(&self, subgraph_ids: &HashSet<&GraphId>) -> Option<SubGraph> {
        let subgraph_ids: HashSet<GraphId> =
            self.subgraph_ids.par_iter().filter(|id| subgraph_ids.contains(id)).cloned().collect();

        if subgraph_ids.is_empty() && self.node_ids.is_empty() && self.edge_ids.is_empty() {
            None
        } else {
            let id = self.id.clone();
            let node_ids = self.node_ids.clone();
            let edge_ids = self.edge_ids.clone();

            Some(SubGraph { id, subgraph_ids, node_ids, edge_ids })
        }
    }

    /// Write the graph to dot format.
    pub fn to_dot<W: ?Sized>(
        &self,
        graph: &Graph,
        indent: usize,
        writer: &mut W,
    ) -> std::io::Result<()>
    where
        W: Write,
    {
        let tabs = "\t".repeat(indent);

        if indent == 0 {
            writeln!(writer, "digraph {} {{", self.id)?;
        } else {
            writeln!(writer, "{}subgraph {} {{", tabs, self.id)?;
        }

        for id in &self.subgraph_ids {
            let subgraph = graph.search_subgraph(id).unwrap();
            subgraph.to_dot(graph, indent + 1, writer)?;
        }

        for id in &self.node_ids {
            let node = graph.search_node(id).unwrap();
            writeln!(writer, "{}", tabs)?;
            node.to_dot(indent + 1, writer)?;
        }

        for id in &self.edge_ids {
            let edge = graph.search_edge(id).unwrap();
            writeln!(writer, "{}", tabs)?;
            edge.to_dot(indent + 1, writer)?;
        }

        writeln!(writer, "{} }}", tabs)?;

        Ok(())
    }
}
