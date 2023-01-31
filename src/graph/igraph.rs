use bimap::BiMap;
use crate::{
    edge::edge::Edge,
    node::node::Node,
    graph::subgraph::SubGraph,
};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct IGraph {
    pub id: String,
    pub subgraphs: Vec<Box<IGraph>>,

    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl IGraph {
    pub fn encode(
        &self,
        nlookup: &BiMap<String, usize>,
        elookup: &BiMap<(String, String), usize>,
    ) -> SubGraph {
        let subgraphs: Vec<Box<SubGraph>> = (self.subgraphs.par_iter())
            .map(|subgraph| Box::new(subgraph.encode(nlookup, elookup)))
            .collect();

        let nodes: Vec<usize> = (self.nodes.par_iter())
            .map(|node| nlookup.get_by_left(&node.id).unwrap())
            .cloned()
            .collect();

        let edges: Vec<usize> = (self.edges.par_iter()) 
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
