use crate::structs::{Edge, EdgeMap, Graph, IGraph, Node};
use bimap::BiMap;
use std::collections::{HashMap, HashSet, VecDeque};

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
        let mut nodes = HashSet::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            if node.id.starts_with(prefix) {
                nodes.insert(idx);
            }
        }

        self.extract(nodes)
    }

    pub fn neighbors(&self, center: &str, depth: usize) -> Option<Graph> {
        let center = self.nlookup.get_by_left(center).unwrap();

        let mut visited = HashSet::new();
        let mut frontier: VecDeque<(usize, usize)> = VecDeque::new();
        frontier.push_back((*center, 0));

        while !frontier.is_empty() {
            let (node, vicinity) = frontier.pop_front().unwrap();

            if vicinity > depth {
                continue;
            }
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node);

            let empty = HashSet::new();
            let tos = self.fwdmap.get(&node).unwrap_or(&empty);
            let froms = self.bwdmap.get(&node).unwrap_or(&empty);
            let nexts = tos.union(froms);

            for next in nexts {
                frontier.push_back((*next, vicinity + 1));
            }
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

        let root = self.root.extract(&nreplace, &ereplace);
        match root {
            Some(root) => {
                let nlookup = Self::get_nlookup(&nodes);
                let elookup = Self::get_elookup(&edges);
                let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &nlookup);

                Some(Graph {
                    id: self.id.clone(),
                    root,
                    nodes,
                    nlookup,
                    edges,
                    elookup,
                    fwdmap,
                    bwdmap,
                })
            }
            None => None,
        }
    }

    pub fn search(&self, id: &str) -> Option<&Node> {
        match self.nlookup.get_by_left(id) {
            Some(idx) => Some(&self.nodes[*idx]),
            None => None,
        }
    }

    pub fn froms(&self, id: &str) -> HashSet<&str> {
        match self.nlookup.get_by_left(id) {
            Some(idx) => {
                let empty = HashSet::new();
                let froms = self.bwdmap.get(idx).cloned().unwrap_or(empty);
                froms
                    .iter()
                    .map(|idx| self.nodes[*idx].id.as_str())
                    .collect()
            }
            None => HashSet::new(),
        }
    }

    pub fn tos(&self, id: &str) -> HashSet<&str> {
        match self.nlookup.get_by_left(id) {
            Some(idx) => {
                let empty = HashSet::new();
                let tos = self.fwdmap.get(idx).cloned().unwrap_or(empty);
                tos.iter().map(|idx| self.nodes[*idx].id.as_str()).collect()
            }
            None => HashSet::new(),
        }
    }

    fn topsort(nodes: Vec<Node>, edges: &Vec<Edge>) -> Vec<Node> {
        let lookup = Self::get_nlookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(edges, &lookup);

        let mut indegrees: HashMap<usize, usize> = (0..nodes.len()).map(|idx| (idx, 0)).collect();
        for (to, froms) in &bwdmap {
            indegrees.insert(*to, froms.len());
        }

        let mut visited: HashMap<usize, bool> = HashMap::new();

        let mut queue = VecDeque::new();
        for (node, indegree) in &indegrees {
            if *indegree == 0 {
                queue.push_back(*node);
                visited.insert(*node, true);
            }
        }

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node);
            if let Some(tos) = fwdmap.get(&node) {
                for to in tos {
                    let indegree = indegrees.get_mut(to).unwrap();
                    *indegree -= 1;

                    if *indegree == 0 {
                        queue.push_back(*to);
                        visited.insert(*to, true);
                    }
                }
            }
        }

        let nodes = sorted
            .iter()
            .map(|idx| nodes[*idx].clone())
            .collect::<Vec<Node>>();

        nodes
    }

    fn get_nlookup(nodes: &Vec<Node>) -> BiMap<String, usize> {
        let mut bimap = BiMap::new();
        for (idx, node) in nodes.iter().enumerate() {
            bimap.insert(node.id.clone(), idx);
        }

        bimap
    }

    fn get_elookup(edges: &Vec<Edge>) -> BiMap<(String, String), usize> {
        let mut bimap = BiMap::new();
        for (idx, edge) in edges.iter().enumerate() {
            bimap.insert((edge.from.clone(), edge.to.clone()), idx);
        }

        bimap
    }

    fn get_edgemaps(edges: &Vec<Edge>, lookup: &BiMap<String, usize>) -> (EdgeMap, EdgeMap) {
        let mut fwdmap = EdgeMap::new();
        let mut bwdmap = EdgeMap::new();

        for edge in edges {
            let from = lookup.get_by_left(edge.from.as_str()).unwrap();
            let to = lookup.get_by_left(edge.to.as_str()).unwrap();

            let entry = fwdmap.entry(*from).or_default();
            entry.insert(*to);

            let entry = bwdmap.entry(*to).or_default();
            entry.insert(*from);
        }

        (fwdmap, bwdmap)
    }

    pub fn to_dot(&self) -> String {
        self.root.to_dot(0, &self.nodes, &self.edges)
    }
}
