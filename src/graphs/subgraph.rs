use crate::{edge::Edge, node::Node};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

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
    pub fn is_empty(&self, empty_subgraph_idxs: &HashSet<SubGraphIndex>) -> bool {
        let nonempty_subgraph_idxs: Vec<usize> = self
            .subgraph_idxs
            .par_iter()
            .filter(|subgraph| !empty_subgraph_idxs.contains(subgraph))
            .cloned()
            .collect();

        nonempty_subgraph_idxs.is_empty() && self.node_idxs.is_empty() && self.edge_idxs.is_empty()
    }

    pub fn collect(&self, subgraphs: &[SubGraph]) -> HashSet<NodeIndex> {
        let node_idxs = self
            .subgraph_idxs
            .iter()
            .map(|&subgraph| {
                let subgraph = &subgraphs[subgraph];
                subgraph.collect(subgraphs)
            })
            .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

        let node_idxs = node_idxs.union(&HashSet::from_iter(self.node_idxs.clone())).cloned().collect();

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

    pub fn extract_subgraph(&self, sreplace: &HashMap<SubGraphIndex, SubGraphIndex>) -> Option<SubGraph> {
        let subgraph_idxs: Vec<SubGraphIndex> = self
            .subgraph_idxs
            .par_iter()
            .filter_map(|idx| sreplace.get(idx).cloned())
            .collect();

        if subgraph_idxs.is_empty() && self.node_idxs.is_empty() && self.edge_idxs.is_empty() {
            None
        } else {
            let id = self.id.clone();
            let node_idxs = self.node_idxs.clone();
            let edge_idxs = self.edge_idxs.clone();

            Some(SubGraph { id, subgraph_idxs, node_idxs, edge_idxs })
        }
    }

    pub fn to_dot(
        &self,
        indent: usize,
        subgraphs: &[SubGraph],
        nodes: &[Node],
        edges: &[Edge],
    ) -> String {
        let mut dot = String::new();
        let tabs = "\t".repeat(indent);

        if indent == 0 {
            writeln!(dot, "digraph {} {{", self.id).unwrap();
        } else {
            writeln!(dot, "{}subgraph {} {{", tabs, self.id).unwrap();
        }

        for &idx in &self.subgraph_idxs {
            let subgraph = &subgraphs[idx];
            dot.push_str(&subgraph.to_dot(indent + 1, subgraphs, nodes, edges));
        }

        for &idx in &self.node_idxs {
            let node = &nodes[idx];
            writeln!(dot, "{}{}", tabs, node.to_dot(indent + 1)).unwrap();
        }

        for &idx in &self.edge_idxs {
            let edge = &edges[idx];
            writeln!(dot, "{}{}", tabs, edge.to_dot(indent + 1)).unwrap();
        }

        writeln!(dot, "{} }}", tabs).unwrap();

        dot
    }
}
