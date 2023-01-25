use std::collections::{ HashSet, HashMap, VecDeque };
use std::boxed::Box;
use bimap::BiMap;
use crate::structs::{ Graph, SubGraph, IGraph, Node, Edge, EdgeMap };

impl Graph {
    pub fn new(id: String, root: IGraph, nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let nodes = Self::topsort(nodes, &edges);
        let lookup = Self::get_lookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &lookup);
        let root = root.encode(&lookup);

        Graph { id, root, nodes, lookup, edges, fwdmap, bwdmap }
    }

    pub fn filter(&self, prefix: &str) -> Graph {
        let mut nodes = Vec::new();
        let mut replace = HashMap::new();
        for (idx, node) in (&self.nodes).iter().enumerate() {
            if node.id.starts_with(prefix) {
                nodes.push(node.clone());
                replace.insert(idx, nodes.len() - 1);
            }
        }

        let mut edges = Vec::new();
        for edge in &self.edges {
            if edge.from.starts_with(prefix) && edge.to.starts_with(prefix) {
                edges.push(edge.clone());
            }
        }

        let root = self.root.filter(&replace);

        let lookup = Self::get_lookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &lookup);

        // TODO filter subgraphs also
        Graph { id: self.id.clone(), root, nodes, lookup, edges, fwdmap, bwdmap }
    }

    pub fn search(&self, id: &str) -> Option<&Node> {
        match self.lookup.get_by_left(id) {
            Some(idx) => Some(&self.nodes[*idx]),
            None => None,
        }
    }

    pub fn froms(&self, id: &str) -> HashSet<&str> { 
        match self.lookup.get_by_left(id) {
            Some(idx) => {
                let empty = HashSet::new();
                let froms = self.bwdmap.get(idx).cloned().unwrap_or(empty);
                froms.iter().map(|idx| self.nodes[*idx].id.as_str()).collect()
            },
            None => HashSet::new()
        }
    }

    pub fn tos(&self, id: &str) -> HashSet<&str> { 
        match self.lookup.get_by_left(id) {
            Some(idx) => {
                let empty = HashSet::new();
                let tos = self.fwdmap.get(idx).cloned().unwrap_or(empty);
                tos.iter().map(|idx| self.nodes[*idx].id.as_str()).collect()
            },
            None => HashSet::new()
        }
    }

    fn topsort(nodes: Vec<Node>, edges: &Vec<Edge>) -> Vec<Node> {
        let lookup = Self::get_lookup(&nodes);
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
        
        let nodes = sorted.iter().map(|idx| nodes[*idx].clone()).collect::<Vec<Node>>();

        nodes
    }

    fn get_lookup(nodes: &Vec<Node>) -> BiMap<String, usize> {
        let mut bimap = BiMap::new();
        for (idx, node) in nodes.iter().enumerate() {
            bimap.insert(node.id.clone(), idx);
        }

        bimap
    }

    fn get_edgemaps(edges: &Vec<Edge>, lookup: &BiMap<String, usize>)-> (EdgeMap, EdgeMap) {
        let mut fwdmap = EdgeMap::new();
        let mut bwdmap = EdgeMap::new();

        for edge in edges {
            let from = lookup.get_by_left(edge.from.as_str()).unwrap();
            let to = lookup.get_by_left(edge.to.as_str()).unwrap();

            let entry = fwdmap.entry(*from).or_insert(HashSet::new());
            entry.insert(*to);

            let entry = bwdmap.entry(*to).or_insert(HashSet::new());
            entry.insert(*from);
        }

        (fwdmap, bwdmap)
    }

    pub fn to_dot(&self) -> String {
        self.root.to_dot(0, &self.nodes)
    }
}

impl SubGraph {
    pub fn filter(&self, replace: &HashMap<usize, usize>) -> SubGraph {
        let mut subgraphs = Vec::new();
        for subgraph in &self.subgraphs {
            subgraphs.push(Box::new((*subgraph).filter(replace)));
        }

        let mut nodes = Vec::new();
        for node in &self.nodes {
            match replace.get(&node) {
                Some(node) => nodes.push(*node),
                None => {},
            }
        }

        let mut edges = Vec::new();
        for edge in &self.edges {
            match (replace.get(&edge.0), replace.get(&edge.1)) {
                (Some(from), Some(to)) => edges.push((*from, *to)),
                _ => {},
            }
        }

        SubGraph { id: self.id.clone(), subgraphs, nodes, edges } 
    }

    pub fn to_dot(&self, indent: usize, nodes: &Vec<Node>) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        if indent == 0 {
            dot.push_str(&format!("{}digraph DAG {{\n", tabs));
        } else {
            dot.push_str(&format!("{}subgraph {} {{\n", tabs, self.id));
        }

        for subgraph in &self.subgraphs {
            dot.push_str(&subgraph.to_dot(indent + 1, nodes));
        }

        for node in &self.nodes {
            let node = &nodes[*node];
            dot.push_str(&format!("{}{}\n", tabs, &node.to_dot(indent + 1)));
        }

        for edge in &self.edges {
            let edge = Edge {
                from: nodes[edge.0].id.clone(),
                to: nodes[edge.1].id.clone(),
            };
            dot.push_str(&format!("{}{}\n", tabs, &edge.to_dot(indent + 1)));
        }

        dot.push_str("}\n");

        dot
    }
}

impl IGraph {
    pub fn encode(&self, lookup: &BiMap<String, usize>) -> SubGraph {
        let subgraphs: Vec<Box<SubGraph>> = self.subgraphs.iter().map(|subgraph| Box::new((*subgraph).encode(lookup))).collect();
        let nodes: Vec<usize> = self.nodes.iter().map(|node| lookup.get_by_left(&node.id).unwrap()).cloned().collect();
        let edges: Vec<(usize, usize)> = self.edges.iter().map(|edge| (lookup.get_by_left(&edge.from).unwrap().clone(), lookup.get_by_left(&edge.to).unwrap().clone())).collect();

        SubGraph { id: self.id.clone(), subgraphs, nodes, edges }
    }
}
