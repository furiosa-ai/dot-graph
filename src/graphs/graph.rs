use crate::{
    edge::{Edge, EdgeId, EdgeMap},
    graphs::{igraph::IGraph, subgraph::SubGraph},
    node::{Node, NodeId},
    DotGraphError,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::io::Write;

pub type GraphId = String;

pub type SubTree = HashMap<GraphId, HashSet<GraphId>>;

#[derive(Debug, Clone)]
/// A `Graph` serves as a database of the entire dot graph.
/// It holds all subgraphs, nodes, and edges in the graph as respective sets.
/// `SubGraph`s hold ids of its children, nodes, and edges
/// such that it can be referenced in `Graph`'s `subgraphs`, `nodes`, and `edges`.
///
/// **All subgraphs, nodes, and edges in the graph MUST HAVE UNIQUE IDS.**
/// Trying to initialize a `Graph` with duplicate subgraphs, nodes, or edges will panic.
pub struct Graph {
    /// Name of the entire graph
    pub id: GraphId,

    /// All subgraphs in the graph (subgraph ids must be unique)
    pub subgraphs: HashSet<SubGraph>,

    /// All nodes in the graph (node ids must be unique)
    pub nodes: HashSet<Node>,

    /// All edges in the graph (edge ids must be unique)
    pub edges: HashSet<Edge>,

    /// Parent-children relationships of the subgraphs
    pub subtree: SubTree,

    /// Map constructed from edges, in forward direction
    fwdmap: EdgeMap,
    /// Map constructed from edges, in backward direction
    bwdmap: EdgeMap,
}

impl Graph {
    /// Constructs a new `graph`
    pub fn new(
        id: GraphId,
        igraphs: &[IGraph],
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<Graph, DotGraphError> {
        assert!(is_set(igraphs));
        assert!(is_set(nodes));
        assert!(is_set(edges)); 

        let subgraphs: HashSet<SubGraph> = igraphs.par_iter().map(|igraph| igraph.encode()).collect();
        let nodes: HashSet<Node> = nodes.par_iter().cloned().collect();
        let edges: HashSet<Edge> = edges.par_iter().cloned().collect();

        let (fwdmap, bwdmap) = make_edge_maps(&nodes, &edges);

        let subtree = make_subtree(&subgraphs);

        Ok(Graph {
            id,
            subgraphs,
            nodes,
            edges,
            subtree,
            fwdmap,
            bwdmap,
        })
    }

    /// Topologically sort nodes in this `Graph`.
    pub fn topsort(&self) -> Vec<NodeId> {
        let mut indegrees: HashMap<NodeId, usize> = HashMap::new();
        for (to, froms) in &self.bwdmap {
            indegrees.insert(to.clone(), froms.len());
        }

        let mut visited: HashSet<NodeId> = HashSet::new();

        let mut queue = VecDeque::new();
        for (node, _) in indegrees.iter().filter(|&(_, &indegree)| indegree == 0) {
            queue.push_back(node.clone());
            visited.insert(node.clone());
        }

        let mut sorted = Vec::new();
        while let Some(id) = queue.pop_front() {
            sorted.push(id.clone());
            if let Some(tos) = self.fwdmap.get(&id) {
                for to in tos {
                    let indegree = indegrees.get_mut(to).unwrap();
                    *indegree -= 1;
                    if *indegree == 0 {
                        queue.push_back(to.clone());
                        visited.insert(to.clone());
                    }
                }
            }
        }

        sorted
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
        let node_ids: HashSet<NodeId> = self
            .nodes
            .par_iter()
            .filter_map(|node| node.id.starts_with(prefix).then_some(node.id.clone()))
            .collect();

        self.extract(node_ids)
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
    pub fn neighbors(&self, center: &NodeId, depth: usize) -> Result<Option<Graph>, DotGraphError> {
        self.nodes.get(center).map_or(
            Err(DotGraphError::NoSuchNode(center.clone(), self.id.clone())),
            |_| {
                let mut visited = HashSet::new();
                let mut frontier: VecDeque<(NodeId, usize)> = VecDeque::new();
                frontier.push_back((center.clone(), 0));

                while let Some((id, vicinity)) = frontier.pop_front() {
                    if vicinity > depth || !visited.insert(id.clone()) {
                        continue;
                    }

                    let tos = self.fwdmap.get(&id).unwrap();
                    let froms = self.bwdmap.get(&id).unwrap();
                    let nexts = tos.union(froms);

                    frontier.extend(nexts.map(|next| (next.clone(), vicinity + 1)));
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
    pub fn subgraph(&self, root: &GraphId) -> Result<Option<Graph>, DotGraphError> {
        self.collect_nodes(root).map_or(
            Err(DotGraphError::NoSuchSubGraph(root.to_string(), self.id.clone())),
            |nodes| {
                let node_ids = nodes.par_iter().map(|node| node.id.clone()).collect();

                Ok(self.extract(node_ids))
            },
        )
    }

    fn extract(&self, node_ids: HashSet<NodeId>) -> Option<Graph> {
        if node_ids.is_empty() {
            return None;
        }

        let mut nodes = HashSet::new();
        for id in node_ids {
            if let Some(node) = self.search_node(&id) {
                nodes.insert(node.clone());
            }
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            let (from, to) = &edge.id;

            if self.search_node(from).is_some() && self.search_node(to).is_some() {
                edges.insert(edge.clone());
            }
        }

        let subgraphs: HashSet<SubGraph> = self
            .subgraphs
            .par_iter()
            .map(|subgraph| subgraph.extract_node_and_edge(&nodes, &edges))
            .collect();

        let empty_subgraph_ids = empty_subgraph_ids(&subgraphs);

        let subgraphs: HashSet<SubGraph> = subgraphs
            .par_iter()
            .filter_map(|subgraph| subgraph.extract_subgraph(&empty_subgraph_ids))
            .collect();
        
        let (fwdmap, bwdmap) = make_edge_maps(&nodes, &edges);

        let subtree = make_subtree(&subgraphs);

        Some(Graph {
            id: self.id.clone(),
            subgraphs,
            nodes,
            edges,
            subtree,
            fwdmap,
            bwdmap,
        })
    }

    /// Search for a subgraph by `id`
    pub fn search_subgraph(&self, id: &GraphId) -> Option<&SubGraph> {
        self.subgraphs.get(id)
    }

    /// Search for a node by `id`
    pub fn search_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Search for an edge by `id`
    pub fn search_edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Get all children subgraphs by `id`
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph with `id`,
    /// `Ok` with collected subgraphs.
    pub fn collect_subgraphs(&self, id: &GraphId) -> Result<HashSet<&SubGraph>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children: HashSet<&SubGraph> =
                    children.par_iter().map(|id| self.search_subgraph(id).unwrap()).collect();
                Ok(children)
            },
        )
    }

    /// Collect all nodes in a subgraph by `id`
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph with `id`,
    /// `Ok` with collected nodes.
    pub fn collect_nodes(&self, id: &GraphId) -> Result<HashSet<&Node>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children_nodes = children
                    .iter()
                    .map(|id| self.collect_nodes(id).unwrap())
                    .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

                let subgraph = self.search_subgraph(id).unwrap();
                let subgraph_nodes: HashSet<&Node> = subgraph.node_ids.par_iter().map(|id| self.search_node(id).unwrap()).collect();

                let nodes = subgraph_nodes.union(&children_nodes).cloned().collect();

                Ok(nodes)
            },
        )
    }

    /// Collect all edges in a subgraph by `id`
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph with `id`,
    /// `Ok` with collected edges. 
    pub fn collect_edges(&self, id: &GraphId) -> Result<HashSet<&Edge>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children_edges = children
                    .iter()
                    .map(|id| self.collect_edges(id).unwrap())
                    .fold(HashSet::new(), |acc, edges| acc.union(&edges).cloned().collect());

                let subgraph = self.search_subgraph(id).unwrap();
                let subgraph_edges: HashSet<&Edge> = subgraph.edge_ids.par_iter().map(|id| self.search_edge(id).unwrap()).collect();

                let edges = subgraph_edges.union(&children_edges).cloned().collect();

                Ok(edges)
            },
        )
    }

    /// Retrieve all nodes that are the predecessors of the node with `id`.
    ///
    /// # Returns
    ///
    /// `Err` if there is no node with `id`,
    /// `Ok` with a set of ids of predecessor nodes.
    pub fn froms(&self, id: &NodeId) -> Result<HashSet<NodeId>, DotGraphError> {
        self.bwdmap.get(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |froms| Ok(froms.clone())
        )
    }

    /// Retrieve all nodes that are the successors of the node with `id`.
    ///
    /// # Returns
    ///
    /// `Err` if there is no node with `id`,
    /// `Ok` with a set of ids of successor nodes.
    pub fn tos(&self, id: &NodeId) -> Result<HashSet<NodeId>, DotGraphError> {
        self.fwdmap.get(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |tos| Ok(tos.clone())
        )
    }

    /// Write the graph to dot format.
    pub fn to_dot<W: ?Sized>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        let root = self.subgraphs.get(&self.id).unwrap();

        root.to_dot(0, &self, writer)
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

fn make_edge_maps(nodes: &HashSet<Node>, edges: &HashSet<Edge>) -> (EdgeMap, EdgeMap) {
    let mut fwdmap = EdgeMap::new();
    let mut bwdmap = EdgeMap::new();

    for edge in edges {
        let (from, to) = &edge.id;

        fwdmap.entry(from.clone()).or_default().insert(to.clone());
        bwdmap.entry(to.clone()).or_default().insert(from.clone());
    }

    for node in nodes {
        let id = &node.id;

        fwdmap.entry(id.clone()).or_insert(HashSet::new());
        bwdmap.entry(id.clone()).or_insert(HashSet::new());
    }

    (fwdmap, bwdmap)
}

fn make_subtree(subgraphs: &HashSet<SubGraph>) -> SubTree {
    let mut subtree = HashMap::new();

    for subgraph in subgraphs {
        let children: HashSet<String> = subgraph.subgraph_ids.par_iter().cloned().collect();
        subtree.insert(subgraph.id.clone(), children);
    }

    subtree
}

fn empty_subgraph_ids(subgraphs: &HashSet<SubGraph>) -> HashSet<GraphId> {
    let mut empty_subgraph_ids: HashSet<GraphId> = HashSet::new();

    loop {
        let updated_empty_subgraph_ids: HashSet<GraphId> = subgraphs
            .par_iter()
            .filter_map(|subgraph| {
                let nonempty_subgraph_ids: HashSet<&GraphId> = subgraph
                    .subgraph_ids
                    .par_iter()
                    .filter_map(|id| (!empty_subgraph_ids.contains(id)).then_some(id))
                    .collect();

                let is_empty = nonempty_subgraph_ids.is_empty()
                    && subgraph.node_ids.is_empty()
                    && subgraph.edge_ids.is_empty();

                is_empty.then_some(subgraph.id.clone())
            })
            .collect();

        if updated_empty_subgraph_ids.len() == empty_subgraph_ids.len() {
            break;
        }

        empty_subgraph_ids = updated_empty_subgraph_ids;
    }

    empty_subgraph_ids
}
