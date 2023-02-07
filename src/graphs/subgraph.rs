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
    pub subgraphs: Vec<SubGraphIndex>,
    pub nodes: Vec<NodeIndex>,
    pub edges: Vec<EdgeIndex>,
}

impl SubGraph {
    pub fn is_empty(&self, empty_subgraph_idxs: &HashSet<SubGraphIndex>) -> bool {
        let subgraphs: Vec<usize> = self
            .subgraphs
            .par_iter()
            .filter(|subgraph| !empty_subgraph_idxs.contains(subgraph))
            .cloned()
            .collect();

        subgraphs.is_empty() && self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn collect(&self, subgraphs: &[SubGraph]) -> HashSet<NodeIndex> {
        let node_idxs = self
            .subgraphs
            .iter()
            .map(|&subgraph| {
                let subgraph = &subgraphs[subgraph];
                subgraph.collect(subgraphs)
            })
            .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

        let node_idxs = node_idxs.union(&HashSet::from_iter(self.nodes.clone())).cloned().collect();

        node_idxs
    }

    pub fn extract_nodes(
        &self,
        nreplace: &HashMap<usize, usize>,
        ereplace: &HashMap<usize, usize>,
    ) -> SubGraph {
        let nodes: Vec<NodeIndex> =
            self.nodes.par_iter().filter_map(|node| nreplace.get(node).cloned()).collect();

        let edges: Vec<EdgeIndex> =
            self.edges.par_iter().filter_map(|edge| ereplace.get(edge).cloned()).collect();

        SubGraph { id: self.id.clone(), subgraphs: self.subgraphs.clone(), nodes, edges }
    }

    pub fn extract_subgraph(&self, sreplace: &HashMap<usize, usize>) -> Option<SubGraph> {
        let subgraphs: Vec<SubGraphIndex> = self
            .subgraphs
            .par_iter()
            .filter_map(|subgraph| sreplace.get(subgraph).cloned())
            .collect();

        if subgraphs.is_empty() && self.nodes.is_empty() && self.edges.is_empty() {
            None
        } else {
            Some(SubGraph {
                id: self.id.clone(),
                subgraphs,
                nodes: self.nodes.clone(),
                edges: self.edges.clone(),
            })
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

        for &subgraph in &self.subgraphs {
            let subgraph = &subgraphs[subgraph];
            dot.push_str(&subgraph.to_dot(indent + 1, subgraphs, nodes, edges));
        }

        for &node in &self.nodes {
            let node = &nodes[node];
            writeln!(dot, "{}{}", tabs, node.to_dot(indent + 1)).unwrap();
        }

        for &edge in &self.edges {
            let edge = &edges[edge];
            writeln!(dot, "{}{}", tabs, edge.to_dot(indent + 1)).unwrap();
        }

        writeln!(dot, "{} }}", tabs).unwrap();

        dot
    }
}
