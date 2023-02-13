use crate::{
    edge::{Edge, EdgeId},
    graphs::{igraph::IGraph, subgraph::SubGraph},
    node::{Node, NodeId},
    DotGraphError,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::io::Write;

pub type GraphId = String;

type SubTree = HashMap<GraphId, HashSet<GraphId>>;
type EdgeMap = HashMap<NodeId, HashSet<NodeId>>;

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
    id: GraphId,

    /// All subgraphs in the graph (subgraph ids must be unique)
    subgraphs: HashSet<SubGraph>,

    /// All nodes in the graph (node ids must be unique)
    nodes: HashSet<Node>,

    /// All edges in the graph (edge ids must be unique)
    edges: HashSet<Edge>,

    /// Parent-children relationships of the subgraphs
    subtree: SubTree,

    /// Map constructed from edges, in forward direction
    fwdmap: EdgeMap,
    /// Map constructed from edges, in backward direction
    bwdmap: EdgeMap,
}

impl Graph {
    /// Constructs a new `graph`
    pub(crate) fn new(
        id: GraphId,
        root: &IGraph,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<Graph, DotGraphError> {
        assert!(is_set(nodes));
        assert!(is_set(edges));

        let subgraphs: HashSet<SubGraph> = root.encode();
        let nodes: HashSet<Node> = nodes.par_iter().cloned().collect();
        let edges: HashSet<Edge> = edges.par_iter().cloned().collect();

        let (fwdmap, bwdmap) = make_edge_maps(&nodes, &edges);

        let subtree = make_subtree(&subgraphs);

        let graph = Graph { id: id.clone(), subgraphs, nodes, edges, subtree, fwdmap, bwdmap };

        if !graph.is_acyclic() {
            return Err(DotGraphError::Cycle(id));
        }

        Ok(graph)
    }

    pub fn id(&self) -> &GraphId {
        &self.id
    }

    pub fn subgraphs(&self) -> HashSet<&GraphId> {
        self.subgraphs.par_iter().map(|subgraph| &subgraph.id).collect()
    }

    pub fn nodes(&self) -> HashSet<&NodeId> {
        self.nodes.par_iter().map(|node| &node.id).collect()
    }

    pub fn edges(&self) -> HashSet<&EdgeId> {
        self.edges.par_iter().map(|edge| &edge.id).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.subgraphs.is_empty() && self.nodes.is_empty() && self.edges.is_empty()
    }

    fn is_acyclic(&self) -> bool {
        self.nodes.len() == self.topsort().len()
    }

    /// Topologically sort nodes in this `Graph`.
    ///
    /// # Returns
    ///
    /// A vector of topologically sorted node ids.
    pub fn topsort(&self) -> Vec<&NodeId> {
        let mut indegrees: HashMap<&NodeId, usize> = HashMap::new();
        for (to, froms) in &self.bwdmap {
            indegrees.insert(to, froms.len());
        }

        let mut visited: HashSet<&NodeId> = HashSet::new();

        let mut queue = VecDeque::new();
        for (&node, _) in indegrees.iter().filter(|&(_, &indegree)| indegree == 0) {
            queue.push_back(node);
            visited.insert(node);
        }

        let mut sorted = Vec::new();
        while let Some(id) = queue.pop_front() {
            sorted.push(id);
            if let Some(tos) = self.fwdmap.get(id) {
                for to in tos {
                    let indegree = indegrees.get_mut(to).unwrap();
                    *indegree -= 1;
                    if *indegree == 0 {
                        queue.push_back(to);
                        visited.insert(to);
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
    pub fn filter(&self, prefix: &str) -> Graph {
        let node_ids: HashSet<&NodeId> = self
            .nodes
            .par_iter()
            .filter_map(|node| node.id.starts_with(prefix).then_some(&node.id))
            .collect();

        self.extract(&node_ids)
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
    /// `Ok` with neighbors `Graph` otherwise.
    pub fn neighbors(&self, center: &NodeId, depth: usize) -> Result<Graph, DotGraphError> {
        self.nodes.get(center).map_or(
            Err(DotGraphError::NoSuchNode(center.clone(), self.id.clone())),
            |_| {
                let mut visited = HashSet::new();
                let mut frontier: VecDeque<(&NodeId, usize)> = VecDeque::new();
                frontier.push_back((center, 0));

                while let Some((id, vicinity)) = frontier.pop_front() {
                    if vicinity > depth || !visited.insert(id) {
                        continue;
                    }

                    let tos = self.fwdmap.get(id).unwrap();
                    let froms = self.bwdmap.get(id).unwrap();
                    let nexts = tos.union(froms);

                    frontier.extend(nexts.map(|next| (next, vicinity + 1)));
                }

                Ok(self.extract(&visited))
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
    /// `Ok` with subgraph-ed `Graph` otherwise.
    pub fn subgraph(&self, root: &GraphId) -> Result<Graph, DotGraphError> {
        self.collect_nodes(root).map_or(
            Err(DotGraphError::NoSuchSubGraph(root.to_string(), self.id.clone())),
            |node_ids| Ok(self.extract(&node_ids)),
        )
    }

    fn extract(&self, node_ids: &HashSet<&NodeId>) -> Graph {
        let mut nodes = HashSet::new();
        for id in node_ids {
            if let Some(node) = self.search_node(id) {
                nodes.insert(node.clone());
            }
        }
        let node_ids: HashSet<&NodeId> = nodes.par_iter().map(|node| &node.id).collect();

        let mut edges = HashSet::new();
        for edge in &self.edges {
            let (from, to) = &edge.id;

            if node_ids.get(from).is_some() && node_ids.get(to).is_some() {
                edges.insert(edge.clone());
            }
        }
        let edge_ids: HashSet<&EdgeId> = edges.par_iter().map(|edge| &edge.id).collect();

        let subgraphs: HashSet<SubGraph> = self
            .subgraphs
            .par_iter()
            .map(|subgraph| subgraph.extract_nodes_and_edges(&node_ids, &edge_ids))
            .collect();

        let empty_subgraph_ids = empty_subgraph_ids(&subgraphs);
        let subgraph_ids: HashSet<&GraphId> = self
            .subgraphs
            .par_iter()
            .filter_map(|subgraph| {
                (!empty_subgraph_ids.contains(&subgraph.id)).then_some(&subgraph.id)
            })
            .collect();

        let subgraphs: HashSet<SubGraph> = subgraphs
            .par_iter()
            .filter_map(|subgraph| subgraph.extract_subgraph(&subgraph_ids))
            .collect();

        let (fwdmap, bwdmap) = make_edge_maps(&nodes, &edges);

        let subtree = make_subtree(&subgraphs);

        Graph { id: self.id.clone(), subgraphs, nodes, edges, subtree, fwdmap, bwdmap }
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
    /// `Ok` with collected subgraph ids.
    pub fn collect_subgraphs(&self, id: &GraphId) -> Result<HashSet<&GraphId>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children: HashSet<&GraphId> =
                    children.par_iter().map(|id| &self.search_subgraph(id).unwrap().id).collect();
                Ok(children)
            },
        )
    }

    /// Collect all nodes in a subgraph by `id`
    ///
    /// # Returns
    ///
    /// `Err` if there is no subgraph with `id`,
    /// `Ok` with collected node ids.
    pub fn collect_nodes(&self, id: &GraphId) -> Result<HashSet<&NodeId>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children_nodes = children
                    .iter()
                    .map(|id| self.collect_nodes(id).unwrap())
                    .fold(HashSet::new(), |acc, nodes| acc.union(&nodes).cloned().collect());

                let subgraph = self.search_subgraph(id).unwrap();
                let subgraph_nodes: HashSet<&NodeId> = subgraph
                    .node_ids
                    .par_iter()
                    .map(|id| &self.search_node(id).unwrap().id)
                    .collect();

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
    /// `Ok` with collected edge ids.
    pub fn collect_edges(&self, id: &GraphId) -> Result<HashSet<&EdgeId>, DotGraphError> {
        self.subtree.get(id).map_or(
            Err(DotGraphError::NoSuchSubGraph(id.to_string(), self.id.clone())),
            |children| {
                let children_edges = children
                    .iter()
                    .map(|id| self.collect_edges(id).unwrap())
                    .fold(HashSet::new(), |acc, edges| acc.union(&edges).cloned().collect());

                let subgraph = self.search_subgraph(id).unwrap();
                let subgraph_edges: HashSet<&EdgeId> = subgraph
                    .edge_ids
                    .par_iter()
                    .map(|id| &self.search_edge(id).unwrap().id)
                    .collect();

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
    pub fn froms(&self, id: &NodeId) -> Result<HashSet<&NodeId>, DotGraphError> {
        self.bwdmap.get(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |froms| {
                let froms = froms.par_iter().map(|from| from).collect();
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
    pub fn tos(&self, id: &NodeId) -> Result<HashSet<&NodeId>, DotGraphError> {
        self.fwdmap.get(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |tos| {
                let tos = tos.par_iter().map(|to| to).collect();
                Ok(tos)
            },
        )
    }

    /// Write the graph to dot format.
    pub fn to_dot<W: ?Sized>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        let root = self.subgraphs.get(&self.id).unwrap();

        root.to_dot(self, 0, writer)
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

        fwdmap.entry(id.clone()).or_default();
        bwdmap.entry(id.clone()).or_default();
    }

    (fwdmap, bwdmap)
}

fn make_subtree(subgraphs: &HashSet<SubGraph>) -> SubTree {
    let mut subtree = HashMap::new();

    for subgraph in subgraphs {
        let children: HashSet<GraphId> = subgraph.subgraph_ids.par_iter().cloned().collect();
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
