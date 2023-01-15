use std::collections::{BTreeSet, BTreeMap, VecDeque};
use bimap::BiMap;
use crate::structs::{ Graph, SubGraph, Node };

impl Graph {
    pub fn topsort(&mut self) {
        let lookup = Self::encode(&self.nodes);
        let (edgemap, mut indegrees) = {
            let mut edgemap: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
            let mut indegrees: BTreeMap<usize, usize> = (0..self.nodes.len()).map(|idx| (idx, 0)).collect();

            for edge in &self.edges {
                let from = lookup.get_by_left(edge.from.as_str()).unwrap();
                let to = lookup.get_by_left(edge.to.as_str()).unwrap();

                let entry = edgemap.entry(*from).or_insert(BTreeSet::new());
                entry.insert(*to);

                let entry = indegrees.get_mut(to).unwrap();
                *entry += 1;
            }

           (edgemap, indegrees)
        };

        let mut visited: BTreeMap<usize, bool> = BTreeMap::new();
        let mut queue = {
            let mut queue = VecDeque::new();
            for (node, indegree) in &indegrees {
                if *indegree == 0 {
                    queue.push_back(*node);
                    visited.insert(*node, true);
                }
            }

            queue
        };

        let sorted = {
            let mut sorted = Vec::new();
            while let Some(node) = queue.pop_front() {
                sorted.push(node);
                if let Some(tos) = edgemap.get(&node) {
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

            sorted.iter().map(|idx| self.nodes[*idx].clone()).collect::<Vec<Node>>()
        };
 
        self.nodes = sorted;
    }

    fn encode(nodes: &Vec<Node>) -> BiMap<&str, usize> {
        let mut bimap = BiMap::new();
        for (idx, node) in nodes.iter().enumerate() {
            bimap.insert(node.id.as_str(), idx);
        }

        bimap
    }
}

impl SubGraph {
}
