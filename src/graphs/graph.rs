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

type IGraphIndex = usize;
type SubGraphIndex = usize;
type NodeIndex = usize;
type EdgeIndex = usize;
type SubTree = HashMap<SubGraphIndex, Vec<SubGraphIndex>>;

#[derive(Debug, Clone)]
pub struct Graph {
    pub id: String,

    pub subtree: SubTree,

    pub subgraphs: Vec<SubGraph>,
    pub slookup: BiMap<String, SubGraphIndex>,

    pub nodes: Vec<Node>,
    pub nlookup: BiMap<String, NodeIndex>,

    pub edges: Vec<Edge>,
    pub elookup: BiMap<(String, String), EdgeIndex>,
    pub fwdmap: EdgeMap,
    pub bwdmap: EdgeMap,
}

impl Graph {
    pub fn new(id: String, igraphs: &[IGraph], nodes: &[Node], edges: &[Edge]) -> Graph {
        assert!(is_set(nodes));
        assert!(is_set(edges));

        let sorted_nodes = topsort(nodes, edges);
        let nlookup = make_nlookup(&sorted_nodes);
        let nodes = sorted_nodes;

        let elookup = make_elookup(edges);
        let (fwdmap, bwdmap) = make_edge_maps(edges, &nlookup);
        let edges = Vec::from(edges);

        let slookup = make_ilookup(igraphs);
        let subgraphs: Vec<SubGraph> =
            igraphs.par_iter().map(|igraph| igraph.encode(&slookup, &nlookup, &elookup)).collect();
        let subtree = make_subtree(&subgraphs);

        Graph { id, subtree, subgraphs, slookup, nodes, nlookup, edges, elookup, fwdmap, bwdmap }
    }

    pub fn filter(&self, prefix: &str) -> Option<Graph> {
        let node_idxs: HashSet<NodeIndex> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter_map(|(idx, node)| node.id.starts_with(prefix).then_some(idx))
            .collect();

        self.extract(node_idxs)
    }

    pub fn neighbors(&self, center: &str, depth: usize) -> Result<Option<Graph>, DotGraphError> {
        self.nlookup.get_by_left(center).map_or(
            Err(DotGraphError::NoSuchNode(center.to_string(), self.id.clone())),
            |center| {
                let mut visited = HashSet::new();
                let mut frontier: VecDeque<(NodeIndex, usize)> = VecDeque::new();
                frontier.push_back((*center, 0));

                let empty = HashSet::new();
                while let Some((node, vicinity)) = frontier.pop_front() {
                    if vicinity > depth || !visited.insert(node) {
                        continue;
                    }

                    let tos = self.fwdmap.get(&node).unwrap_or(&empty);
                    let froms = self.bwdmap.get(&node).unwrap_or(&empty);
                    let nexts = tos.union(froms);

                    frontier.extend(nexts.map(|&next| (next, vicinity + 1)));
                }

                Ok(self.extract(visited))
            },
        )
    }

    pub fn subgraph(&self, root: &str) -> Result<Option<Graph>, DotGraphError> {
        self.slookup.get_by_left(root).map_or(
            Err(DotGraphError::NoSuchSubGraph(root.to_string(), self.id.clone())),
            |&root| {
                let root = &self.subgraphs[root];
                let node_idxs = root.collect(&self.subgraphs);

                Ok(self.extract(node_idxs))
            },
        )
    }

    pub fn extract(&self, node_idxs: HashSet<NodeIndex>) -> Option<Graph> {
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
            .map(|subgraph| subgraph.extract_nodes(&nreplace, &ereplace))
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
            .filter_map(|subgraph| subgraph.extract_subgraph(&sreplace))
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

    pub fn search(&self, id: &str) -> Option<&Node> {
        self.nlookup.get_by_left(id).map(|&idx| &self.nodes[idx])
    }

    pub fn froms(&self, id: &str) -> Result<HashSet<&str>, DotGraphError> {
        self.nlookup.get_by_left(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |idx| {
                let froms = self.bwdmap.get(idx).cloned().unwrap_or_default();
                let froms = (froms.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect();
                Ok(froms)
            },
        )
    }

    pub fn tos(&self, id: &str) -> Result<HashSet<&str>, DotGraphError> {
        self.nlookup.get_by_left(id).map_or(
            Err(DotGraphError::NoSuchNode(id.to_string(), self.id.clone())),
            |idx| {
                let tos = self.fwdmap.get(idx).cloned().unwrap_or_default();
                let tos = (tos.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect();
                Ok(tos)
            },
        )
    }

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

    (fwdmap, bwdmap)
}

fn make_subtree(subgraphs: &[SubGraph]) -> SubTree {
    let mut subtree = HashMap::new();

    for (idx, subgraph) in subgraphs.iter().enumerate() {
        if !subgraph.subgraph_idxs.is_empty() {
            subtree.insert(idx, subgraph.subgraph_idxs.clone());
        }
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
