use std::collections::{ BTreeMap, HashSet, HashMap };
use bimap::BiMap;

#[derive(Debug, Clone)]
pub struct Graph {
    pub id: String,
    pub root: SubGraph,

    pub nodes: Vec<Node>,
    pub nlookup: BiMap<String, usize>,

    pub edges: Vec<Edge>,
    pub elookup: BiMap<(String, String), usize>,
    pub fwdmap: EdgeMap,
    pub bwdmap: EdgeMap,
}

#[derive(Debug, Clone)]
pub struct SubGraph {
    pub id: String,

    pub subgraphs: Vec<Box<SubGraph>>,

    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
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
    pub attrs: BTreeMap<String, String>,
}

pub type EdgeMap = HashMap<usize, HashSet<usize>>;
