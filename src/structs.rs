use std::collections::{ BTreeMap, HashSet, HashMap };
use bimap::BiMap;

// TODO current graph data structure assumes that the root DAG doesn't hold any nodes by itself
#[derive(Debug, Clone)]
pub struct Graph {
    pub id: String,
    pub root: SubGraph,

    pub nodes: Vec<Node>,
    pub lookup: BiMap<String, usize>,

    pub edges: Vec<Edge>,
    pub fwdmap: EdgeMap,
    pub bwdmap: EdgeMap,
}

#[derive(Debug, Clone)]
pub struct SubGraph {
    pub id: String,

    pub subgraphs: Vec<Box<SubGraph>>,

    pub nodes: Vec<usize>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct IGraph {
    pub id: String,
    pub subgraphs: Vec<Box<IGraph>>,

    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub id: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

pub type EdgeMap = HashMap<usize, HashSet<usize>>;
