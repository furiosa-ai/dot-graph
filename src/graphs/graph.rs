use crate::{
    edge::{Edge, EdgeMap},
    graphs::{igraph::IGraph, subgraph::SubGraph},
    node::Node,
};
use bimap::BiMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct Graph {
    pub id: String,

    pub subtree: HashMap<usize, Vec<usize>>,

    pub subgraphs: Vec<SubGraph>,
    pub slookup: BiMap<String, usize>,

    pub nodes: Vec<Node>,
    pub nlookup: BiMap<String, usize>,

    pub edges: Vec<Edge>,
    pub elookup: BiMap<(String, String), usize>,
    pub fwdmap: EdgeMap,
    pub bwdmap: EdgeMap,
}

impl Graph {
    pub fn new(id: String, igraphs: Vec<IGraph>, nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let nodes = topsort(nodes, &edges);
        let slookup = make_ilookup(&igraphs);
        let nlookup = make_nlookup(&nodes);
        let elookup = make_elookup(&edges);
        let (fwdmap, bwdmap) = make_edge_maps(&edges, &nlookup);
        let subgraphs: Vec<SubGraph> =
            igraphs.par_iter().map(|igraph| igraph.encode(&slookup, &nlookup, &elookup)).collect();
        let subtree = make_subtree(&subgraphs);

        Graph { id, subtree, subgraphs, slookup, nodes, nlookup, edges, elookup, fwdmap, bwdmap }
    }

    pub fn filter(&self, prefix: &str) -> Option<Graph> {
        let nodes: HashSet<usize> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter_map(|(idx, node)| node.id.starts_with(prefix).then_some(idx))
            .collect();

        self.extract(nodes)
    }

    pub fn neighbors(&self, center: &str, depth: usize) -> Option<Graph> {
        let center = self.nlookup.get_by_left(center).unwrap();

        let mut visited = HashSet::new();
        let mut frontier: VecDeque<(usize, usize)> = VecDeque::new();
        frontier.push_back((*center, 0));

        while let Some((node, vicinity)) = frontier.pop_front() {
            if vicinity > depth || visited.contains(&node) {
                continue;
            }

            visited.insert(node);

            let empty = HashSet::new();
            let tos = self.fwdmap.get(&node).unwrap_or(&empty);
            let froms = self.bwdmap.get(&node).unwrap_or(&empty);
            let nexts = tos.union(froms);

            frontier.extend(nexts.map(|&next| (next, vicinity + 1)));
        }

        self.extract(visited)
    }

    pub fn subgraph(&self, root: &str) -> Option<Graph> {
        let &root = self.slookup.get_by_left(root).unwrap();
        let root = &self.subgraphs[root];
        let nodes = root.collect(&self.subgraphs);

        self.extract(nodes)
    }

    pub fn extract(&self, extract: HashSet<usize>) -> Option<Graph> {
        if extract.is_empty() {
            return None;
        }

        let mut nodes = Vec::new();
        let mut nreplace = HashMap::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            if extract.contains(&idx) {
                nodes.push(node.clone());
                nreplace.insert(idx, nodes.len() - 1);
            }
        }

        let mut edges = Vec::new();
        let mut ereplace = HashMap::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            let from = self.nlookup.get_by_left(&edge.from).unwrap();
            let to = self.nlookup.get_by_left(&edge.to).unwrap();

            if extract.contains(from) && extract.contains(to) {
                edges.push(edge.clone());
                ereplace.insert(idx, edges.len() - 1);
            }
        }

        let subgraphs: Vec<SubGraph> = self
            .subgraphs
            .par_iter()
            .map(|subgraph| subgraph.extract_nodes(&nreplace, &ereplace))
            .collect();
        let mut empty: HashSet<usize> = HashSet::new();
        loop {
            let before = empty.len();

            empty = subgraphs
                .par_iter()
                .enumerate()
                .filter_map(|(idx, subgraph)| subgraph.is_empty(&empty).then_some(idx))
                .collect();

            let after = empty.len();
            if before == after {
                break;
            }
        }

        let mut sreplace = HashMap::new();
        for idx in 0..subgraphs.len() {
            if !empty.contains(&idx) {
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

    pub fn froms(&self, id: &str) -> HashSet<&str> {
        self.nlookup
            .get_by_left(id)
            .map(|idx| {
                let froms = self.bwdmap.get(idx).cloned().unwrap_or_default();
                (froms.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect()
            })
            .unwrap_or_default()
    }

    pub fn tos(&self, id: &str) -> HashSet<&str> {
        self.nlookup
            .get_by_left(id)
            .map(|idx| {
                let tos = self.fwdmap.get(idx).cloned().unwrap_or_default();
                (tos.iter()).map(|&idx| self.nodes[idx].id.as_str()).collect()
            })
            .unwrap_or_default()
    }

    pub fn to_dot(&self) -> String {
        let root = self.subgraphs.last().unwrap();

        root.to_dot(0, &self.subgraphs, &self.nodes, &self.edges)
    }
}

fn topsort(nodes: Vec<Node>, edges: &[Edge]) -> Vec<Node> {
    let lookup = make_nlookup(&nodes);
    let (fwdmap, bwdmap) = make_edge_maps(edges, &lookup);

    let mut indegrees: HashMap<usize, usize> = (0..nodes.len()).map(|idx| (idx, 0)).collect();
    for (&to, froms) in &bwdmap {
        indegrees.insert(to, froms.len());
    }

    let mut visited: HashSet<usize> = HashSet::new();

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
                if let Some(0) = (indegrees.get_mut(to)).map(|i| {
                    *i -= 1;
                    i
                }) {
                    queue.push_back(*to);
                    visited.insert(*to);
                }
            }
        }
    }

    sorted
}

fn make_ilookup(subgraphs: &[IGraph]) -> BiMap<String, usize> {
    (subgraphs.iter().enumerate()).map(|(idx, subgraph)| (subgraph.id.clone(), idx)).collect()
}

fn make_slookup(subgraphs: &[SubGraph]) -> BiMap<String, usize> {
    (subgraphs.iter().enumerate()).map(|(idx, subgraph)| (subgraph.id.clone(), idx)).collect()
}

fn make_nlookup(nodes: &[Node]) -> BiMap<String, usize> {
    (nodes.iter().enumerate()).map(|(idx, node)| (node.id.clone(), idx)).collect()
}

fn make_elookup(edges: &[Edge]) -> BiMap<(String, String), usize> {
    (edges.iter().enumerate())
        .map(|(idx, edge)| ((edge.from.clone(), edge.to.clone()), idx))
        .collect()
}

fn make_edge_maps(edges: &[Edge], lookup: &BiMap<String, usize>) -> (EdgeMap, EdgeMap) {
    let mut fwdmap = EdgeMap::new();
    let mut bwdmap = EdgeMap::new();

    for edge in edges {
        let &from = lookup.get_by_left(edge.from.as_str()).unwrap();
        let &to = lookup.get_by_left(edge.to.as_str()).unwrap();

        let entry = fwdmap.entry(from).or_default();
        entry.insert(to);

        let entry = bwdmap.entry(to).or_default();
        entry.insert(from);
    }

    (fwdmap, bwdmap)
}

fn make_subtree(subgraphs: &[SubGraph]) -> HashMap<usize, Vec<usize>> {
    let mut subtree = HashMap::new();

    for (idx, subgraph) in subgraphs.iter().enumerate() {
        if !subgraph.subgraphs.is_empty() {
            subtree.insert(idx, subgraph.subgraphs.clone());
        }
    }

    subtree
}
