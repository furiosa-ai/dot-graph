use std::collections::{ BTreeMap, HashSet, HashMap };
use bimap::BiMap;

#[derive(Debug)]
pub struct Graph {
    pub subgraphs: Vec<SubGraph>,

    pub nodes: Vec<Node>,
    pub lookup: BiMap<String, usize>,

    pub edges: Vec<Edge>,
    pub fwdmap: EdgeMap,
    pub bwdmap: EdgeMap,
}

#[derive(Debug)]
pub struct SubGraph {
    pub id: String,
    pub parent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: String,
    pub parent: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

pub type EdgeMap = HashMap<usize, HashSet<usize>>;
