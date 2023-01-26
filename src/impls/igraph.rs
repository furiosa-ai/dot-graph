use crate::structs::{IGraph, SubGraph};
use bimap::BiMap;

impl IGraph {
    pub fn encode(
        &self,
        nlookup: &BiMap<String, usize>,
        elookup: &BiMap<(String, String), usize>,
    ) -> SubGraph {
        let subgraphs: Vec<Box<SubGraph>> = self
            .subgraphs
            .iter()
            .map(|subgraph| Box::new((*subgraph).encode(nlookup, elookup)))
            .collect();
        let nodes: Vec<usize> = self
            .nodes
            .iter()
            .map(|node| nlookup.get_by_left(&node.id).unwrap())
            .cloned()
            .collect();
        let edges: Vec<usize> = self
            .edges
            .iter()
            .map(|edge| {
                elookup
                    .get_by_left(&(edge.from.clone(), edge.to.clone()))
                    .unwrap()
            })
            .cloned()
            .collect();

        SubGraph {
            id: self.id.clone(),
            subgraphs,
            nodes,
            edges,
        }
    }
}
