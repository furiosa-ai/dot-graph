use crate::{edge::Edge, node::Node};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io::Write;

type SubGraphIndex = usize;
type NodeIndex = usize;
type EdgeIndex = usize;

#[derive(Debug, Clone)]
pub struct SubGraph {
    pub id: String,
    pub subgraph_idxs: Vec<SubGraphIndex>,
    pub node_idxs: Vec<NodeIndex>,
    pub edge_idxs: Vec<EdgeIndex>,
}

impl SubGraph {
    pub fn collect(&self, subgraphs: &[SubGraph]) -> HashSet<NodeIndex> {
        let node_idxs = self
            .subgraph_idxs
            .iter()
            .map(|&idx| {
                let subgraph = &subgraphs[idx];
                subgraph.collect(subgraphs)
            })
            .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

        let node_idxs =
            node_idxs.union(&HashSet::from_iter(self.node_idxs.clone())).cloned().collect();

        node_idxs
    }

    pub fn extract_nodes(
        &self,
        nreplace: &HashMap<NodeIndex, NodeIndex>,
        ereplace: &HashMap<EdgeIndex, EdgeIndex>,
    ) -> SubGraph {
        let id = self.id.clone();

        let subgraph_idxs = self.subgraph_idxs.clone();

        let node_idxs: Vec<NodeIndex> =
            self.node_idxs.par_iter().filter_map(|idx| nreplace.get(idx).cloned()).collect();

        let edge_idxs: Vec<EdgeIndex> =
            self.edge_idxs.par_iter().filter_map(|idx| ereplace.get(idx).cloned()).collect();

        SubGraph { id, subgraph_idxs, node_idxs, edge_idxs }
    }

    pub fn extract_subgraph(
        &self,
        sreplace: &HashMap<SubGraphIndex, SubGraphIndex>,
    ) -> Option<SubGraph> {
        let subgraph_idxs: Vec<SubGraphIndex> =
            self.subgraph_idxs.par_iter().filter_map(|idx| sreplace.get(idx).cloned()).collect();

        if subgraph_idxs.is_empty() && self.node_idxs.is_empty() && self.edge_idxs.is_empty() {
            None
        } else {
            let id = self.id.clone();
            let node_idxs = self.node_idxs.clone();
            let edge_idxs = self.edge_idxs.clone();

            Some(SubGraph { id, subgraph_idxs, node_idxs, edge_idxs })
        }
    }

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

        for &idx in &self.subgraph_idxs {
            let subgraph = &subgraphs[idx];
            subgraph.to_dot(indent + 1, subgraphs, nodes, edges, writer)?;
        }

        for &idx in &self.node_idxs {
            let node = &nodes[idx];
            writeln!(writer, "{}", tabs)?;
            node.to_dot(indent + 1, writer)?;
        }

        for &idx in &self.edge_idxs {
            let edge = &edges[idx];
            writeln!(writer, "{}", tabs)?;
            edge.to_dot(indent + 1, writer)?;
        }

        writeln!(writer, "{} }}", tabs)?;

        Ok(())
    }
}
