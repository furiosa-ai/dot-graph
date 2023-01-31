use crate::{
    edge::edge::Edge,
    node::node::Node,
};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SubGraph {
    pub id: String,

    pub subgraphs: Vec<Box<SubGraph>>,

    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
}

impl SubGraph {
    pub fn extract(
        &self,
        nreplace: &HashMap<usize, usize>,
        ereplace: &HashMap<usize, usize>,
    ) -> Option<SubGraph> {
        let subgraphs: Vec<Box<SubGraph>> = self.subgraphs.par_iter()
            .filter_map(|subgraph| if let Some(subgraph) = subgraph.extract(nreplace, ereplace) {
                Some(Box::new(subgraph))
            } else {
                None
            })
            .collect();

        let nodes: Vec<usize> = self.nodes.par_iter() 
            .filter_map(|node| if let Some(&node) = nreplace.get(node) {
                Some(node)
            } else {
                None
            })
            .collect();

        let edges: Vec<usize> = self.edges.par_iter() 
            .filter_map(|edge| if let Some(&edge) = ereplace.get(edge) {
                Some(edge)
            } else {
                None
            })
            .collect();

        if subgraphs.is_empty() && nodes.is_empty() && edges.is_empty() {
            None
        } else {
            Some(SubGraph {
                id: self.id.clone(),
                subgraphs,
                nodes,
                edges,
            })
        }
    }

    pub fn to_dot(&self, indent: usize, nodes: &[Node], edges: &[Edge]) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        if indent == 0 {
            dot.push_str(&format!("{}digraph DAG {{\n", tabs));
        } else {
            dot.push_str(&format!("{}subgraph {} {{\n", tabs, self.id));
        }

        for subgraph in &self.subgraphs {
            dot.push_str(&subgraph.to_dot(indent + 1, nodes, edges));
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
