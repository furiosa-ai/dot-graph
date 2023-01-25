use std::collections::{ HashSet, HashMap, VecDeque };
use std::boxed::Box;
use bimap::BiMap;
use crate::structs::{ Graph, SubGraph, IGraph, Node, Edge, EdgeMap };

impl Graph {
    pub fn new(id: String, root: IGraph, nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let nodes = Self::topsort(nodes, &edges);
        let nlookup = Self::get_nlookup(&nodes);
        let elookup = Self::get_elookup(&edges);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &nlookup);
        let root = root.encode(&nlookup, &elookup);

        Graph { id, root, nodes, nlookup, edges, elookup, fwdmap, bwdmap }
    }

    pub fn filter(&self, prefix: &str) -> Option<Graph> {
        let mut nodes = Vec::new();
        let mut nreplace = HashMap::new();
        for (idx, node) in (&self.nodes).iter().enumerate() {
            if node.id.starts_with(prefix) {
                nodes.push(node.clone());
                nreplace.insert(idx, nodes.len() - 1);
            }
        }

        let mut edges = Vec::new();
        let mut ereplace = HashMap::new();
        for (idx, edge) in (&self.edges).iter().enumerate() {
            if edge.from.starts_with(prefix) && edge.to.starts_with(prefix) {
                edges.push(edge.clone());
                ereplace.insert(idx, edges.len() - 1);
            }
        }

        let root = self.root.filter(&nreplace, &ereplace);
        match root {
            Some(root) => {
                let nlookup = Self::get_nlookup(&nodes);
                let elookup = Self::get_elookup(&edges);
                let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &nlookup);

                Some(Graph { id: self.id.clone(), root, nodes, nlookup, edges, elookup, fwdmap, bwdmap })
            },
            None => None
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
                froms.iter().map(|idx| self.nodes[*idx].id.as_str()).collect()
            },
            None => HashSet::new()
        }
    }

    pub fn tos(&self, id: &str) -> HashSet<&str> { 
        match self.nlookup.get_by_left(id) {
            Some(idx) => {
                let empty = HashSet::new();
                let tos = self.fwdmap.get(idx).cloned().unwrap_or(empty);
                tos.iter().map(|idx| self.nodes[*idx].id.as_str()).collect()
            },
            None => HashSet::new()
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
        
        let nodes = sorted.iter().map(|idx| nodes[*idx].clone()).collect::<Vec<Node>>();

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
        self.root.to_dot(0, &self.nodes, &self.edges)
    }
}

impl SubGraph {
    pub fn filter(&self, nreplace: &HashMap<usize, usize>, ereplace: &HashMap<usize, usize>) -> Option<SubGraph> {
        let mut subgraphs = Vec::new();
        for subgraph in &self.subgraphs {
            match (*subgraph).filter(nreplace, ereplace) {
                Some(subgraph) => subgraphs.push(Box::new(subgraph)),
                None => {},
            }
        }

        let mut nodes = Vec::new();
        for node in &self.nodes {
            match nreplace.get(&node) {
                Some(node) => nodes.push(*node),
                None => {},
            }
        }

        let mut edges = Vec::new();
        for edge in &self.edges {
            match ereplace.get(&edge) {
                Some(edge) => edges.push(*edge),
                None => {},
            }
        }

        if subgraphs.is_empty() && nodes.is_empty() && edges.is_empty() {
            None
        } else {
            Some(SubGraph { id: self.id.clone(), subgraphs, nodes, edges })
        }
    }

    pub fn to_dot(&self, indent: usize, nodes: &Vec<Node>, edges: &Vec<Edge>) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        if indent == 0 {
            dot.push_str(&format!("{}digraph DAG {{\n", tabs));
        } else {
            dot.push_str(&format!("{}subgraph {} {{\n", tabs, self.id));
        }

        for subgraph in &self.subgraphs {
            dot.push_str(&subgraph.to_dot(indent + 1, nodes, edges));
        }

        for node in &self.nodes {
            let node = &nodes[*node];
            dot.push_str(&format!("{}{}\n", tabs, &node.to_dot(indent + 1)));
        }

        for edge in &self.edges {
            let edge = &edges[*edge];
            dot.push_str(&format!("{}{}\n", tabs, &edge.to_dot(indent + 1)));
        }

        dot.push_str(&format!("{} }}\n", tabs));

        dot
    }
}

impl IGraph {
    pub fn encode(&self, nlookup: &BiMap<String, usize>, elookup: &BiMap<(String, String), usize>) -> SubGraph {
        let subgraphs: Vec<Box<SubGraph>> = self.subgraphs.iter().map(|subgraph| Box::new((*subgraph).encode(nlookup, elookup))).collect();
        let nodes: Vec<usize> = self.nodes.iter().map(|node| nlookup.get_by_left(&node.id).unwrap()).cloned().collect();
        let edges: Vec<usize> = self.edges.iter().map(|edge| elookup.get_by_left(&(edge.from.clone(), edge.to.clone())).unwrap()).cloned().collect();

        SubGraph { id: self.id.clone(), subgraphs, nodes, edges }
    }
}
