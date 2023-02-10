use crate::{
    edge::Edge,
    graphs::graph::{EdgeIndex, NodeIndex, SubGraphIndex},
    node::Node,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io::Write;

pub type GraphId = String;

#[derive(Debug, Clone)]
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

    /// Indices of its children subgraphs, referenced in `Graph`
    pub subgraph_ids: Vec<SubGraphIndex>,
    /// Indices of its own nodes, referened in `Graph`
    pub node_ids: Vec<NodeIndex>,
    /// Indices of its own edges, referenced in `Graph`
    pub edge_ids: Vec<EdgeIndex>,
}

impl SubGraph {
    /// Collect all nodes contained in this and all its descendant subgraphs
    pub fn collect_nodes(&self, subgraphs: &[SubGraph]) -> HashSet<NodeIndex> {
        let node_ids = self
            .subgraph_ids
            .iter()
            .map(|&idx| {
                let subgraph = &subgraphs[idx];
                subgraph.collect_nodes(subgraphs)
            })
            .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

        let node_ids =
            node_ids.union(&HashSet::from_iter(self.node_ids.clone())).cloned().collect();

        node_ids
    }

    pub fn replace_node_and_edge(
        &self,
        nreplace: &HashMap<NodeIndex, NodeIndex>,
        ereplace: &HashMap<EdgeIndex, EdgeIndex>,
    ) -> SubGraph {
        let id = self.id.clone();

        let subgraph_ids = self.subgraph_ids.clone();

        let node_ids: Vec<NodeIndex> =
            self.node_ids.par_iter().filter_map(|idx| nreplace.get(idx).cloned()).collect();

        let edge_ids: Vec<EdgeIndex> =
            self.edge_ids.par_iter().filter_map(|idx| ereplace.get(idx).cloned()).collect();

        SubGraph { id, subgraph_ids, node_ids, edge_ids }
    }

    pub fn replace_subgraph(
        &self,
        sreplace: &HashMap<SubGraphIndex, SubGraphIndex>,
    ) -> Option<SubGraph> {
        let subgraph_ids: Vec<SubGraphIndex> =
            self.subgraph_ids.par_iter().filter_map(|idx| sreplace.get(idx).cloned()).collect();

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
        indent: usize,
        subgraphs: &[SubGraph],
        nodes: &[Node],
        edges: &[Edge],
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

        for &idx in &self.subgraph_ids {
            let subgraph = &subgraphs[idx];
            subgraph.to_dot(indent + 1, subgraphs, nodes, edges, writer)?;
        }

        for &idx in &self.node_ids {
            let node = &nodes[idx];
            writeln!(writer, "{}", tabs)?;
            node.to_dot(indent + 1, writer)?;
        }

        for &idx in &self.edge_ids {
            let edge = &edges[idx];
            writeln!(writer, "{}", tabs)?;
            edge.to_dot(indent + 1, writer)?;
        }

        writeln!(writer, "{} }}", tabs)?;

        Ok(())
    }
}
