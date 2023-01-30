use crate::structs::{Edge, Node, SubGraph};
use std::collections::HashMap;

impl SubGraph {
    pub fn extract(
        &self,
        nreplace: &HashMap<usize, usize>,
        ereplace: &HashMap<usize, usize>,
    ) -> Option<SubGraph> {
        let mut subgraphs = Vec::new();
        for subgraph in &self.subgraphs {
            if let Some(subgraph) = subgraph.extract(nreplace, ereplace) {
                subgraphs.push(Box::new(subgraph));
            }
        }

        let mut nodes = Vec::new();
        for node in &self.nodes {
            if let Some(&node) = nreplace.get(node) {
                nodes.push(node);
            }
        }

        let mut edges = Vec::new();
        for edge in &self.edges {
            if let Some(&edge) = ereplace.get(edge) {
                edges.push(edge);
            }
        }

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
