use crate::{
    edge::{Edge, EdgeMap},
    graphs::{igraph::IGraph, subgraph::SubGraph},
    node::Node,
    DotGraphError,
};
use bimap::BiMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::io::Write;

pub type IGraphIndex = usize;
pub type SubGraphIndex = usize;
pub type NodeIndex = usize;
pub type EdgeIndex = usize;
pub type SubTree = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone)]
/// A `Graph` serves as a database of the entire dot graph.
/// It holds all subgraphs, nodes, and edges in the graph as respective vectors.
///
/// `SubGraph`s hold indices of its children, nodes, and edges
/// such that it can be referenced in `Graph`'s `subgraphs`, `nodes`, and `edges`.
///
/// **All nodes and edges in the graph MUST be UNIQUE.**
/// Trying to initialize a `Graph` with duplicate nodes or edges will panic.
pub struct Graph {
    /// Name of the entire graph
    pub id: String,

    /// Parent-children relationships of the subgraphs
    pub subtree: SubTree,

    /// All subgraphs in the graph
    pub subgraphs: Vec<SubGraph>,
    /// Lookup for subgraphs using subgraph id
    pub slookup: BiMap<String, SubGraphIndex>,

    /// All nodes in the graph, topologically sorted (nodes must be unique)
    pub nodes: Vec<Node>,
    /// Lookup for nodes using node id
    pub nlookup: BiMap<String, NodeIndex>,

    /// All edges in the graph (edges must be unique)
    pub edges: Vec<Edge>,
    /// Lookup for edges using edge endpoint ids, i.e, (from, to)
    pub elookup: BiMap<(String, String), EdgeIndex>,
    /// Map constructed from edges, in forward direction
    pub fwdmap: EdgeMap,
    /// Map constructed from edges, in backward direction
    pub bwdmap: EdgeMap,
}

impl Graph {
    /// Constructs a new `graph` 
    pub fn new(id: String, igraphs: &[IGraph], nodes: &[Node], edges: &[Edge]) -> Result<Graph, DotGraphError> {
        assert!(is_set(nodes));
        assert!(is_set(edges));

        let sorted_nodes = topsort(nodes, edges);
        if sorted_nodes.len() < nodes.len() {
            return Err(DotGraphError::Cycle(id));
        }

        let nlookup = make_nlookup(&sorted_nodes);
        let nodes = sorted_nodes;

        let elookup = make_elookup(edges);
        let (fwdmap, bwdmap) = make_edge_maps(edges, &nlookup);
        let edges = Vec::from(edges);

        let slookup = make_ilookup(igraphs);
        let subgraphs: Vec<SubGraph> =
            igraphs.par_iter().map(|igraph| igraph.encode(&slookup, &nlookup, &elookup)).collect();
        let subtree = make_subtree(&subgraphs);

        Ok(Graph { id, subtree, subgraphs, slookup, nodes, nlookup, edges, elookup, fwdmap, bwdmap })
    }

    /// Constructs a new `Graph`, with nodes starting with the given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix of node id
    ///
    /// # Returns
    ///
    /// A `Graph` wrapped in `Some` if the filter is valid, i.e., there exists some node matching
    /// the prefix, or `None` otherwise. 
    pub fn filter(&self, prefix: &str) -> Option<Graph> {
        let node_idxs: HashSet<NodeIndex> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter_map(|(idx, node)| node.id.starts_with(prefix).then_some(idx))
            .collect();

        self.extract(node_idxs)
    }

    /// Constructs a new `Graph`, given a center node and depth limit.
    ///
    /// # Arguments
    ///
    /// * `center` - Id of the center node 
    /// * `depth` - Depth limit of the desired neighborhood
    ///
    /// # Returns
    ///
    /// `Err` if there is no node named `center`,
    /// `Ok` with some `Graph` otherwise.
    pub fn neighbors(&self, center: &str, depth: usize) -> Result<Option<Graph>, DotGraphError> {
        self.nlookup.get_by_left(center).map_or(
            Err(DotGraphError::NoSuchNode(center.to_string(), self.id.clone())),
            |center| {
                let mut visited = HashSet::new();
                let mut frontier: VecDeque<(NodeIndex, usize)> = VecDeque::new();
                frontier.push_back((*center, 0));

                while let Some((node, vicinity)) = frontier.pop_front() {
                    if vicinity > depth || !visited.insert(node) {
                        continue;
                    }

                    let tos = self.fwdmap.get(&node).unwrap();
                    let froms = self.bwdmap.get(&node).unwrap();
                    let nexts = tos.union(froms);

                    frontier.extend(nexts.map(|&next| (next, vicinity + 1)));
                }

                Ok(self.extract(visited))
            },
        )
    }

    /// Constructs a new `Graph`, with a new `root`.
    ///
    /// # Arguments
    ///
    /// * `root` - Id of the new root subgraph
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph named `root`,
    /// `Ok` with some `Graph` otherwise.
    pub fn subgraph(&self, root: &str) -> Result<Option<Graph>, DotGraphError> {
        self.slookup.get_by_left(root).map_or(
            Err(DotGraphError::NoSuchSubGraph(root.to_string(), self.id.clone())),
            |&idx| {
                let root = &self.subgraphs[idx];
                let node_idxs = root.collect_nodes(&self.subgraphs);

                Ok(self.extract(node_idxs))
            },
        )
    }

    fn extract(&self, node_idxs: HashSet<NodeIndex>) -> Option<Graph> {
        if node_idxs.is_empty() {
            return None;
        }

        let mut nodes = Vec::new();
        let mut nreplace = HashMap::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            if node_idxs.contains(&idx) {
                nodes.push(node.clone());
                nreplace.insert(idx, nreplace.len());
            }
        }

        let mut edges = Vec::new();
        let mut ereplace = HashMap::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            let from = self.nlookup.get_by_left(&edge.from).unwrap();
            let to = self.nlookup.get_by_left(&edge.to).unwrap();

            if node_idxs.contains(from) && node_idxs.contains(to) {
                edges.push(edge.clone());
                ereplace.insert(idx, ereplace.len());
            }
        }

        let subgraphs: Vec<SubGraph> = self
            .subgraphs
            .par_iter()
            .map(|subgraph| subgraph.replace_node_and_edge(&nreplace, &ereplace))
            .collect();

        let empty_subgraph_idxs = empty_subgraph_idxs(&subgraphs);

        let mut sreplace = HashMap::new();
        for idx in 0..subgraphs.len() {
            if !empty_subgraph_idxs.contains(&idx) {
                sreplace.insert(idx, sreplace.len());
            }
        }

        let subgraphs: Vec<SubGraph> = subgraphs
            .par_iter()
            .filter_map(|subgraph| subgraph.replace_subgraph(&sreplace))
            .collect();

        let subtree = make_subtree(&subgraphs);
        let slookup = make_slookup(&subgraphs);
        let nlookup = make_nlookup(&nodes);
        let elookup = make_elookup(&edges);
        let (fwdmap, bwdmap) = make_edge_maps(&edges, &nlookup);

        Some(Graph {
            id: self.id.clone(),
            subtree,
            subgraphs,
            slookup,
            nodes,
            nlookup,
            edges,
            elookup,
            fwdmap,
            bwdmap,
        })
    }

    /// Search for a subgraph by `id`
    pub fn search_subgraph(&self, id: &str) -> Option<&SubGraph> {
        self.slookup.get_by_left(id).map(|&idx| &self.subgraphs[idx])
    }

    /// Search for a node by `id`
    pub fn search_node(&self, id: &str) -> Option<&Node> {
        self.nlookup.get_by_left(id).map(|&idx| &self.nodes[idx])
    }

    /// Search for an edge by `from`, `to` ids
    pub fn search_edge(&self, from: &str, to: &str) -> Option<&Edge> {
        self.elookup.get_by_left(&(from.to_string(), to.to_string())).map(|&idx| &self.edges[idx])
    }

    /// Get all children subgraphs by `id`
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph with `id`,
    /// `Ok` with a vector of children subgraphs.
    pub fn children(&self, id: &str) -> Result<Vec<&SubGraph>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children: Vec<&SubGraph> = children.par_iter().map(|id| self.search_subgraph(id).unwrap()).collect();
                Ok(children)
            }
        )
    }

    /// Retrieve all nodes that are the predecessors of the node with `id`.
    ///
    /// # Returns
    ///
    /// `Err` if there is no node with `id`,
    /// `Ok` with a set of ids of predecessor nodes.
    pub fn froms(&self, id: &str) -> Result<HashSet<&str>, DotGraphError> {
        self.nlookup.get_by_left(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |idx| {
                let froms = self.bwdmap.get(idx).cloned().unwrap();
                let froms = (froms.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect();
                Ok(froms)
            },
        )
    }

    /// Retrieve all nodes that are the successors of the node with `id`.
    ///
    /// # Returns
    ///
    /// `Err` if there is no node with `id`,
    /// `Ok` with a set of ids of successor nodes.
    pub fn tos(&self, id: &str) -> Result<HashSet<&str>, DotGraphError> {
        self.nlookup.get_by_left(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |idx| {
                let tos = self.fwdmap.get(idx).cloned().unwrap();
                let tos = (tos.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect();
                Ok(tos)
            },
        )
    }

    /// Write the graph to dot format.
    pub fn to_dot<W: ?Sized>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        let &root = self.slookup.get_by_left(&self.id).unwrap();
        let root = &self.subgraphs[root];

        root.to_dot(0, &self.subgraphs, &self.nodes, &self.edges, writer)
    }
}

fn is_set<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut unique = HashSet::new();
    iter.into_iter().all(move |x| unique.insert(x))
}

fn topsort(nodes: &[Node], edges: &[Edge]) -> Vec<Node> {
    let nlookup = make_nlookup(nodes);
    let (fwdmap, bwdmap) = make_edge_maps(edges, &nlookup);

    let mut indegrees: HashMap<NodeIndex, usize> = (0..nodes.len()).map(|idx| (idx, 0)).collect();
    for (&to, froms) in &bwdmap {
        indegrees.insert(to, froms.len());
    }

    let mut visited: HashSet<NodeIndex> = HashSet::new();

    let mut queue = VecDeque::new();
    for (&node, _) in indegrees.iter().filter(|&(_, &indegree)| indegree == 0) {
        queue.push_back(node);
        visited.insert(node);
    }

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop_front() {
        sorted.push(nodes[node].clone());
        if let Some(tos) = fwdmap.get(&node) {
            for to in tos {
                let indegree = indegrees.get_mut(to).unwrap();
                *indegree -= 1;
                if *indegree == 0 {
                    queue.push_back(*to);
                    visited.insert(*to);
                }
            }
        }
    }

    sorted
}

fn make_ilookup(subgraphs: &[IGraph]) -> BiMap<String, IGraphIndex> {
    (subgraphs.iter().enumerate()).map(|(idx, subgraph)| (subgraph.id.clone(), idx)).collect()
}

fn make_slookup(subgraphs: &[SubGraph]) -> BiMap<String, SubGraphIndex> {
    (subgraphs.iter().enumerate()).map(|(idx, subgraph)| (subgraph.id.clone(), idx)).collect()
}

fn make_nlookup(nodes: &[Node]) -> BiMap<String, NodeIndex> {
    (nodes.iter().enumerate()).map(|(idx, node)| (node.id.clone(), idx)).collect()
}

fn make_elookup(edges: &[Edge]) -> BiMap<(String, String), EdgeIndex> {
    (edges.iter().enumerate())
        .map(|(idx, edge)| ((edge.from.clone(), edge.to.clone()), idx))
        .collect()
}

fn make_edge_maps(edges: &[Edge], nlookup: &BiMap<String, NodeIndex>) -> (EdgeMap, EdgeMap) {
    let mut fwdmap = EdgeMap::new();
    let mut bwdmap = EdgeMap::new();

    for edge in edges {
        let &from = nlookup.get_by_left(edge.from.as_str()).unwrap();
        let &to = nlookup.get_by_left(edge.to.as_str()).unwrap();

        fwdmap.entry(from).or_default().insert(to);
        bwdmap.entry(to).or_default().insert(from);
    }

    for &idx in nlookup.right_values() {
        fwdmap.entry(idx).or_insert(HashSet::new());
        bwdmap.entry(idx).or_insert(HashSet::new());
    }

    (fwdmap, bwdmap)
}

fn make_subtree(subgraphs: &[SubGraph]) -> SubTree {
    let mut subtree = HashMap::new();

    for subgraph in subgraphs {
        let children: HashSet<String> = subgraph.subgraph_idxs.par_iter().map(|&idx| subgraphs[idx].id.clone()).collect();
        subtree.insert(subgraph.id.clone(), children);
    }

    subtree
}

fn empty_subgraph_idxs(subgraphs: &[SubGraph]) -> HashSet<SubGraphIndex> {
    let mut empty_subgraph_idxs: HashSet<usize> = HashSet::new();

    loop {
        let updated_empty_subgraph_idxs: HashSet<usize> = subgraphs
            .par_iter()
            .enumerate()
            .filter_map(|(idx, subgraph)| {
                let nonempty_subgraph_idxs: Vec<usize> = subgraph
                    .subgraph_idxs
                    .par_iter()
                    .filter(|idx| !empty_subgraph_idxs.contains(idx))
                    .cloned()
                    .collect();

                let is_empty = nonempty_subgraph_idxs.is_empty()
                    && subgraph.node_idxs.is_empty()
                    && subgraph.edge_idxs.is_empty();

                is_empty.then_some(idx)
            })
            .collect();

        if updated_empty_subgraph_idxs.len() == empty_subgraph_idxs.len() {
            break;
        }

        empty_subgraph_idxs = updated_empty_subgraph_idxs;
    }

    empty_subgraph_idxs
}
