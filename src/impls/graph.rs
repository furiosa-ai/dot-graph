use std::collections::{ HashSet, HashMap, VecDeque };
use bimap::BiMap;
use crate::structs::{ Graph, SubGraph, Node, Edge, EdgeMap };

impl Graph {
    pub fn new(subgraphs: Vec<SubGraph>, nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let lookup = Self::get_lookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&edges, &lookup);

        let mut graph = Graph { subgraphs, nodes, lookup, edges, fwdmap, bwdmap };
        graph.topsort();

        graph
    }

    fn topsort(&mut self) {
        let mut indegrees: HashMap<usize, usize> = (0..self.nodes.len()).map(|idx| (idx, 0)).collect();
        for (to, froms) in &self.bwdmap {
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
            if let Some(tos) = self.fwdmap.get(&node) {
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
        
        let nodes = sorted.iter().map(|idx| self.nodes[*idx].clone()).collect::<Vec<Node>>();
        let lookup = Self::get_lookup(&nodes);
        let (fwdmap, bwdmap) = Self::get_edgemaps(&self.edges, &lookup);

        self.nodes = nodes;
        self.lookup = lookup;
        self.fwdmap = fwdmap;
        self.bwdmap = bwdmap;
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
}

impl SubGraph {
}
