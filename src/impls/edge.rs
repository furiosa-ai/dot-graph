use crate::structs::Edge;

impl Edge {
    pub fn to_dot(&self, indent: usize) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        dot.push_str(&format!("{}{} -> {}", tabs, self.from, self.to));

        dot
    }
}
