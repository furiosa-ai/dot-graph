use crate::{
    edge::edge::{Edge, EdgeMap},
    graph::{igraph::IGraph, subgraph::SubGraph},
    node::node::Node,
};
use bimap::BiMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

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

impl Graph {
    pub fn new(id: String, root: IGraph, nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let nodes = Self::topsort(nodes, &edges);
        let nlookup = Self::get_nlookup(&nodes);
        let elookup = Self::get_elookup(&edges);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &nlookup);
        let root = root.encode(&nlookup, &elookup);

        Graph {
            id,
            root,
            nodes,
            nlookup,
            edges,
            elookup,
            fwdmap,
            bwdmap,
        }
    }

    pub fn filter(&self, prefix: &str) -> Option<Graph> {
        let nodes: HashSet<usize> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if node.id.starts_with(prefix) {
                    Some(idx)
                } else {
                    None
                }
            })
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

    pub fn extract(&self, extract: HashSet<usize>) -> Option<Graph> {
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

        self.root.extract(&nreplace, &ereplace).map(|root| {
            let nlookup = Self::get_nlookup(&nodes);
            let elookup = Self::get_elookup(&edges);
            let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &nlookup);

            Graph {
                id: self.id.clone(),
                root,
                nodes,
                nlookup,
                edges,
                elookup,
                fwdmap,
                bwdmap,
            }
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
                (froms.iter())
                    .map(|&idx| self.nodes[idx].id.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn tos(&self, id: &str) -> HashSet<&str> {
        self.nlookup
            .get_by_left(id)
            .map(|idx| {
                let tos = self.fwdmap.get(idx).cloned().unwrap_or_default();
                (tos.iter())
                    .map(|&idx| self.nodes[idx].id.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn topsort(nodes: Vec<Node>, edges: &[Edge]) -> Vec<Node> {
        let lookup = Self::get_nlookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(edges, &lookup);

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

    fn get_nlookup(nodes: &[Node]) -> BiMap<String, usize> {
        (nodes.iter().enumerate())
            .map(|(idx, node)| (node.id.clone(), idx))
            .collect()
    }

    fn get_elookup(edges: &[Edge]) -> BiMap<(String, String), usize> {
        (edges.iter().enumerate())
            .map(|(idx, edge)| ((edge.from.clone(), edge.to.clone()), idx))
            .collect()
    }

    fn get_edgemaps(edges: &[Edge], lookup: &BiMap<String, usize>) -> (EdgeMap, EdgeMap) {
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

    pub fn to_dot(&self) -> String {
        self.root.to_dot(0, &self.nodes, &self.edges)
    }
}
