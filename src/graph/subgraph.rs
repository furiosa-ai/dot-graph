use crate::{edge::edge::Edge, node::node::Node};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct SubGraph {
    pub id: String,
    pub subgraphs: Vec<usize>,
    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
}

impl SubGraph {
    pub fn is_empty(&self, empty: &HashSet<usize>) -> bool {
        let subgraphs: Vec<usize> = self
            .subgraphs
            .par_iter()
            .filter(|subgraph| !empty.contains(subgraph))
            .cloned()
            .collect();

        subgraphs.is_empty() && self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn collect(&self, subgraphs: &[SubGraph]) -> HashSet<usize> {
        let nodes = self
            .subgraphs
            .iter()
            .map(|&subgraph| {
                let subgraph = &subgraphs[subgraph];
                subgraph.collect(subgraphs)
            })
            .fold(HashSet::new(), |acc, nodes| {
                acc.union(&nodes).cloned().collect()
            });

        let nodes = nodes
            .union(&HashSet::from_iter(self.nodes.clone()))
            .cloned()
            .collect();

        nodes
    }

    pub fn extract_nodes(
        &self,
        nreplace: &HashMap<usize, usize>,
        ereplace: &HashMap<usize, usize>,
    ) -> SubGraph {
        let nodes: Vec<usize> = self
            .nodes
            .par_iter()
            .filter_map(|node| {
                if let Some(&node) = nreplace.get(node) {
                    Some(node)
                } else {
                    None
                }
            })
            .collect();

        let edges: Vec<usize> = self
            .edges
            .par_iter()
            .filter_map(|edge| {
                if let Some(&edge) = ereplace.get(edge) {
                    Some(edge)
                } else {
                    None
                }
            })
            .collect();

        SubGraph {
            id: self.id.clone(),
            subgraphs: self.subgraphs.clone(),
            nodes,
            edges,
        }
    }

    pub fn extract_subgraph(&self, sreplace: &HashMap<usize, usize>) -> Option<SubGraph> {
        let subgraphs: Vec<usize> = self
            .subgraphs
            .par_iter()
            .filter_map(|subgraph| {
                if let Some(&subgraph) = sreplace.get(subgraph) {
                    Some(subgraph)
                } else {
                    None
                }
            })
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
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        if indent == 0 {
            dot.push_str(&format!("{}digraph {} {{\n", tabs, self.id));
        } else {
            dot.push_str(&format!("{}subgraph {} {{\n", tabs, self.id));
        }

        for &subgraph in &self.subgraphs {
            let subgraph = &subgraphs[subgraph];
            dot.push_str(&subgraph.to_dot(indent + 1, subgraphs, nodes, edges));
        }

        for &node in &self.nodes {
            let node = &nodes[node];
            dot.push_str(&format!("{}{}\n", tabs, &node.to_dot(indent + 1)));
        }

        for &edge in &self.edges {
            let edge = &edges[edge];
            dot.push_str(&format!("{}{}\n", tabs, &edge.to_dot(indent + 1)));
        }

        dot.push_str(&format!("{} }}\n", tabs));

        dot
    }
}
